//! Network Trends v7.32.0 - WiFi Signal & Link Quality Trends
//!
//! Collects and tracks network metrics over time windows:
//! - WiFi: RSSI, link speed, retries, drops, disconnects
//! - Ethernet: link state, speed, duplex, errors
//!
//! Uses local tools only (iw, nmcli, ethtool) - no remote API calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network interface type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    WiFi,
    Ethernet,
    Virtual,
    Unknown,
}

impl InterfaceType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WiFi => "WiFi",
            Self::Ethernet => "Ethernet",
            Self::Virtual => "Virtual",
            Self::Unknown => "Unknown",
        }
    }
}

/// WiFi signal sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiSample {
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    /// Interface name
    pub interface: String,
    /// RSSI in dBm (negative values)
    pub rssi_dbm: Option<i32>,
    /// Link quality as percentage (0-100)
    pub link_quality_percent: Option<u8>,
    /// TX bitrate in Mbps
    pub tx_bitrate_mbps: Option<f32>,
    /// RX bitrate in Mbps
    pub rx_bitrate_mbps: Option<f32>,
    /// TX retries count
    pub tx_retries: Option<u64>,
    /// TX failed count
    pub tx_failed: Option<u64>,
    /// Connected SSID
    pub ssid: Option<String>,
    /// Connection state
    pub connected: bool,
}

/// Ethernet sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetSample {
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    /// Interface name
    pub interface: String,
    /// Link state (up/down)
    pub link_up: bool,
    /// Speed in Mbps
    pub speed_mbps: Option<u32>,
    /// Duplex mode
    pub duplex: Option<String>,
    /// RX errors
    pub rx_errors: Option<u64>,
    /// TX errors
    pub tx_errors: Option<u64>,
    /// RX dropped
    pub rx_dropped: Option<u64>,
    /// TX dropped
    pub tx_dropped: Option<u64>,
}

/// Network trend window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTrendWindow {
    /// Window label (1h, 24h, 7d, 30d)
    pub label: String,
    /// Number of samples in window
    pub sample_count: usize,
    /// Average RSSI (WiFi only)
    pub avg_rssi_dbm: Option<f32>,
    /// Min RSSI (WiFi only)
    pub min_rssi_dbm: Option<i32>,
    /// Max RSSI (WiFi only)
    pub max_rssi_dbm: Option<i32>,
    /// Average link quality (WiFi only)
    pub avg_link_quality: Option<f32>,
    /// Average TX bitrate (WiFi only)
    pub avg_tx_bitrate: Option<f32>,
    /// Total TX retries (WiFi only)
    pub total_tx_retries: Option<u64>,
    /// Total TX failures (WiFi only)
    pub total_tx_failed: Option<u64>,
    /// Disconnect count
    pub disconnect_count: u32,
    /// Uptime percentage (0.0-1.0)
    pub uptime_fraction: f32,
}

/// Network trends for an interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceTrends {
    /// Interface name
    pub interface: String,
    /// Interface type
    pub iface_type: InterfaceType,
    /// Current state
    pub current_connected: bool,
    /// Current SSID (WiFi only)
    pub current_ssid: Option<String>,
    /// Trends per window
    pub windows: HashMap<String, NetworkTrendWindow>,
}

/// Collect current WiFi sample using iw
pub fn collect_wifi_sample(interface: &str) -> Option<WiFiSample> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    // Try iw first
    if let Some(sample) = collect_wifi_sample_iw(interface, timestamp) {
        return Some(sample);
    }

    // Fallback to nmcli
    collect_wifi_sample_nmcli(interface, timestamp)
}

/// Collect WiFi sample using iw dev <iface> station dump
fn collect_wifi_sample_iw(interface: &str, timestamp: u64) -> Option<WiFiSample> {
    let output = std::process::Command::new("iw")
        .args(["dev", interface, "station", "dump"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sample = WiFiSample {
        timestamp,
        interface: interface.to_string(),
        rssi_dbm: None,
        link_quality_percent: None,
        tx_bitrate_mbps: None,
        rx_bitrate_mbps: None,
        tx_retries: None,
        tx_failed: None,
        ssid: None,
        connected: false,
    };

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("signal:") {
            // signal: -52 dBm
            if let Some(val) = trimmed.split_whitespace().nth(1) {
                sample.rssi_dbm = val.parse().ok();
                sample.connected = true;
            }
        } else if trimmed.starts_with("tx bitrate:") {
            // tx bitrate: 866.7 MBit/s
            if let Some(val) = trimmed.split_whitespace().nth(2) {
                sample.tx_bitrate_mbps = val.parse().ok();
            }
        } else if trimmed.starts_with("rx bitrate:") {
            if let Some(val) = trimmed.split_whitespace().nth(2) {
                sample.rx_bitrate_mbps = val.parse().ok();
            }
        } else if trimmed.starts_with("tx retries:") {
            if let Some(val) = trimmed.split_whitespace().nth(2) {
                sample.tx_retries = val.parse().ok();
            }
        } else if trimmed.starts_with("tx failed:") {
            if let Some(val) = trimmed.split_whitespace().nth(2) {
                sample.tx_failed = val.parse().ok();
            }
        }
    }

    // Get SSID
    let ssid_output = std::process::Command::new("iw")
        .args(["dev", interface, "info"])
        .output()
        .ok()?;

    if ssid_output.status.success() {
        let ssid_stdout = String::from_utf8_lossy(&ssid_output.stdout);
        for line in ssid_stdout.lines() {
            if line.trim().starts_with("ssid ") {
                sample.ssid = Some(line.trim().strip_prefix("ssid ")?.to_string());
                break;
            }
        }
    }

    // Convert RSSI to link quality (rough approximation)
    // -50 dBm or better = 100%, -100 dBm = 0%
    if let Some(rssi) = sample.rssi_dbm {
        let quality = ((rssi + 100) as f32 * 2.0).clamp(0.0, 100.0);
        sample.link_quality_percent = Some(quality as u8);
    }

    if sample.connected {
        Some(sample)
    } else {
        None
    }
}

/// Collect WiFi sample using nmcli
fn collect_wifi_sample_nmcli(interface: &str, timestamp: u64) -> Option<WiFiSample> {
    let output = std::process::Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,STATE,CONNECTION,SIGNAL", "device", "wifi", "list", "ifname", interface])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 && parts[0] == interface {
            let signal: Option<u8> = parts[3].parse().ok();

            return Some(WiFiSample {
                timestamp,
                interface: interface.to_string(),
                rssi_dbm: signal.map(|s| {
                    // Convert signal percentage to approximate dBm
                    // 100% ≈ -50 dBm, 0% ≈ -100 dBm
                    -100 + (s as i32 / 2)
                }),
                link_quality_percent: signal,
                tx_bitrate_mbps: None,
                rx_bitrate_mbps: None,
                tx_retries: None,
                tx_failed: None,
                ssid: if parts.len() > 2 { Some(parts[2].to_string()) } else { None },
                connected: parts[1] == "connected",
            });
        }
    }

    None
}

/// Collect current Ethernet sample using ethtool
pub fn collect_ethernet_sample(interface: &str) -> Option<EthernetSample> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    // Check link state from /sys
    let carrier_path = format!("/sys/class/net/{}/carrier", interface);
    let link_up = std::fs::read_to_string(&carrier_path)
        .map(|s| s.trim() == "1")
        .unwrap_or(false);

    let mut sample = EthernetSample {
        timestamp,
        interface: interface.to_string(),
        link_up,
        speed_mbps: None,
        duplex: None,
        rx_errors: None,
        tx_errors: None,
        rx_dropped: None,
        tx_dropped: None,
    };

    // Get speed from /sys
    let speed_path = format!("/sys/class/net/{}/speed", interface);
    if let Ok(content) = std::fs::read_to_string(&speed_path) {
        sample.speed_mbps = content.trim().parse().ok();
    }

    // Get duplex from /sys
    let duplex_path = format!("/sys/class/net/{}/duplex", interface);
    if let Ok(content) = std::fs::read_to_string(&duplex_path) {
        sample.duplex = Some(content.trim().to_string());
    }

    // Get statistics from /sys/class/net/<iface>/statistics
    let stats_dir = format!("/sys/class/net/{}/statistics", interface);
    sample.rx_errors = read_sys_stat(&stats_dir, "rx_errors");
    sample.tx_errors = read_sys_stat(&stats_dir, "tx_errors");
    sample.rx_dropped = read_sys_stat(&stats_dir, "rx_dropped");
    sample.tx_dropped = read_sys_stat(&stats_dir, "tx_dropped");

    // Try ethtool for additional info
    if let Ok(output) = std::process::Command::new("ethtool")
        .arg(interface)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Speed:") {
                    // Speed: 1000Mb/s
                    if let Some(speed_str) = trimmed.split_whitespace().nth(1) {
                        if let Some(num) = speed_str.strip_suffix("Mb/s") {
                            sample.speed_mbps = num.parse().ok();
                        }
                    }
                } else if trimmed.starts_with("Duplex:") {
                    if let Some(duplex) = trimmed.split_whitespace().nth(1) {
                        sample.duplex = Some(duplex.to_lowercase());
                    }
                }
            }
        }
    }

    Some(sample)
}

fn read_sys_stat(stats_dir: &str, name: &str) -> Option<u64> {
    let path = format!("{}/{}", stats_dir, name);
    std::fs::read_to_string(&path)
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Detect network interface type
pub fn detect_interface_type(interface: &str) -> InterfaceType {
    // Check if wireless
    let wireless_path = format!("/sys/class/net/{}/wireless", interface);
    if std::path::Path::new(&wireless_path).exists() {
        return InterfaceType::WiFi;
    }

    // Check if virtual
    let virtual_path = format!("/sys/devices/virtual/net/{}", interface);
    if std::path::Path::new(&virtual_path).exists() {
        return InterfaceType::Virtual;
    }

    // Check if physical ethernet
    let device_path = format!("/sys/class/net/{}/device", interface);
    if std::path::Path::new(&device_path).exists() {
        return InterfaceType::Ethernet;
    }

    InterfaceType::Unknown
}

/// Get all network interfaces
pub fn list_network_interfaces() -> Vec<(String, InterfaceType)> {
    let mut interfaces = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                // Skip loopback
                if name == "lo" {
                    continue;
                }
                let iface_type = detect_interface_type(name);
                interfaces.push((name.to_string(), iface_type));
            }
        }
    }

    interfaces.sort_by_key(|(name, _)| name.clone());
    interfaces
}

/// Check if iw tool is available
pub fn is_iw_available() -> bool {
    std::process::Command::new("which")
        .arg("iw")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if ethtool is available
pub fn is_ethtool_available() -> bool {
    std::process::Command::new("which")
        .arg("ethtool")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if nmcli is available
pub fn is_nmcli_available() -> bool {
    std::process::Command::new("which")
        .arg("nmcli")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Format RSSI for display with signal strength indicator
pub fn format_rssi(rssi: i32) -> String {
    let strength = if rssi >= -50 {
        "excellent"
    } else if rssi >= -60 {
        "good"
    } else if rssi >= -70 {
        "fair"
    } else if rssi >= -80 {
        "weak"
    } else {
        "poor"
    };
    format!("{} dBm ({})", rssi, strength)
}

/// Format link quality percentage with trend
pub fn format_link_quality(quality: u8) -> String {
    let indicator = if quality >= 80 {
        "█████"
    } else if quality >= 60 {
        "████░"
    } else if quality >= 40 {
        "███░░"
    } else if quality >= 20 {
        "██░░░"
    } else {
        "█░░░░"
    };
    format!("{}% {}", quality, indicator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rssi() {
        assert!(format_rssi(-45).contains("excellent"));
        assert!(format_rssi(-55).contains("good"));
        assert!(format_rssi(-65).contains("fair"));
        assert!(format_rssi(-75).contains("weak"));
        assert!(format_rssi(-85).contains("poor"));
    }

    #[test]
    fn test_interface_type_label() {
        assert_eq!(InterfaceType::WiFi.label(), "WiFi");
        assert_eq!(InterfaceType::Ethernet.label(), "Ethernet");
    }

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_network_interfaces();
        // Should not contain loopback
        assert!(!interfaces.iter().any(|(name, _)| name == "lo"));
    }
}
