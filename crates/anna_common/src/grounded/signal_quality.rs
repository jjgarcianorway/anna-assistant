//! Signal Quality v7.19.0 - Network and Storage Health Signals
//!
//! Sources:
//! - iw dev INTERFACE station dump for WiFi signal
//! - ethtool -S INTERFACE for ethernet stats
//! - smartctl -a /dev/DEVICE for SMART health
//! - nvme smart-log /dev/DEVICE for NVMe health
//!
//! Provides honest signal quality without inference or guessing.

use std::process::Command;

/// WiFi signal quality metrics
#[derive(Debug, Clone, Default)]
pub struct WifiSignal {
    pub interface: String,
    pub ssid: String,
    pub signal_dbm: Option<i32>,
    pub tx_bitrate: Option<String>,
    pub rx_bitrate: Option<String>,
    pub tx_failed: u64,
    pub tx_retries: u64,
    pub beacon_loss: u64,
    pub disconnects: u64,
    pub source: String,
}

/// Ethernet signal quality metrics
#[derive(Debug, Clone, Default)]
pub struct EthernetSignal {
    pub interface: String,
    pub link_detected: bool,
    pub speed: Option<String>,
    pub duplex: Option<String>,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub collisions: u64,
    pub source: String,
}

/// Storage health signal metrics
#[derive(Debug, Clone, Default)]
pub struct StorageSignal {
    pub device: String,
    pub model: String,
    pub smart_status: String,  // PASSED, FAILED, or unknown
    pub temperature_c: Option<u32>,
    pub power_on_hours: Option<u64>,
    pub media_errors: u64,
    pub reallocated_sectors: u64,
    pub pending_sectors: u64,
    pub source: String,
}

/// NVMe specific signal metrics
#[derive(Debug, Clone, Default)]
pub struct NvmeSignal {
    pub device: String,
    pub model: String,
    pub temperature_c: Option<u32>,
    pub percentage_used: Option<u8>,
    pub media_errors: u64,
    pub power_on_hours: Option<u64>,
    pub unsafe_shutdowns: u64,
    pub source: String,
}

/// Signal quality assessment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalHealth {
    Good,
    Warning,
    Critical,
    Unknown,
}

impl SignalHealth {
    pub fn emoji(&self) -> &'static str {
        match self {
            SignalHealth::Good => "🟢",
            SignalHealth::Warning => "🟡",
            SignalHealth::Critical => "🔴",
            SignalHealth::Unknown => "⚪",
        }
    }
}

/// Get WiFi signal quality for an interface
pub fn get_wifi_signal(interface: &str) -> WifiSignal {
    let mut signal = WifiSignal::default();
    signal.interface = interface.to_string();
    signal.source = "iw, /proc/net/wireless".to_string();

    // Get station info from iw
    if let Ok(output) = Command::new("iw")
        .args(["dev", interface, "station", "dump"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_iw_station(&stdout, &mut signal);
        }
    }

    // Get SSID from iw
    if let Ok(output) = Command::new("iw")
        .args(["dev", interface, "link"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("SSID:") {
                    signal.ssid = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    break;
                }
            }
        }
    }

    // Try to get disconnect count from journalctl (last hour)
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "1 hour ago", "-u", "NetworkManager", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            signal.disconnects = stdout
                .lines()
                .filter(|l| l.contains("disconnected") || l.contains("deauthenticated"))
                .count() as u64;
        }
    }

    signal
}

/// Parse iw station dump output
fn parse_iw_station(output: &str, signal: &mut WifiSignal) {
    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("signal:") {
            if let Some(dbm_str) = line.split_whitespace().nth(1) {
                signal.signal_dbm = dbm_str.parse().ok();
            }
        } else if line.starts_with("tx bitrate:") {
            signal.tx_bitrate = line.split(':').nth(1).map(|s| s.trim().to_string());
        } else if line.starts_with("rx bitrate:") {
            signal.rx_bitrate = line.split(':').nth(1).map(|s| s.trim().to_string());
        } else if line.starts_with("tx failed:") {
            signal.tx_failed = line.split(':').nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("tx retries:") {
            signal.tx_retries = line.split(':').nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("beacon loss:") {
            signal.beacon_loss = line.split(':').nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
        }
    }
}

/// Assess WiFi signal health
impl WifiSignal {
    pub fn health(&self) -> SignalHealth {
        // Check signal strength
        if let Some(dbm) = self.signal_dbm {
            if dbm < -80 {
                return SignalHealth::Critical;
            }
            if dbm < -70 {
                return SignalHealth::Warning;
            }
        }

        // Check for high error rates
        if self.tx_failed > 100 || self.beacon_loss > 50 {
            return SignalHealth::Critical;
        }
        if self.tx_failed > 20 || self.beacon_loss > 10 {
            return SignalHealth::Warning;
        }

        // Check disconnects
        if self.disconnects > 3 {
            return SignalHealth::Warning;
        }

        if self.signal_dbm.is_some() {
            SignalHealth::Good
        } else {
            SignalHealth::Unknown
        }
    }

    pub fn signal_bars(&self) -> &'static str {
        match self.signal_dbm {
            Some(dbm) if dbm >= -50 => "▂▄▆█",
            Some(dbm) if dbm >= -60 => "▂▄▆_",
            Some(dbm) if dbm >= -70 => "▂▄__",
            Some(dbm) if dbm >= -80 => "▂___",
            Some(_) => "____",
            None => "????",
        }
    }
}

/// Get Ethernet signal quality for an interface
pub fn get_ethernet_signal(interface: &str) -> EthernetSignal {
    let mut signal = EthernetSignal::default();
    signal.interface = interface.to_string();
    signal.source = "ethtool, /sys/class/net".to_string();

    // Get link status from ethtool
    if let Ok(output) = Command::new("ethtool")
        .arg(interface)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Speed:") {
                    signal.speed = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("Duplex:") {
                    signal.duplex = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("Link detected:") {
                    signal.link_detected = line.contains("yes");
                }
            }
        }
    }

    // Get error stats from /sys
    let sys_path = format!("/sys/class/net/{}/statistics", interface);
    signal.rx_errors = read_sys_stat(&sys_path, "rx_errors");
    signal.tx_errors = read_sys_stat(&sys_path, "tx_errors");
    signal.rx_dropped = read_sys_stat(&sys_path, "rx_dropped");
    signal.tx_dropped = read_sys_stat(&sys_path, "tx_dropped");
    signal.collisions = read_sys_stat(&sys_path, "collisions");

    signal
}

/// Read a statistic from /sys/class/net
fn read_sys_stat(base_path: &str, stat: &str) -> u64 {
    let path = format!("{}/{}", base_path, stat);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Assess Ethernet signal health
impl EthernetSignal {
    pub fn health(&self) -> SignalHealth {
        if !self.link_detected {
            return SignalHealth::Critical;
        }

        let total_errors = self.rx_errors + self.tx_errors;
        let total_dropped = self.rx_dropped + self.tx_dropped;

        if total_errors > 1000 || total_dropped > 500 {
            return SignalHealth::Critical;
        }
        if total_errors > 100 || total_dropped > 50 {
            return SignalHealth::Warning;
        }

        SignalHealth::Good
    }
}

/// Get storage SMART health for a device
pub fn get_storage_signal(device: &str) -> StorageSignal {
    let mut signal = StorageSignal::default();
    signal.device = device.to_string();
    signal.source = "smartctl".to_string();

    // Try smartctl -a
    if let Ok(output) = Command::new("smartctl")
        .args(["-a", device])
        .output()
    {
        if output.status.success() || output.status.code() == Some(4) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_smartctl(&stdout, &mut signal);
        }
    }

    signal
}

/// Parse smartctl output
fn parse_smartctl(output: &str, signal: &mut StorageSignal) {
    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("Device Model:") || line.starts_with("Model Number:") {
            signal.model = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("SMART overall-health") {
            signal.smart_status = if line.contains("PASSED") {
                "PASSED".to_string()
            } else if line.contains("FAILED") {
                "FAILED".to_string()
            } else {
                "unknown".to_string()
            };
        } else if line.contains("Temperature_Celsius") || line.contains("Airflow_Temperature") {
            // SMART attribute format: ID ATTRIBUTE_NAME FLAG VALUE WORST THRESH TYPE UPDATED RAW
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                signal.temperature_c = parts[9].parse().ok();
            }
        } else if line.contains("Power_On_Hours") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                signal.power_on_hours = parts[9].parse().ok();
            }
        } else if line.contains("Reallocated_Sector") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                signal.reallocated_sectors = parts[9].parse().unwrap_or(0);
            }
        } else if line.contains("Current_Pending_Sector") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                signal.pending_sectors = parts[9].parse().unwrap_or(0);
            }
        } else if line.contains("Media_Wearout") || line.contains("Media and Data") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(last) = parts.last() {
                signal.media_errors = last.parse().unwrap_or(0);
            }
        }
    }
}

/// Assess storage signal health
impl StorageSignal {
    pub fn health(&self) -> SignalHealth {
        if self.smart_status == "FAILED" {
            return SignalHealth::Critical;
        }

        if self.reallocated_sectors > 0 || self.pending_sectors > 0 {
            return SignalHealth::Warning;
        }

        if let Some(temp) = self.temperature_c {
            if temp > 60 {
                return SignalHealth::Critical;
            }
            if temp > 50 {
                return SignalHealth::Warning;
            }
        }

        if self.smart_status == "PASSED" {
            SignalHealth::Good
        } else {
            SignalHealth::Unknown
        }
    }
}

/// Get NVMe specific health info
pub fn get_nvme_signal(device: &str) -> NvmeSignal {
    let mut signal = NvmeSignal::default();
    signal.device = device.to_string();
    signal.source = "nvme smart-log".to_string();

    // Try nvme smart-log
    if let Ok(output) = Command::new("nvme")
        .args(["smart-log", device])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_nvme_smart(&stdout, &mut signal);
        }
    }

    // Get model from nvme id-ctrl
    if let Ok(output) = Command::new("nvme")
        .args(["id-ctrl", device, "-o", "normal"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("mn") && line.contains(":") {
                    signal.model = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    break;
                }
            }
        }
    }

    signal
}

/// Parse nvme smart-log output
fn parse_nvme_smart(output: &str, signal: &mut NvmeSignal) {
    for line in output.lines() {
        let line = line.trim().to_lowercase();

        if line.contains("temperature") && !line.contains("warning") {
            if let Some(temp_str) = line.split(':').nth(1) {
                let temp_str = temp_str.trim().replace(" c", "").replace(",", "");
                signal.temperature_c = temp_str.split_whitespace().next()
                    .and_then(|s| s.parse().ok());
            }
        } else if line.contains("percentage_used") || line.contains("percentage used") {
            if let Some(pct_str) = line.split(':').nth(1) {
                let pct_str = pct_str.trim().replace("%", "");
                signal.percentage_used = pct_str.parse().ok();
            }
        } else if line.contains("media_errors") || line.contains("media errors") {
            if let Some(err_str) = line.split(':').nth(1) {
                signal.media_errors = err_str.trim().replace(",", "").parse().unwrap_or(0);
            }
        } else if line.contains("power_on_hours") || line.contains("power on hours") {
            if let Some(hrs_str) = line.split(':').nth(1) {
                signal.power_on_hours = hrs_str.trim().replace(",", "").parse().ok();
            }
        } else if line.contains("unsafe_shutdowns") || line.contains("unsafe shutdowns") {
            if let Some(cnt_str) = line.split(':').nth(1) {
                signal.unsafe_shutdowns = cnt_str.trim().replace(",", "").parse().unwrap_or(0);
            }
        }
    }
}

/// Assess NVMe signal health
impl NvmeSignal {
    pub fn health(&self) -> SignalHealth {
        if self.media_errors > 0 {
            return SignalHealth::Critical;
        }

        if let Some(pct) = self.percentage_used {
            if pct > 90 {
                return SignalHealth::Critical;
            }
            if pct > 70 {
                return SignalHealth::Warning;
            }
        }

        if let Some(temp) = self.temperature_c {
            if temp > 70 {
                return SignalHealth::Critical;
            }
            if temp > 60 {
                return SignalHealth::Warning;
            }
        }

        if self.unsafe_shutdowns > 100 {
            return SignalHealth::Warning;
        }

        SignalHealth::Good
    }
}

/// Get all hot signals (warnings/criticals) for hw overview
pub fn get_hot_signals() -> Vec<String> {
    let mut signals = Vec::new();

    // Check WiFi interfaces
    if let Ok(output) = Command::new("iw")
        .args(["dev"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Interface") {
                    if let Some(iface) = line.split_whitespace().nth(1) {
                        let wifi = get_wifi_signal(iface);
                        match wifi.health() {
                            SignalHealth::Critical => {
                                signals.push(format!(
                                    "{} WiFi {} signal critical {}dBm",
                                    SignalHealth::Critical.emoji(),
                                    iface,
                                    wifi.signal_dbm.unwrap_or(-99)
                                ));
                            }
                            SignalHealth::Warning => {
                                signals.push(format!(
                                    "{} WiFi {} signal weak {}dBm",
                                    SignalHealth::Warning.emoji(),
                                    iface,
                                    wifi.signal_dbm.unwrap_or(-99)
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Check storage devices
    if let Ok(output) = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "disk" {
                    let device = format!("/dev/{}", parts[0]);

                    // Check if NVMe
                    if parts[0].starts_with("nvme") {
                        let nvme = get_nvme_signal(&device);
                        match nvme.health() {
                            SignalHealth::Critical => {
                                signals.push(format!(
                                    "{} NVMe {} health critical",
                                    SignalHealth::Critical.emoji(),
                                    parts[0]
                                ));
                            }
                            SignalHealth::Warning => {
                                let msg = if let Some(pct) = nvme.percentage_used {
                                    format!("{}% used", pct)
                                } else {
                                    "wear warning".to_string()
                                };
                                signals.push(format!(
                                    "{} NVMe {} {}",
                                    SignalHealth::Warning.emoji(),
                                    parts[0],
                                    msg
                                ));
                            }
                            _ => {}
                        }
                    } else {
                        let storage = get_storage_signal(&device);
                        match storage.health() {
                            SignalHealth::Critical => {
                                signals.push(format!(
                                    "{} Storage {} SMART {}",
                                    SignalHealth::Critical.emoji(),
                                    parts[0],
                                    storage.smart_status
                                ));
                            }
                            SignalHealth::Warning => {
                                let msg = if storage.reallocated_sectors > 0 {
                                    format!("{} reallocated sectors", storage.reallocated_sectors)
                                } else if let Some(temp) = storage.temperature_c {
                                    format!("{}°C", temp)
                                } else {
                                    "attention needed".to_string()
                                };
                                signals.push(format!(
                                    "{} Storage {} {}",
                                    SignalHealth::Warning.emoji(),
                                    parts[0],
                                    msg
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    signals.truncate(5);
    signals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_health_emoji() {
        assert_eq!(SignalHealth::Good.emoji(), "🟢");
        assert_eq!(SignalHealth::Warning.emoji(), "🟡");
        assert_eq!(SignalHealth::Critical.emoji(), "🔴");
    }

    #[test]
    fn test_wifi_signal_bars() {
        let mut wifi = WifiSignal::default();
        wifi.signal_dbm = Some(-45);
        assert_eq!(wifi.signal_bars(), "▂▄▆█");

        wifi.signal_dbm = Some(-75);
        assert_eq!(wifi.signal_bars(), "▂___");
    }

    #[test]
    fn test_storage_health_assessment() {
        let mut storage = StorageSignal::default();
        storage.smart_status = "PASSED".to_string();
        assert_eq!(storage.health(), SignalHealth::Good);

        storage.reallocated_sectors = 5;
        assert_eq!(storage.health(), SignalHealth::Warning);

        storage.smart_status = "FAILED".to_string();
        assert_eq!(storage.health(), SignalHealth::Critical);
    }
}
