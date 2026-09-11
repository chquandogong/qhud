//! Passive, process-free system sampling. Keep one collector for the life of
//! the poll thread: CPU and I/O values need two observations, not fresh objects.

mod disk;
pub mod gpu;

use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{DiskRefreshKind, Disks, MemoryRefreshKind, Networks, System};

pub const INTERVAL: Duration = Duration::from_secs(2);
const HISTORY_MS: u64 = 60_000;
const HISTORY_POINTS: usize = 30;
const MAX_GAP: Duration = Duration::from_secs(10);
const STORAGE_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Default, Serialize)]
pub struct SystemSample {
    pub at_ms: u64,
    pub cpu_pct: Option<f64>,
    pub memory_pct: Option<f64>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub gpu_pct: Option<f64>,
    pub disk_read_bps: Option<f64>,
    pub disk_write_bps: Option<f64>,
    pub disk_used_bytes: Option<u64>,
    pub disk_total_bytes: Option<u64>,
    pub network_rx_bps: Option<f64>,
    pub network_tx_bps: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SystemSnapshot {
    pub sampled_at_ms: u64,
    pub interval_ms: u64,
    pub cpu_count: Option<usize>,
    /// Sticky after the first successful reading; a later null gpu_pct is a gap.
    pub gpu: Option<GpuInfo>,
    pub current: SystemSample,
    pub history: Vec<SystemSample>,
}

pub struct Collector {
    system: System,
    networks: Networks,
    network_rates: Rates,
    disks: Disks,
    disk_io: disk::Sampler,
    storage: Option<(u64, u64)>,
    storage_at: Option<Instant>,
    gpu: gpu::GpuSampler,
    gpu_info: Option<GpuInfo>,
    previous: Option<Instant>,
    history: VecDeque<SystemSample>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            // Do not use new_all: it scans every process and task.
            system: System::new(),
            networks: Networks::new(),
            network_rates: Rates::default(),
            disks: Disks::new(),
            disk_io: disk::Sampler::new(),
            storage: None,
            storage_at: None,
            gpu: gpu::GpuSampler::new(),
            gpu_info: None,
            previous: None,
            history: VecDeque::with_capacity(HISTORY_POINTS),
        }
    }

    /// Resume after a hidden/minimized window without averaging over the pause.
    /// Keep known GPU support so its column does not blink out during warmup.
    pub fn reset(&mut self) {
        let info = self.gpu_info.take().map(|mut info| {
            info.memory_used_bytes = None;
            info.memory_total_bytes = None;
            info
        });
        *self = Self::new();
        self.gpu_info = info;
    }

    /// Called by the existing poll loop. Never sleeps, launches a process, or
    /// requests external data. Elapsed time uses a monotonic clock; wall time is
    /// only for display and history placement.
    pub fn sample(&mut self) -> SystemSnapshot {
        let now = Instant::now();
        let at_ms = timestamp_ms();
        let elapsed = self.previous.map(|previous| now.duration_since(previous));
        let resumed = elapsed.is_some_and(|elapsed| elapsed > MAX_GAP);
        if resumed {
            self.system = System::new();
            self.network_rates = Rates::default();
            self.disk_io = disk::Sampler::new();
            self.gpu = gpu::GpuSampler::new();
            self.history.clear();
            self.storage_at = None;
        }

        self.system.refresh_cpu_usage();
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let cpu_ready = elapsed.is_some_and(|elapsed| {
            elapsed >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL && elapsed <= MAX_GAP
        });
        let cpu_count = self.system.cpus().len();
        let cpu_pct = (cpu_ready && cpu_count > 0)
            .then(|| percentage(self.system.global_cpu_usage() as f64))
            .flatten();
        let memory = capacity(self.system.used_memory(), self.system.total_memory());

        self.networks.refresh(true);
        let counters = self
            .networks
            .iter()
            .filter(|(name, data)| network_is_usable(name, data))
            .map(|(name, data)| {
                (
                    name.clone(),
                    (data.total_received(), data.total_transmitted()),
                )
            })
            .collect();
        let (network_rx_bps, network_tx_bps) = self.network_rates.update(counters, now);
        let (disk_read_bps, disk_write_bps) = self.disk_io.sample(now);

        if self
            .storage_at
            .is_none_or(|last| now.duration_since(last) >= STORAGE_INTERVAL)
        {
            self.disks
                .refresh_specifics(true, DiskRefreshKind::nothing().with_storage());
            self.storage = storage_capacity(&self.disks);
            self.storage_at = Some(now);
        }

        let gpu_reading = self.gpu.sample();
        let gpu_pct = gpu_reading
            .as_ref()
            .and_then(|reading| percentage(reading.usage_percent));
        if let Some(reading) = gpu_reading {
            if self
                .gpu_info
                .as_ref()
                .is_some_and(|old| old.name != reading.name)
            {
                // A different selected adapter must not inherit the old graph.
                for point in &mut self.history {
                    point.gpu_pct = None;
                }
            }
            let memory = reading
                .memory_used_bytes
                .zip(reading.memory_total_bytes)
                .and_then(|(used, total)| capacity(used, total));
            self.gpu_info = Some(GpuInfo {
                name: reading.name,
                memory_used_bytes: memory.map(|(used, _)| used),
                memory_total_bytes: memory.map(|(_, total)| total),
            });
        } else if let Some(info) = &mut self.gpu_info {
            // Preserve device support/name, never stale VRAM as a current value.
            info.memory_used_bytes = None;
            info.memory_total_bytes = None;
        }

        let current = SystemSample {
            at_ms,
            cpu_pct,
            memory_pct: memory
                .and_then(|(used, total)| percentage(used as f64 / total as f64 * 100.0)),
            memory_used_bytes: memory.map(|(used, _)| used),
            memory_total_bytes: memory.map(|(_, total)| total),
            gpu_pct: if resumed { None } else { gpu_pct },
            disk_read_bps,
            disk_write_bps,
            disk_used_bytes: self.storage.map(|(used, _)| used),
            disk_total_bytes: self.storage.map(|(_, total)| total),
            network_rx_bps,
            network_tx_bps,
        };
        push_history(&mut self.history, current.clone());
        self.previous = Some(now);
        SystemSnapshot {
            sampled_at_ms: at_ms,
            interval_ms: INTERVAL.as_millis() as u64,
            cpu_count: (cpu_count > 0).then_some(cpu_count),
            gpu: self.gpu_info.clone(),
            current,
            history: self.history.iter().cloned().collect(),
        }
    }
}

/// Explicit DEMO data, entirely independent of this machine's telemetry.
pub fn demo_snapshot() -> SystemSnapshot {
    const GIB: u64 = 1024 * 1024 * 1024;
    let sampled_at_ms = timestamp_ms();
    let history: Vec<_> = (0..HISTORY_POINTS)
        .map(|index| {
            let phase = index as f64 * 0.6;
            SystemSample {
                at_ms: sampled_at_ms.saturating_sub(
                    (HISTORY_POINTS - index - 1) as u64 * INTERVAL.as_millis() as u64,
                ),
                cpu_pct: Some(18.0 + phase.sin() * 9.0),
                memory_pct: Some(50.0),
                memory_used_bytes: Some(16 * GIB),
                memory_total_bytes: Some(32 * GIB),
                gpu_pct: Some(12.0 + phase.cos() * 8.0),
                disk_read_bps: Some((3.0 + phase.sin() * 2.0) * 1024.0 * 1024.0),
                disk_write_bps: Some((1.5 + phase.cos()) * 1024.0 * 1024.0),
                disk_used_bytes: Some(196 * GIB),
                disk_total_bytes: Some(512 * GIB),
                network_rx_bps: Some((0.6 + phase.sin() * 0.4) * 1024.0 * 1024.0),
                network_tx_bps: Some((40.0 + phase.cos() * 30.0) * 1024.0),
            }
        })
        .collect();
    SystemSnapshot {
        sampled_at_ms,
        interval_ms: INTERVAL.as_millis() as u64,
        cpu_count: Some(16),
        gpu: Some(GpuInfo {
            name: "Demo GPU".to_string(),
            memory_used_bytes: Some(2 * GIB),
            memory_total_bytes: Some(8 * GIB),
        }),
        current: history.last().expect("demo has 30 points").clone(),
        history,
    }
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn percentage(value: f64) -> Option<f64> {
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
}

fn capacity(used: u64, total: u64) -> Option<(u64, u64)> {
    (total > 0 && used <= total).then_some((used, total))
}

fn push_history(history: &mut VecDeque<SystemSample>, sample: SystemSample) {
    if history
        .back()
        .is_some_and(|last| last.at_ms >= sample.at_ms)
    {
        // Wall-clock correction: discard the incompatible time axis.
        history.clear();
    }
    while history
        .front()
        .is_some_and(|first| sample.at_ms.saturating_sub(first.at_ms) >= HISTORY_MS)
        || history.len() >= HISTORY_POINTS
    {
        history.pop_front();
    }
    history.push_back(sample);
}

type Counters = HashMap<String, (u64, u64)>;

/// A baseline for each interface/device prevents a newly connected device's
/// lifetime counter from becoming a traffic spike. A missing/reset counter
/// makes this aggregate unavailable for one sample instead of fabricating zero.
#[derive(Default)]
struct Rates {
    previous: Option<(Instant, Counters)>,
}

impl Rates {
    fn update(&mut self, current: Counters, now: Instant) -> (Option<f64>, Option<f64>) {
        let old = self.previous.replace((now, current));
        let Some((then, old)) = old else {
            return (None, None);
        };
        let current = &self.previous.as_ref().expect("baseline just set").1;
        let elapsed = now.duration_since(then).as_secs_f64();
        let rate = aggregate_rate(&old, current, elapsed);
        (rate.map(|rate| rate.0), rate.map(|rate| rate.1))
    }
}

fn aggregate_rate(old: &Counters, current: &Counters, seconds: f64) -> Option<(f64, f64)> {
    if !seconds.is_finite()
        || seconds <= 0.0
        || seconds > MAX_GAP.as_secs_f64()
        || current.is_empty()
        || old.len() != current.len()
    {
        return None;
    }
    let mut first = 0u64;
    let mut second = 0u64;
    for (key, &(current_first, current_second)) in current {
        let &(old_first, old_second) = old.get(key)?;
        first = first.checked_add(current_first.checked_sub(old_first)?)?;
        second = second.checked_add(current_second.checked_sub(old_second)?)?;
    }
    let rate = (first as f64 / seconds, second as f64 / seconds);
    (rate.0.is_finite() && rate.1.is_finite()).then_some(rate)
}

fn network_is_usable(name: &str, data: &sysinfo::NetworkData) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "lo" || lower == "lo0" || lower.contains("loopback") {
        return false;
    }
    if !data
        .ip_networks()
        .iter()
        .any(|network| !network.addr.is_loopback() && !network.addr.is_unspecified())
    {
        return false;
    }
    #[cfg(target_os = "linux")]
    {
        // `unknown` is valid for VPNs; down/dormant/testing interfaces are not.
        let state = std::fs::read_to_string(
            std::path::Path::new("/sys/class/net")
                .join(name)
                .join("operstate"),
        );
        if let Ok(state) = state {
            return matches!(state.trim(), "up" | "unknown");
        }
    }
    true
}

fn storage_capacity(disks: &Disks) -> Option<(u64, u64)> {
    let mut seen = HashSet::new();
    let mut used = 0u64;
    let mut total = 0u64;
    for disk in disks.list() {
        let Some(key) = disk::volume_key(disk) else {
            continue;
        };
        if !seen.insert(key) {
            continue;
        }
        let disk_total = disk.total_space();
        let disk_used = disk_total.checked_sub(disk.available_space())?;
        if capacity(disk_used, disk_total).is_none() {
            continue;
        }
        used = used.checked_add(disk_used)?;
        total = total.checked_add(disk_total)?;
    }
    capacity(used, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counters(items: &[(&str, u64, u64)]) -> Counters {
        items
            .iter()
            .map(|(key, a, b)| (key.to_string(), (*a, *b)))
            .collect()
    }

    #[test]
    fn rates_use_actual_elapsed_and_valid_idle_is_zero() {
        let old = counters(&[("a", 100, 200), ("b", 200, 300)]);
        let current = counters(&[("a", 350, 700), ("b", 450, 800)]);
        assert_eq!(aggregate_rate(&old, &current, 2.5), Some((200.0, 400.0)));
        assert_eq!(aggregate_rate(&old, &old, 2.0), Some((0.0, 0.0)));
    }

    #[test]
    fn new_interface_and_counter_reset_need_a_new_baseline() {
        let old = counters(&[("a", 100, 200)]);
        assert_eq!(
            aggregate_rate(
                &old,
                &counters(&[("a", 200, 250), ("b", 1_000_000, 5)]),
                2.0
            ),
            None
        );
        assert_eq!(aggregate_rate(&old, &counters(&[("a", 5, 250)]), 2.0), None);
        assert_eq!(aggregate_rate(&old, &Counters::new(), 2.0), None);
        assert_eq!(
            aggregate_rate(&counters(&[("a", 100, 200), ("gone", 5, 5)]), &old, 2.0),
            None
        );
        let now = Instant::now();
        let mut rates = Rates::default();
        assert_eq!(rates.update(old.clone(), now), (None, None));
        assert_eq!(
            rates.update(old.clone(), now + INTERVAL),
            (Some(0.0), Some(0.0))
        );
        assert_eq!(
            rates.update(old.clone(), now + Duration::from_secs(30)),
            (None, None)
        );
        assert_eq!(
            rates.update(old, now + Duration::from_secs(32)),
            (Some(0.0), Some(0.0))
        );
    }

    #[test]
    fn rates_reject_invalid_time_and_overflow() {
        let values = counters(&[("a", 1, 2)]);
        for seconds in [0.0, -1.0, f64::NAN, f64::INFINITY, 10.1] {
            assert_eq!(aggregate_rate(&values, &values, seconds), None);
        }
        assert_eq!(
            aggregate_rate(
                &counters(&[("a", 0, 0), ("b", 0, 0)]),
                &counters(&[("a", u64::MAX, 0), ("b", 1, 0)]),
                2.0
            ),
            None
        );
        assert_eq!(
            aggregate_rate(&counters(&[("a", 0, 0)]), &values, f64::from_bits(1)),
            None
        );
    }

    #[test]
    fn history_is_bounded_in_count_and_time_and_resets_clock_reversal() {
        let mut history = VecDeque::new();
        for index in 0..100 {
            push_history(
                &mut history,
                SystemSample {
                    at_ms: index * 2000,
                    ..SystemSample::default()
                },
            );
        }
        assert_eq!(history.len(), HISTORY_POINTS);
        assert_eq!(history.front().unwrap().at_ms, 140_000);
        push_history(
            &mut history,
            SystemSample {
                at_ms: 300_000,
                ..SystemSample::default()
            },
        );
        assert_eq!(history.len(), 1);
        push_history(
            &mut history,
            SystemSample {
                at_ms: 100_000,
                ..SystemSample::default()
            },
        );
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn invalid_values_remain_missing_in_json() {
        for value in [f64::NAN, f64::INFINITY, -0.1, 100.1] {
            assert_eq!(percentage(value), None);
        }
        assert_eq!(percentage(0.0), Some(0.0));
        assert_eq!(percentage(100.0), Some(100.0));
        assert_eq!(capacity(0, 0), None);
        assert_eq!(capacity(10, 9), None);
        let json = serde_json::to_value(SystemSample::default()).unwrap();
        assert!(json["cpu_pct"].is_null());
        assert!(json["disk_read_bps"].is_null());
    }
}
