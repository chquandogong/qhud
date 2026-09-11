//! Optional native GPU telemetry. No subprocess is launched by the sampler.
//!
//! Windows uses the WDDM GPU Engine performance counters. Linux uses the
//! amdgpu sysfs API and, when installed, NVIDIA's NVML driver library. A
//! missing/failed measurement is `None`; only a valid idle reading is zero.

#[derive(Clone, Debug)]
pub struct GpuReading {
    pub name: String,
    pub usage_percent: f64,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

pub struct GpuSampler {
    inner: platform::Sampler,
    // Select the busiest supported adapter on the first successful read, then
    // retain it so a single graph never combines different adapters.
    selected: Option<String>,
}

impl GpuSampler {
    pub fn new() -> Self {
        Self {
            inner: platform::Sampler::new(),
            selected: None,
        }
    }

    pub fn sample(&mut self) -> Option<GpuReading> {
        select_reading(&mut self.selected, self.inner.sample())
    }
}

fn select_reading(selected: &mut Option<String>, readings: Vec<GpuReading>) -> Option<GpuReading> {
    let mut readings = readings
        .into_iter()
        .filter(|reading| valid_percent(reading.usage_percent).is_some());
    if let Some(name) = selected {
        return readings.find(|reading| &reading.name == name);
    }
    // Stable order resolves idle/tied devices deterministically.
    let mut busiest: Option<GpuReading> = None;
    for reading in readings {
        if busiest
            .as_ref()
            .is_none_or(|current| reading.usage_percent > current.usage_percent)
        {
            busiest = Some(reading);
        }
    }
    if let Some(reading) = &busiest {
        *selected = Some(reading.name.clone());
    }
    busiest
}

fn valid_percent(value: f64) -> Option<f64> {
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
}

// Keep these pure helpers available on Linux too, where CI exercises the
// Windows instance grouping without requiring a GPU or a performance provider.
#[cfg(any(target_os = "windows", test))]
fn engine_key(instance: &str) -> Option<(String, u32)> {
    // pid_42_luid_0x00000000_0x0000abcd_phys_0_eng_1_engtype_Copy
    let parts: Vec<_> = instance.split('_').collect();
    if parts.len() < 11
        || parts[0] != "pid"
        || parts[2] != "luid"
        || parts[5] != "phys"
        || parts[7] != "eng"
        || parts[9] != "engtype"
    {
        return None;
    }
    parts[1].parse::<u32>().ok()?;
    let high = u32::from_str_radix(parts[3].strip_prefix("0x")?, 16).ok()?;
    let low = u32::from_str_radix(parts[4].strip_prefix("0x")?, 16).ok()?;
    let physical = parts[6].parse::<u32>().ok()?;
    let engine = parts[8].parse::<u32>().ok()?;
    Some((format!("{high:08x}:{low:08x}/{physical}"), engine))
}

#[cfg(any(target_os = "windows", test))]
fn engine_readings<'a>(instances: impl IntoIterator<Item = (&'a str, f64)>) -> Vec<GpuReading> {
    use std::collections::BTreeMap;
    let mut engines = BTreeMap::<(String, u32), f64>::new();
    for (name, value) in instances {
        if let (Some(key), Some(value)) = (engine_key(name), valid_percent(value)) {
            *engines.entry(key).or_default() += value;
        }
    }
    let mut adapters = BTreeMap::<String, f64>::new();
    for ((adapter, _engine), value) in engines {
        // Multiple processes share an engine; independent engines run in
        // parallel. Match Task Manager's busiest-engine convention, not a sum
        // across engines (which would overstate GPU utilization).
        let total = adapters.entry(adapter).or_default();
        *total = total.max(value.min(100.0));
    }
    adapters
        .into_iter()
        .map(|(adapter, usage_percent)| GpuReading {
            name: format!("GPU adapter {adapter}"),
            usage_percent,
            memory_used_bytes: None,
            memory_total_bytes: None,
        })
        .collect()
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{GpuReading, engine_readings};
    use std::{mem::size_of, ptr};
    use windows_sys::Win32::System::Performance::{
        PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
        PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA, PdhAddEnglishCounterW, PdhCloseQuery,
        PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
    };

    pub struct Sampler {
        query: Option<Query>,
    }

    struct Query {
        handle: PDH_HQUERY,
        counter: PDH_HCOUNTER,
        warmed: bool,
    }

    // PDH queries have no thread affinity. This owns the handle exclusively;
    // sample requires &mut self, so queries cannot race with collection/drop.
    unsafe impl Send for Query {}

    impl Sampler {
        pub fn new() -> Self {
            Self {
                query: Query::new(),
            }
        }

        pub fn sample(&mut self) -> Vec<GpuReading> {
            self.query.as_mut().map(Query::sample).unwrap_or_default()
        }
    }

    impl Query {
        fn new() -> Option<Self> {
            let mut handle = PDH_HQUERY::default();
            // SAFETY: null requests a live local query; handle is an out value.
            if unsafe { PdhOpenQueryW(ptr::null(), 0, &mut handle) } != 0 {
                return None;
            }
            let mut query = Self {
                handle,
                counter: PDH_HCOUNTER::default(),
                warmed: false,
            };
            // English-counter registration is language-neutral, including on
            // Korean Windows. The array API expands the wildcard at sampling.
            let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0"
                .encode_utf16()
                .collect();
            // SAFETY: path is NUL-terminated, query is live, counter is an out value.
            if unsafe { PdhAddEnglishCounterW(handle, path.as_ptr(), 0, &mut query.counter) } != 0 {
                return None; // Query::drop also closes the failed registration.
            }
            Some(query)
        }

        fn sample(&mut self) -> Vec<GpuReading> {
            // SAFETY: this owns a live query, accessed exclusively.
            if unsafe { PdhCollectQueryData(self.handle) } != 0 {
                self.warmed = false;
                return Vec::new();
            }
            if !self.warmed {
                self.warmed = true;
                return Vec::new(); // Rate counters need two separate samples.
            }
            self.read_array().unwrap_or_default()
        }

        fn read_array(&self) -> Option<Vec<GpuReading>> {
            // The process-instance set may grow between the size/read calls.
            // Retry with a fresh sizing call; PDH says the short-buffer size
            // returned by the second call cannot be trusted for reallocation.
            for _ in 0..3 {
                let mut bytes = 0;
                let mut count = 0;
                // SAFETY: null buffer and zero bytes ask for required capacity.
                let status = unsafe {
                    PdhGetFormattedCounterArrayW(
                        self.counter,
                        PDH_FMT_DOUBLE,
                        &mut bytes,
                        &mut count,
                        ptr::null_mut(),
                    )
                };
                if status != PDH_MORE_DATA || bytes == 0 || bytes > 16 * 1024 * 1024 {
                    return None;
                }
                // u64 storage guarantees the native structure's 8-byte
                // alignment, unlike casting an arbitrary Vec<u8> allocation.
                let mut storage = vec![0_u64; (bytes as usize).div_ceil(size_of::<u64>())];
                let allocated = storage.len() * size_of::<u64>();
                let buffer = storage.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
                // SAFETY: buffer has at least bytes bytes with proper alignment.
                let status = unsafe {
                    PdhGetFormattedCounterArrayW(
                        self.counter,
                        PDH_FMT_DOUBLE,
                        &mut bytes,
                        &mut count,
                        buffer,
                    )
                };
                if status == PDH_MORE_DATA {
                    continue;
                }
                if status != 0
                    || bytes as usize > allocated
                    || count as usize > bytes as usize / size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>()
                {
                    return None;
                }
                // SAFETY: success initializes count items, bounds checked above.
                let items = unsafe { std::slice::from_raw_parts(buffer, count as usize) };
                let mut instances = Vec::with_capacity(items.len());
                for item in items {
                    if item.FmtValue.CStatus != PDH_CSTATUS_VALID_DATA
                        && item.FmtValue.CStatus != PDH_CSTATUS_NEW_DATA
                    {
                        continue;
                    }
                    let start = storage.as_ptr() as usize;
                    let end = start + bytes as usize;
                    let name = item.szName as usize;
                    if name < start || name >= end || name % size_of::<u16>() != 0 {
                        continue;
                    }
                    let capacity = ((end - name) / size_of::<u16>()).min(1024);
                    // SAFETY: name is inside PDH's initialized allocation and
                    // aligned; length is bounded by that allocation.
                    let chars = unsafe { std::slice::from_raw_parts(item.szName, capacity) };
                    let Some(length) = chars.iter().position(|c| *c == 0) else {
                        continue;
                    };
                    let name = String::from_utf16_lossy(&chars[..length]);
                    // SAFETY: PDH_FMT_DOUBLE selected this union member.
                    let value = unsafe { item.FmtValue.Anonymous.doubleValue };
                    instances.push((name, value));
                }
                return Some(engine_readings(
                    instances
                        .iter()
                        .map(|(name, value)| (name.as_str(), *value)),
                ));
            }
            None
        }
    }

    impl Drop for Query {
        fn drop(&mut self) {
            // SAFETY: handle came from a successful open and is closed once.
            // Closing the query closes every counter owned by it as well.
            unsafe { PdhCloseQuery(self.handle) };
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{GpuReading, valid_percent};
    use std::{fs, path::PathBuf};

    pub struct Sampler {
        amd_devices: Vec<(String, PathBuf)>,
        nvidia: Option<nvml::Nvml>,
    }

    impl Sampler {
        pub fn new() -> Self {
            let mut amd_devices = Vec::new();
            if let Ok(entries) = fs::read_dir("/sys/class/drm") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if !is_card(&name) {
                        continue;
                    }
                    let device = entry.path().join("device");
                    if device.join("gpu_busy_percent").is_file() {
                        amd_devices.push((format!("AMD GPU ({name})"), device));
                    }
                }
            }
            amd_devices.sort_by(|a, b| a.0.cmp(&b.0));
            Self {
                amd_devices,
                nvidia: nvml::Nvml::new(),
            }
        }

        pub fn sample(&mut self) -> Vec<GpuReading> {
            let mut readings = Vec::new();
            for (name, device) in &self.amd_devices {
                let Some(usage_percent) = fs::read_to_string(device.join("gpu_busy_percent"))
                    .ok()
                    .and_then(|value| parse_percent(&value))
                else {
                    continue;
                };
                let memory = |file: &str| {
                    fs::read_to_string(device.join(file))
                        .ok()
                        .and_then(|value| value.trim().parse::<u64>().ok())
                };
                readings.push(GpuReading {
                    name: name.clone(),
                    usage_percent,
                    memory_used_bytes: memory("mem_info_vram_used"),
                    memory_total_bytes: memory("mem_info_vram_total").filter(|total| *total > 0),
                });
            }
            if let Some(nvidia) = &self.nvidia {
                readings.extend(nvidia.sample());
            }
            readings
        }
    }

    fn is_card(name: &str) -> bool {
        name.strip_prefix("card")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|c| c.is_ascii_digit()))
    }

    fn parse_percent(value: &str) -> Option<f64> {
        value.trim().parse::<f64>().ok().and_then(valid_percent)
    }

    mod nvml {
        use super::GpuReading;
        use libloading::Library;
        use std::ffi::{c_char, c_uint, c_void};

        // The small stable C ABI used here is documented by NVIDIA's NVML
        // Device Queries and Initialization/Cleanup reference. Loading is
        // optional, so the application runs without the proprietary driver.
        type Device = *mut c_void;
        type Status = c_uint;
        type Init = unsafe extern "C" fn() -> Status;
        type Shutdown = unsafe extern "C" fn() -> Status;
        type Count = unsafe extern "C" fn(*mut c_uint) -> Status;
        type Handle = unsafe extern "C" fn(c_uint, *mut Device) -> Status;
        type Name = unsafe extern "C" fn(Device, *mut c_char, c_uint) -> Status;
        type Usage = unsafe extern "C" fn(Device, *mut Utilization) -> Status;
        type Memory = unsafe extern "C" fn(Device, *mut MemoryInfo) -> Status;

        #[repr(C)]
        #[derive(Default)]
        struct Utilization {
            gpu: c_uint,
            memory: c_uint,
        }

        #[repr(C)]
        #[derive(Default)]
        struct MemoryInfo {
            total: u64,
            free: u64,
            used: u64,
        }

        pub struct Nvml {
            // Keep the library alive until after Drop calls nvmlShutdown.
            _library: Library,
            shutdown: Shutdown,
            usage: Usage,
            memory: Option<Memory>,
            devices: Vec<(String, Device)>,
        }

        // NVIDIA documents NVML as thread-safe. Device handles are opaque
        // driver values, and this sampler is only accessed exclusively.
        unsafe impl Send for Nvml {}

        impl Nvml {
            pub fn new() -> Option<Self> {
                // SAFETY: fixed system driver soname; every function is bound
                // to its documented ABI, and the library is retained in Self.
                unsafe {
                    let library = Library::new("libnvidia-ml.so.1").ok()?;
                    let init = *library.get::<Init>(b"nvmlInit_v2\0").ok()?;
                    let shutdown = *library.get::<Shutdown>(b"nvmlShutdown\0").ok()?;
                    let count = *library.get::<Count>(b"nvmlDeviceGetCount_v2\0").ok()?;
                    let handle = *library
                        .get::<Handle>(b"nvmlDeviceGetHandleByIndex_v2\0")
                        .ok()?;
                    let name = *library.get::<Name>(b"nvmlDeviceGetName\0").ok()?;
                    let usage = *library
                        .get::<Usage>(b"nvmlDeviceGetUtilizationRates\0")
                        .ok()?;
                    let memory = library
                        .get::<Memory>(b"nvmlDeviceGetMemoryInfo\0")
                        .ok()
                        .map(|f| *f);
                    if init() != 0 {
                        return None;
                    }
                    // Construct immediately so any subsequent failure runs
                    // nvmlShutdown exactly once before unloading the library.
                    let mut nvml = Self {
                        _library: library,
                        shutdown,
                        usage,
                        memory,
                        devices: Vec::new(),
                    };
                    let mut device_count = 0;
                    if count(&mut device_count) != 0 {
                        return None;
                    }
                    for index in 0..device_count.min(64) {
                        let mut device = std::ptr::null_mut();
                        if handle(index, &mut device) != 0 || device.is_null() {
                            continue;
                        }
                        let mut buffer = [0_u8; 256];
                        let label = if name(
                            device,
                            buffer.as_mut_ptr().cast(),
                            buffer.len() as c_uint,
                        ) == 0
                        {
                            let end = buffer
                                .iter()
                                .position(|value| *value == 0)
                                .unwrap_or(buffer.len());
                            String::from_utf8_lossy(&buffer[..end]).trim().to_owned()
                        } else {
                            "NVIDIA GPU".to_owned()
                        };
                        // Index distinguishes two devices with the same model.
                        nvml.devices
                            .push((format!("{label} (GPU {index})"), device));
                    }
                    (!nvml.devices.is_empty()).then_some(nvml)
                }
            }

            pub fn sample(&self) -> Vec<GpuReading> {
                let mut readings = Vec::new();
                for (name, device) in &self.devices {
                    let mut usage = Utilization::default();
                    // SAFETY: live NVML handle and output buffer of the ABI type.
                    if unsafe { (self.usage)(*device, &mut usage) } != 0 || usage.gpu > 100 {
                        continue;
                    }
                    let mut memory = MemoryInfo::default();
                    let has_memory = self.memory.is_some_and(|read| {
                        // SAFETY: the optional symbol has the documented v1 ABI.
                        (unsafe { read(*device, &mut memory) == 0 })
                            && memory.total > 0
                            && memory.used <= memory.total
                    });
                    readings.push(GpuReading {
                        name: name.clone(),
                        usage_percent: usage.gpu as f64,
                        memory_used_bytes: has_memory.then_some(memory.used),
                        memory_total_bytes: has_memory.then_some(memory.total),
                    });
                }
                readings
            }
        }

        impl Drop for Nvml {
            fn drop(&mut self) {
                // SAFETY: this instance called init successfully; its library
                // and all symbols remain alive until after this method returns.
                unsafe { (self.shutdown)() };
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn only_drm_cards_are_probed_not_connector_or_render_nodes() {
            assert!(is_card("card0"));
            assert!(is_card("card12"));
            for invalid in ["card", "card0-DP-1", "renderD128", "card-1"] {
                assert!(!is_card(invalid));
            }
        }

        #[test]
        fn sysfs_idle_is_valid_and_errors_are_missing() {
            assert_eq!(parse_percent("0\n"), Some(0.0));
            assert_eq!(parse_percent("100\n"), Some(100.0));
            for invalid in ["", "unavailable", "NaN", "-1", "101"] {
                assert_eq!(parse_percent(invalid), None);
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
    use super::GpuReading;

    pub struct Sampler;

    impl Sampler {
        pub fn new() -> Self {
            Self
        }

        pub fn sample(&mut self) -> Vec<GpuReading> {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instance(pid: u32, adapter: u32, physical: u32, engine: u32, kind: &str) -> String {
        format!(
            "pid_{pid}_luid_0x00000000_0x{adapter:08x}_phys_{physical}_eng_{engine}_engtype_{kind}"
        )
    }

    fn reading(name: &str, usage_percent: f64) -> GpuReading {
        GpuReading {
            name: name.to_owned(),
            usage_percent,
            memory_used_bytes: None,
            memory_total_bytes: None,
        }
    }

    #[test]
    fn process_load_sums_within_engine_but_independent_engines_use_max() {
        let names = [
            instance(1, 10, 0, 0, "3D"),
            instance(2, 10, 0, 0, "3D"),
            instance(3, 10, 0, 1, "Copy"),
        ];
        let readings = engine_readings([
            (names[0].as_str(), 25.0),
            (names[1].as_str(), 35.0),
            (names[2].as_str(), 50.0),
        ]);
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].usage_percent, 60.0);
    }

    #[test]
    fn physical_adapters_are_distinct_and_engine_totals_are_capped() {
        let names = [
            instance(1, 10, 0, 0, "3D"),
            instance(2, 10, 0, 0, "3D"),
            instance(3, 10, 1, 0, "3D"),
            instance(4, 11, 0, 0, "3D"),
        ];
        let readings = engine_readings([
            (names[0].as_str(), 70.0),
            (names[1].as_str(), 60.0),
            (names[2].as_str(), 20.0),
            (names[3].as_str(), 30.0),
        ]);
        assert_eq!(readings.len(), 3);
        assert_eq!(readings[0].usage_percent, 100.0);
        assert_eq!(readings[1].usage_percent, 20.0);
        assert_eq!(readings[2].usage_percent, 30.0);
    }

    #[test]
    fn invalid_instances_and_nonfinite_values_do_not_fake_idle() {
        let name = instance(1, 10, 0, 0, "3D");
        assert!(
            engine_readings([
                ("_Total", 50.0),
                ("pid_bad_luid_0x0_0x1_phys_0_eng_0_engtype_3D", 0.0),
                (name.as_str(), f64::NAN),
                (name.as_str(), -1.0),
                (name.as_str(), f64::INFINITY),
            ])
            .is_empty()
        );
        assert_eq!(
            engine_readings([(name.as_str(), 0.0)])[0].usage_percent,
            0.0
        );
    }

    #[test]
    fn gpu_selection_is_sticky_across_load_changes_and_missing_samples() {
        let mut selected = None;
        let first =
            select_reading(&mut selected, vec![reading("A", 20.0), reading("B", 70.0)]).unwrap();
        assert_eq!(first.name, "B");
        let next =
            select_reading(&mut selected, vec![reading("A", 90.0), reading("B", 0.0)]).unwrap();
        assert_eq!(next.name, "B");
        assert_eq!(next.usage_percent, 0.0);
        assert!(select_reading(&mut selected, vec![reading("A", 80.0)]).is_none());
        assert_eq!(selected.as_deref(), Some("B"));
    }

    #[test]
    fn unsupported_or_invalid_devices_are_never_selected() {
        let mut selected = None;
        assert!(select_reading(&mut selected, Vec::new()).is_none());
        assert!(select_reading(&mut selected, vec![reading("A", f64::NAN)]).is_none());
        assert!(selected.is_none());
        assert_eq!(
            select_reading(&mut selected, vec![reading("A", 0.0)])
                .unwrap()
                .usage_percent,
            0.0
        );
    }
}
