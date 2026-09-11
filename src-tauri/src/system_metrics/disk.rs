//! Physical-device I/O rather than a sum of mounted partitions. sysinfo owns
//! capacity discovery, but does not expose whether its I/O query succeeded.

#[cfg(target_os = "windows")]
mod platform {
    use std::time::Instant;

    type Handle = isize;
    #[repr(C)]
    #[derive(Default)]
    struct CounterValue {
        status: u32,
        value: f64,
    }

    #[link(name = "pdh")]
    unsafe extern "system" {
        fn PdhOpenQueryW(source: *const u16, user: usize, query: *mut Handle) -> u32;
        fn PdhAddEnglishCounterW(
            query: Handle,
            path: *const u16,
            user: usize,
            counter: *mut Handle,
        ) -> u32;
        fn PdhCollectQueryData(query: Handle) -> u32;
        fn PdhGetFormattedCounterValue(
            counter: Handle,
            format: u32,
            kind: *mut u32,
            value: *mut CounterValue,
        ) -> u32;
        fn PdhCloseQuery(query: Handle) -> u32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetVolumeNameForVolumeMountPointW(path: *const u16, volume: *mut u16, size: u32) -> i32;
        fn GetDriveTypeW(root: *const u16) -> u32;
    }

    pub struct Sampler {
        query: Handle,
        read: Handle,
        write: Handle,
        ready: bool,
    }

    impl Sampler {
        pub fn new() -> Self {
            let mut sampler = Self {
                query: 0,
                read: 0,
                write: 0,
                ready: false,
            };
            // Query handles remain owned by this object on the polling thread.
            let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut sampler.query) } == 0;
            if !opened {
                sampler.query = 0;
                return sampler;
            }
            let read = wide(r"\PhysicalDisk(_Total)\Disk Read Bytes/sec");
            let write = wide(r"\PhysicalDisk(_Total)\Disk Write Bytes/sec");
            // English counter paths work on localized Windows too.
            let added = unsafe {
                PdhAddEnglishCounterW(sampler.query, read.as_ptr(), 0, &mut sampler.read) == 0
                    && PdhAddEnglishCounterW(sampler.query, write.as_ptr(), 0, &mut sampler.write)
                        == 0
            };
            if !added {
                unsafe {
                    PdhCloseQuery(sampler.query);
                }
                sampler.query = 0;
            }
            sampler
        }

        pub fn sample(&mut self, _now: Instant) -> (Option<f64>, Option<f64>) {
            if self.query == 0 {
                return (None, None);
            }
            // PDH computes rates using its actual collection timestamps.
            if unsafe { PdhCollectQueryData(self.query) } != 0 {
                self.ready = false;
                return (None, None);
            }
            if !std::mem::replace(&mut self.ready, true) {
                return (None, None);
            }
            (value(self.read), value(self.write))
        }
    }

    impl Drop for Sampler {
        fn drop(&mut self) {
            if self.query != 0 {
                unsafe {
                    PdhCloseQuery(self.query);
                }
            }
        }
    }

    fn value(counter: Handle) -> Option<f64> {
        let mut output = CounterValue::default();
        // PDH_FMT_DOUBLE | PDH_FMT_NOCAP100: rates are bytes/sec, not percent.
        let status = unsafe {
            PdhGetFormattedCounterValue(counter, 0x200 | 0x8000, std::ptr::null_mut(), &mut output)
        };
        (status == 0 && output.status <= 1 && output.value.is_finite() && output.value >= 0.0)
            .then_some(output.value)
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn volume_key(disk: &sysinfo::Disk) -> Option<String> {
        use std::os::windows::ffi::OsStrExt;
        let mount: Vec<u16> = disk
            .mount_point()
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // Local removable/fixed volumes only, never mapped network shares.
        if !matches!(unsafe { GetDriveTypeW(mount.as_ptr()) }, 2 | 3) {
            return None;
        }
        let mut volume = [0u16; 128];
        if unsafe {
            GetVolumeNameForVolumeMountPointW(
                mount.as_ptr(),
                volume.as_mut_ptr(),
                volume.len() as u32,
            )
        } == 0
        {
            return None;
        }
        let len = volume.iter().position(|value| *value == 0)?;
        Some(String::from_utf16_lossy(&volume[..len]).to_ascii_lowercase())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::super::{Counters, Rates};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    pub struct Sampler {
        devices: Vec<(String, PathBuf)>,
        discovered_at: Option<Instant>,
        rates: Rates,
    }

    impl Sampler {
        pub fn new() -> Self {
            Self {
                devices: Vec::new(),
                discovered_at: None,
                rates: Rates::default(),
            }
        }

        pub fn sample(&mut self, now: Instant) -> (Option<f64>, Option<f64>) {
            if self
                .discovered_at
                .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(30))
            {
                self.devices = std::fs::read_dir("/sys/block")
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    // /sys/block holds whole disks; device excludes loop, ram,
                    // zram, dm and md layers whose underlying I/O is counted.
                    .filter(|entry| entry.path().join("device").exists())
                    .map(|entry| {
                        (
                            entry.file_name().to_string_lossy().into_owned(),
                            entry.path().join("stat"),
                        )
                    })
                    .collect();
                self.discovered_at = Some(now);
            }
            let mut counters = Counters::new();
            for (name, path) in &self.devices {
                let Some(value) = std::fs::read_to_string(path)
                    .ok()
                    .and_then(|text| parse_stat(&text))
                else {
                    // A failed device is unavailable, not an idle/partial sum.
                    self.rates = Rates::default();
                    self.discovered_at = None;
                    return (None, None);
                };
                counters.insert(name.clone(), value);
            }
            self.rates.update(counters, now)
        }
    }

    pub fn parse_stat(text: &str) -> Option<(u64, u64)> {
        let mut fields = text.split_whitespace();
        let read = fields.nth(2)?.parse::<u64>().ok()?.checked_mul(512)?;
        let write = fields.nth(3)?.parse::<u64>().ok()?.checked_mul(512)?;
        Some((read, write))
    }

    pub fn volume_key(disk: &sysinfo::Disk) -> Option<String> {
        use std::os::unix::fs::MetadataExt;
        let source = disk.name().to_string_lossy();
        // A single filesystem can appear at several bind/subvolume mounts.
        // Device identity deduplicates them while keeping separate partitions.
        let source = Path::new(source.split('[').next()?);
        let id = std::fs::metadata(source)
            .ok()
            .map(|device| device.rdev())
            .filter(|id| *id != 0)
            // Some systems report /dev/root without creating that alias. Only
            // device paths may use this fallback: remote FUSE mounts also have
            // a nonzero mount.dev(), but are not local storage capacity.
            .or_else(|| {
                if !may_use_mount_device(source) {
                    return None;
                }
                std::fs::metadata(disk.mount_point())
                    .ok()
                    .map(|mount| mount.dev())
            })?;
        (id != 0).then(|| format!("{id}:{}", disk.file_system().to_string_lossy()))
    }

    fn may_use_mount_device(source: &Path) -> bool {
        source != Path::new("/dev")
            && source.starts_with("/dev")
            && !source
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn block_stat_sector_counters_use_fixed_512_byte_units() {
            assert_eq!(
                parse_stat("1 2 300 4 5 6 700 8 9 10 11"),
                Some((153_600, 358_400))
            );
            assert_eq!(parse_stat("1 2 x 4 5 6 7"), None);
            assert_eq!(parse_stat("1 2 3"), None);
            assert_eq!(parse_stat("1 2 18446744073709551615 4 5 6 7"), None);
        }

        #[test]
        fn mount_device_fallback_accepts_device_aliases_but_not_remote_sources() {
            for source in [
                "/dev/root",
                "/dev/disk/by-uuid/local-root",
                "/dev/mapper/root",
            ] {
                assert!(may_use_mount_device(Path::new(source)), "{source}");
            }
            for source in [
                "user@host:/data",
                "remote:bucket",
                "pool/dataset",
                "/device/root",
                "/dev",
                "/dev/../mnt/remote",
                "dev/root",
            ] {
                assert!(!may_use_mount_device(Path::new(source)), "{source}");
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
    pub struct Sampler;
    impl Sampler {
        pub fn new() -> Self {
            Self
        }
        pub fn sample(&mut self, _now: std::time::Instant) -> (Option<f64>, Option<f64>) {
            (None, None)
        }
    }
    pub fn volume_key(_disk: &sysinfo::Disk) -> Option<String> {
        None
    }
}

pub use platform::{Sampler, volume_key};
