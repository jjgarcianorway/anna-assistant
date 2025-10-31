// Anna v0.10 Telemetry Collection Module
// Read-only system metrics collection with privacy redaction

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

/// Complete telemetry snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub ts: u64,
    pub host_id: String,
    pub kernel: String,
    pub distro: String,
    pub uptime_s: u64,
    pub cpu: CpuMetrics,
    pub mem: MemMetrics,
    pub disk: Vec<DiskMetrics>,
    pub net: Vec<NetMetrics>,
    pub power: Option<PowerMetrics>,
    pub gpu: Vec<GpuMetrics>,
    pub processes: Vec<ProcessMetrics>,
    pub systemd_units: Vec<SystemdUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub cores: Vec<CpuCore>,
    pub load_avg: [f32; 3],
    pub throttle_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCore {
    pub core: usize,
    pub util_pct: f32,
    pub temp_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemMetrics {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub cached_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub mount: String,
    pub device: String,
    pub fstype: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub pct: f32,
    pub inodes_pct: Option<f32>,
    pub read_iops: u64,
    pub write_iops: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetMetrics {
    pub iface: String,
    pub rx_kbps: f64,
    pub tx_kbps: f64,
    pub link_state: String,
    pub ipv4_redacted: Option<String>,
    pub ipv6_prefix: Option<String>,
    pub mac_hash: String,
    pub rssi_dbm: Option<i32>,
    pub ssid_hash: Option<String>,
    pub vpn_flag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerMetrics {
    pub percent: u8,
    pub status: String, // Charging, Discharging, Full, Unknown
    pub on_ac_bool: bool,
    pub time_to_empty_min: Option<u32>,
    pub time_to_full_min: Option<u32>,
    pub power_now_w: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub device_id: String,
    pub util_pct: Option<f32>,
    pub temp_c: Option<f32>,
    pub mem_used_mb: Option<u64>,
    pub mem_total_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_mb: f64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub unit: String,
    pub load: String,
    pub active: String,
    pub sub: String,
}

/// Privacy redaction utilities
pub mod privacy {
    use sha2::{Digest, Sha256};

    /// Hash a MAC address to 16-char hex string
    pub fn hash_mac(mac: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(mac.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }

    /// Redact last octet of IPv4 address
    pub fn redact_ipv4(ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}.xxx", parts[0], parts[1], parts[2])
        } else {
            "invalid".to_string()
        }
    }

    /// Extract /64 prefix from IPv6 address
    pub fn redact_ipv6(ip: &str) -> String {
        let parts: Vec<&str> = ip.split(':').collect();
        if parts.len() >= 4 {
            format!("{}:{}:{}:{}::/64", parts[0], parts[1], parts[2], parts[3])
        } else {
            "invalid".to_string()
        }
    }

    /// Hash hostname or SSID to 16-char hex
    pub fn hash_string(s: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }
}

/// Main telemetry collector
pub struct TelemetryCollector {
    sys: System,
    last_net_stats: HashMap<String, (u64, u64)>, // iface -> (rx_bytes, tx_bytes)
    last_collection_time: Option<SystemTime>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            last_net_stats: HashMap::new(),
            last_collection_time: None,
        }
    }

    /// Collect a complete telemetry snapshot
    pub fn collect(&mut self) -> Result<TelemetrySnapshot> {
        let now = SystemTime::now();
        let ts = now.duration_since(UNIX_EPOCH)?.as_secs();

        // Refresh system info
        self.sys.refresh_all();

        let snapshot = TelemetrySnapshot {
            ts,
            host_id: self.collect_host_id()?,
            kernel: self.collect_kernel()?,
            distro: self.collect_distro()?,
            uptime_s: System::uptime(),
            cpu: self.collect_cpu()?,
            mem: self.collect_memory()?,
            disk: self.collect_disks()?,
            net: self.collect_network(now)?,
            power: self.collect_power().ok(),
            gpu: self.collect_gpu()?,
            processes: self.collect_processes()?,
            systemd_units: self.collect_systemd_units()?,
        };

        self.last_collection_time = Some(now);
        Ok(snapshot)
    }

    fn collect_host_id(&self) -> Result<String> {
        let hostname = fs::read_to_string("/etc/hostname")
            .or_else(|_| fs::read_to_string("/proc/sys/kernel/hostname"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        Ok(privacy::hash_string(&hostname))
    }

    fn collect_kernel(&self) -> Result<String> {
        Ok(System::kernel_version().unwrap_or_else(|| "unknown".to_string()))
    }

    fn collect_distro(&self) -> Result<String> {
        if let Some(os_release) = System::long_os_version() {
            return Ok(os_release);
        }

        // Fallback: parse /etc/os-release
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return Ok(line.split('=').nth(1).unwrap_or("unknown")
                        .trim_matches('"').to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    fn collect_cpu(&mut self) -> Result<CpuMetrics> {
        self.sys.refresh_cpu();

        let mut cores = Vec::new();
        for (idx, cpu) in self.sys.cpus().iter().enumerate() {
            cores.push(CpuCore {
                core: idx,
                util_pct: cpu.cpu_usage(),
                temp_c: None, // Will be filled from hwmon
            });
        }

        // Collect temperatures from hwmon
        self.collect_cpu_temps(&mut cores)?;

        // Load averages
        let load = System::load_average();
        let load_avg = [load.one as f32, load.five as f32, load.fifteen as f32];

        // Throttle flags (check for thermal throttling)
        let throttle_flags = self.check_throttle_flags()?;

        Ok(CpuMetrics {
            cores,
            load_avg,
            throttle_flags,
        })
    }

    fn collect_cpu_temps(&self, cores: &mut [CpuCore]) -> Result<()> {
        // Read from /sys/class/hwmon/hwmon*/temp*_input
        let hwmon_base = Path::new("/sys/class/hwmon");
        if !hwmon_base.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(hwmon_base)? {
            let entry = entry?;
            let hwmon_path = entry.path();

            // Check if this is a CPU temp sensor
            let name_path = hwmon_path.join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                let name = name.trim();
                if name.contains("coretemp") || name.contains("k10temp") || name.contains("cpu") {
                    // Read temp*_input files
                    for temp_entry in fs::read_dir(&hwmon_path)? {
                        let temp_entry = temp_entry?;
                        let temp_file = temp_entry.file_name();
                        let temp_str = temp_file.to_string_lossy();

                        if temp_str.starts_with("temp") && temp_str.ends_with("_input") {
                            if let Ok(temp_raw) = fs::read_to_string(temp_entry.path()) {
                                if let Ok(temp_millideg) = temp_raw.trim().parse::<i32>() {
                                    let temp_c = temp_millideg as f32 / 1000.0;

                                    // Try to match to a core (heuristic: temp1=core0, etc.)
                                    if let Some(idx_str) = temp_str.strip_prefix("temp").and_then(|s| s.split('_').next()) {
                                        if let Ok(idx) = idx_str.parse::<usize>() {
                                            if idx > 0 && idx <= cores.len() {
                                                cores[idx - 1].temp_c = Some(temp_c);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_throttle_flags(&self) -> Result<Vec<String>> {
        let mut flags = Vec::new();

        // Check /sys/devices/system/cpu/cpu*/thermal_throttle/*
        let cpu_base = Path::new("/sys/devices/system/cpu");
        if !cpu_base.exists() {
            return Ok(flags);
        }

        for entry in fs::read_dir(cpu_base)? {
            let entry = entry?;
            let cpu_path = entry.path();
            let cpu_name = entry.file_name();

            if cpu_name.to_string_lossy().starts_with("cpu") {
                let throttle_path = cpu_path.join("thermal_throttle/core_throttle_count");
                if let Ok(count_str) = fs::read_to_string(&throttle_path) {
                    if let Ok(count) = count_str.trim().parse::<u64>() {
                        if count > 0 {
                            flags.push(format!("{}: {} throttles", cpu_name.to_string_lossy(), count));
                        }
                    }
                }
            }
        }

        Ok(flags)
    }

    fn collect_memory(&self) -> Result<MemMetrics> {
        let total_mb = self.sys.total_memory() / 1024;
        let used_mb = self.sys.used_memory() / 1024;
        let free_mb = self.sys.free_memory() / 1024;
        let available_mb = self.sys.available_memory() / 1024;

        // Cached is approximated as available - free
        let cached_mb = if available_mb > free_mb {
            available_mb - free_mb
        } else {
            0
        };

        let swap_total_mb = self.sys.total_swap() / 1024;
        let swap_used_mb = self.sys.used_swap() / 1024;

        Ok(MemMetrics {
            total_mb,
            used_mb,
            free_mb,
            cached_mb,
            swap_total_mb,
            swap_used_mb,
        })
    }

    fn collect_disks(&self) -> Result<Vec<DiskMetrics>> {
        use sysinfo::Disks;
        let mut disks = Vec::new();
        let sys_disks = Disks::new_with_refreshed_list();

        for disk in &sys_disks {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);

            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let pct = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
            } else {
                0.0
            };

            let mount = disk.mount_point().to_string_lossy().to_string();
            let device = disk.name().to_string_lossy().to_string();
            let fstype = format!("{:?}", disk.file_system()).trim_matches('"').to_string();

            // Read IO stats from /sys/block if available
            let (read_iops, write_iops) = self.read_disk_iostats(&device).unwrap_or((0, 0));

            // Inodes percentage (requires statfs, skip for now)
            let inodes_pct = None;

            disks.push(DiskMetrics {
                mount,
                device,
                fstype,
                total_gb,
                used_gb,
                pct,
                inodes_pct,
                read_iops,
                write_iops,
            });
        }

        Ok(disks)
    }

    fn read_disk_iostats(&self, _device: &str) -> Result<(u64, u64)> {
        // Parse /proc/diskstats for the device
        // Format: major minor name reads ... writes ...
        // For simplicity in v0.10, return zeros (can be enhanced later)
        Ok((0, 0))
    }

    fn collect_network(&mut self, now: SystemTime) -> Result<Vec<NetMetrics>> {
        use sysinfo::Networks;
        let mut nets = Vec::new();
        let elapsed_secs = if let Some(last_time) = self.last_collection_time {
            now.duration_since(last_time)?.as_secs_f64()
        } else {
            1.0 // First collection, assume 1 second
        };

        let networks = Networks::new_with_refreshed_list();
        for (iface_name, data) in &networks {
            let rx_bytes = data.total_received();
            let tx_bytes = data.total_transmitted();

            // Calculate rates
            let (rx_kbps, tx_kbps) = if let Some((last_rx, last_tx)) = self.last_net_stats.get(iface_name.as_str()) {
                let rx_delta = rx_bytes.saturating_sub(*last_rx) as f64;
                let tx_delta = tx_bytes.saturating_sub(*last_tx) as f64;
                (
                    (rx_delta / elapsed_secs) / 1024.0,
                    (tx_delta / elapsed_secs) / 1024.0,
                )
            } else {
                (0.0, 0.0)
            };

            // Update last stats
            self.last_net_stats.insert(iface_name.to_string(), (rx_bytes, tx_bytes));

            // Read additional info from /sys/class/net/<iface>/
            let (link_state, ipv4, ipv6, mac, vpn_flag) = self.read_net_sysfs(iface_name)?;

            nets.push(NetMetrics {
                iface: iface_name.clone(),
                rx_kbps,
                tx_kbps,
                link_state,
                ipv4_redacted: ipv4.map(|ip| privacy::redact_ipv4(&ip)),
                ipv6_prefix: ipv6.map(|ip| privacy::redact_ipv6(&ip)),
                mac_hash: privacy::hash_mac(&mac),
                rssi_dbm: None, // WiFi RSSI requires iwconfig/iw parsing
                ssid_hash: None,
                vpn_flag,
            });
        }

        Ok(nets)
    }

    fn read_net_sysfs(&self, iface: &str) -> Result<(String, Option<String>, Option<String>, String, bool)> {
        let sysfs_path = Path::new("/sys/class/net").join(iface);

        // Link state
        let link_state = fs::read_to_string(sysfs_path.join("operstate"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        // MAC address
        let mac = fs::read_to_string(sysfs_path.join("address"))
            .unwrap_or_else(|_| "00:00:00:00:00:00".to_string())
            .trim()
            .to_string();

        // IPs (requires parsing `ip addr show`)
        let (ipv4, ipv6) = self.parse_ip_addr(iface)?;

        // VPN flag: check if iface name matches common patterns
        let vpn_flag = iface.starts_with("tun") || iface.starts_with("wg") || iface.starts_with("vpn");

        Ok((link_state, ipv4, ipv6, mac, vpn_flag))
    }

    fn parse_ip_addr(&self, iface: &str) -> Result<(Option<String>, Option<String>)> {
        // Use `ip addr show <iface>` to get IP addresses
        // For simplicity in MVP, skip this (requires shell-out or netlink parsing)
        // Return None for both
        let _ = iface;
        Ok((None, None))
    }

    fn collect_power(&self) -> Result<PowerMetrics> {
        let power_supply_path = Path::new("/sys/class/power_supply");
        if !power_supply_path.exists() {
            anyhow::bail!("No power supply found");
        }

        // Find BAT* device
        for entry in fs::read_dir(power_supply_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("BAT") {
                let bat_path = entry.path();

                let capacity = fs::read_to_string(bat_path.join("capacity"))?
                    .trim()
                    .parse::<u8>()?;

                let status = fs::read_to_string(bat_path.join("status"))?
                    .trim()
                    .to_string();

                // Check AC adapter
                let on_ac = self.check_ac_adapter()?;

                return Ok(PowerMetrics {
                    percent: capacity,
                    status,
                    on_ac_bool: on_ac,
                    time_to_empty_min: None, // Requires calculation
                    time_to_full_min: None,
                    power_now_w: self.read_power_now(&bat_path).ok(),
                });
            }
        }

        anyhow::bail!("No battery found")
    }

    fn check_ac_adapter(&self) -> Result<bool> {
        let power_supply_path = Path::new("/sys/class/power_supply");

        for entry in fs::read_dir(power_supply_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("AC") || name_str.starts_with("ADP") {
                let online = fs::read_to_string(entry.path().join("online"))?
                    .trim()
                    .parse::<u8>()?;

                return Ok(online == 1);
            }
        }

        Ok(false)
    }

    fn read_power_now(&self, bat_path: &Path) -> Result<f32> {
        let power_now = fs::read_to_string(bat_path.join("power_now"))?
            .trim()
            .parse::<u64>()?;

        // Convert microwatts to watts
        Ok(power_now as f32 / 1_000_000.0)
    }

    fn collect_gpu(&self) -> Result<Vec<GpuMetrics>> {
        let mut gpus = Vec::new();

        // Read from /sys/class/drm/card*/device/
        let drm_path = Path::new("/sys/class/drm");
        if !drm_path.exists() {
            return Ok(gpus);
        }

        for entry in fs::read_dir(drm_path)? {
            let entry = entry?;
            let card_name = entry.file_name();
            let card_str = card_name.to_string_lossy();

            if card_str.starts_with("card") && !card_str.contains('-') {
                let device_path = entry.path().join("device");

                // Try to read GPU temp (AMD)
                let temp_c = fs::read_to_string(device_path.join("hwmon/hwmon*/temp1_input"))
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .map(|t| t as f32 / 1000.0);

                gpus.push(GpuMetrics {
                    device_id: card_str.to_string(),
                    util_pct: None, // Requires vendor-specific APIs
                    temp_c,
                    mem_used_mb: None,
                    mem_total_mb: None,
                });
            }
        }

        Ok(gpus)
    }

    fn collect_processes(&self) -> Result<Vec<ProcessMetrics>> {
        let mut processes: Vec<ProcessMetrics> = self.sys.processes()
            .values()
            .map(|proc| ProcessMetrics {
                pid: proc.pid().as_u32(),
                name: proc.name().to_string(),
                cpu_pct: proc.cpu_usage(),
                mem_mb: proc.memory() as f64 / 1024.0,
                state: format!("{:?}", proc.status()),
            })
            .collect();

        // Sort by CPU usage and take top 50
        processes.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());
        processes.truncate(50);

        Ok(processes)
    }

    fn collect_systemd_units(&self) -> Result<Vec<SystemdUnit>> {
        // Parse `systemctl list-units --no-pager --plain` output
        // For MVP, skip this (requires shell-out or D-Bus parsing)
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_redact_ipv4() {
        assert_eq!(privacy::redact_ipv4("192.168.1.42"), "192.168.1.xxx");
        assert_eq!(privacy::redact_ipv4("10.0.0.1"), "10.0.0.xxx");
    }

    #[test]
    fn test_privacy_redact_ipv6() {
        let result = privacy::redact_ipv6("2001:db8::1");
        assert!(result.starts_with("2001:db8:"));
        assert!(result.ends_with("::/64"));
    }

    #[test]
    fn test_privacy_hash_mac() {
        let hash = privacy::hash_mac("aa:bb:cc:dd:ee:ff");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_telemetry_collection() -> Result<()> {
        let mut collector = TelemetryCollector::new();
        let snapshot = collector.collect()?;

        assert!(snapshot.ts > 0);
        assert!(!snapshot.host_id.is_empty());
        assert!(!snapshot.cpu.cores.is_empty());
        assert!(snapshot.mem.total_mb > 0);

        Ok(())
    }
}
