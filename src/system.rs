use crate::imports::*;

pub struct System {
    pub system_id: Option<u64>,
    pub cpu_physical_cores: Option<usize>,
    pub cpu_frequency: Option<u64>,
    pub cpu_brand: Option<String>,
    pub total_memory: u64,
    pub long_os_version: Option<String>,
    pub disk_usage: Option<DiskUsage>,
}

impl System {
    pub fn ram_as_gb(&self) -> u64 {
        self.total_memory / 1024 / 1024 / 1024
    }
}

impl AsRef<System> for System {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl From<&System> for Vec<Content> {
    fn from(system: &System) -> Self {
        let mut rows = Vec::new();
        if let Some(long_os_version) = &system.long_os_version {
            rows.push(Content::field("OS:", long_os_version.clone()));
            rows.push(Content::space());
        }
        if let Some(cpu_physical_cores) = system.cpu_physical_cores {
            let mut info = format!("{cpu_physical_cores} Cores");
            if let Some(cpu_frequency) = system.cpu_frequency {
                info = format!("{info} @{:1.2} GHz", cpu_frequency as f64 / 1000.0);
            }
            rows.push(Content::field("CPU:", info));
        }
        if let Some(cpu_brand) = &system.cpu_brand {
            rows.push(Content::field("", cpu_brand.clone()));
        }
        rows.push(Content::field(
            "RAM:",
            as_gb(system.total_memory as f64, false, false),
        ));
        if let Some(disk_usage) = &system.disk_usage {
            let total = as_gb(disk_usage.total as f64, false, false);
            let used = as_gb(disk_usage.used as f64, false, false);
            let available = as_gb(disk_usage.available as f64, false, false);

            let max = [&total, &used, &available]
                .iter()
                .map(|v| v.len())
                .max()
                .unwrap_or(0);

            rows.push(Content::separator());
            rows.push(Content::field(
                "Storage Total:",
                total.pad_to_width_with_alignment(max, Alignment::Right),
            ));
            rows.push(Content::field(
                "Used:",
                used.pad_to_width_with_alignment(max, Alignment::Right),
            ));
            rows.push(Content::field(
                "Available:",
                format!(
                    "{} ({:1.0}%)",
                    available.pad_to_width_with_alignment(max, Alignment::Right),
                    disk_usage.capacity() * 100.0
                ),
            ));
        }
        rows
    }
}

impl Default for System {
    fn default() -> Self {
        use sysinfo::*;
        let mut system = System::new();
        system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency());
        system.refresh_memory();
        // system.refresh_processes();
        let cpus = system.cpus();
        let cpu_physical_core_count = system.physical_core_count();
        let long_os_version = System::long_os_version();
        let total_memory = system.total_memory();

        let (cpu_frequency, cpu_brand) = cpus
            .first()
            .map(|cpu| (cpu.frequency(), cpu.brand().to_string()))
            .unzip();

        let disk_usage = disk_usage();
        let system_id = Self::try_system_id_as_u64();
        // ---

        // for (pid, process) in system.processes() {
        //     println!("{} {}", pid, process.exe().map(|p| p.display().to_string()).unwrap_or_default());
        // }

        Self {
            system_id,
            cpu_physical_cores: cpu_physical_core_count,
            cpu_frequency,
            cpu_brand,
            total_memory,
            long_os_version,
            disk_usage,
        }
    }
}

impl System {
    fn try_system_id_as_u64() -> Option<u64> {
        Self::try_system_id().and_then(|v| v[0..8].try_into().ok().map(u64::from_be_bytes))
    }
    fn try_system_id() -> Option<Vec<u8>> {
        let some_id = if let Ok(mut file) = std::fs::File::open("/etc/machine-id") {
            // fetch the system id from /etc/machine-id
            let mut machine_id = String::new();
            file.read_to_string(&mut machine_id).ok();
            machine_id.trim().to_string()
        } else if let Ok(Some(mac)) = mac_address::get_mac_address() {
            // fallback on the mac address
            mac.to_string().trim().to_string()
        } else {
            // 🤷
            return None;
        };
        let mut sha256 = Sha256::default();
        sha256.update(some_id.as_bytes());
        Some(sha256.finalize().to_vec())
    }

    pub fn system_id_as_hex(&self) -> String {
        format!("{:016x}", self.system_id.unwrap_or_default())
    }
}

#[derive(Debug)]
pub struct DiskUsage {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

impl DiskUsage {
    pub fn capacity(&self) -> f64 {
        self.available as f64 / self.total as f64
    }
}
pub fn disk_usage() -> Option<DiskUsage> {
    let lines = cmd!("df", "-k", home_folder()).read().ok()?;
    let lines = lines.lines();
    if let Some(line) = lines.last() {
        let mut parts = line.split_whitespace();
        let total = parts.nth(1).and_then(|v| u64::from_str(v).ok())? * 1024;
        let used = parts.next().and_then(|v| u64::from_str(v).ok())? * 1024;
        let available = parts.next().and_then(|v| u64::from_str(v).ok())? * 1024;
        Some(DiskUsage {
            total,
            used,
            available,
        })
    } else {
        None
    }
}
