//! Network Awareness v7.13.0 - Interfaces, Traffic and State
//!
//! Sources:
//! - ip link, ip addr, ip -s link
//! - /sys/class/net
//! - nmcli device status (when available)
//! - rfkill (when available)
//! - lspci, lsusb for hardware identification

use std::fs;
use std::path::Path;
use std::process::Command;

/// Network interface type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Ethernet,
    WiFi,
    Loopback,
    Bridge,
    Virtual,
    Bluetooth,
    Unknown,
}

impl InterfaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterfaceType::Ethernet => "ethernet",
            InterfaceType::WiFi => "wifi",
            InterfaceType::Loopback => "loopback",
            InterfaceType::Bridge => "bridge",
            InterfaceType::Virtual => "virtual",
            InterfaceType::Bluetooth => "bluetooth",
            InterfaceType::Unknown => "unknown",
        }
    }

    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower == "lo" {
            InterfaceType::Loopback
        } else if name_lower.starts_with("wl") || name_lower.starts_with("wlan") {
            InterfaceType::WiFi
        } else if name_lower.starts_with("en") || name_lower.starts_with("eth") {
            InterfaceType::Ethernet
        } else if name_lower.starts_with("br") || name_lower.starts_with("virbr") {
            InterfaceType::Bridge
        } else if name_lower.starts_with("veth") || name_lower.starts_with("docker")
                  || name_lower.starts_with("tun") || name_lower.starts_with("tap") {
            InterfaceType::Virtual
        } else if name_lower.starts_with("bn") || name_lower.contains("bluetooth") {
            InterfaceType::Bluetooth
        } else {
            InterfaceType::Unknown
        }
    }
}

/// Network interface link state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Up,
    Down,
    Unknown,
}

impl LinkState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkState::Up => "up",
            LinkState::Down => "down",
            LinkState::Unknown => "unknown",
        }
    }
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub iface_type: InterfaceType,
    pub driver: Option<String>,
    pub mac: Option<String>,
    pub state: LinkState,
    pub ip_addrs: Vec<String>,
    /// Bytes received since boot
    pub rx_bytes: u64,
    /// Bytes transmitted since boot
    pub tx_bytes: u64,
    /// Hardware model (from lspci/lsusb)
    pub hardware_model: Option<String>,
}

impl NetworkInterface {
    pub fn state_str(&self) -> &str {
        match self.state {
            LinkState::Up => "connected",
            LinkState::Down => "disconnected",
            LinkState::Unknown => "unknown",
        }
    }

    pub fn primary_ip(&self) -> Option<&String> {
        self.ip_addrs.first()
    }
}

/// Network inventory summary
#[derive(Debug, Clone, Default)]
pub struct NetworkSummary {
    pub total_interfaces: usize,
    pub wifi_interfaces: Vec<String>,
    pub ethernet_interfaces: Vec<String>,
    pub wifi_up: usize,
    pub ethernet_up: usize,
}

impl NetworkSummary {
    pub fn format_compact(&self) -> String {
        let mut parts = Vec::new();

        if !self.wifi_interfaces.is_empty() {
            let status = if self.wifi_up > 0 { "up" } else { "down" };
            parts.push(format!("wifi: {} [{}]", self.wifi_interfaces.join(", "), status));
        }

        if !self.ethernet_interfaces.is_empty() {
            let status = if self.ethernet_up > 0 { "up" } else { "down" };
            parts.push(format!("ethernet: {} [{}]", self.ethernet_interfaces.join(", "), status));
        }

        if parts.is_empty() {
            "no physical interfaces".to_string()
        } else {
            format!("{} interfaces ({})", self.total_interfaces, parts.join(", "))
        }
    }
}

/// Get all network interfaces
pub fn get_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    // Read from /sys/class/net
    let net_path = Path::new("/sys/class/net");
    if let Ok(entries) = fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback for general listing
            if name == "lo" {
                continue;
            }

            let iface_type = InterfaceType::from_name(&name);
            let iface_path = entry.path();

            // Get driver
            let driver = fs::read_link(iface_path.join("device/driver"))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

            // Get MAC address
            let mac = fs::read_to_string(iface_path.join("address"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| s != "00:00:00:00:00:00");

            // Get link state
            let state = fs::read_to_string(iface_path.join("operstate"))
                .ok()
                .map(|s| {
                    match s.trim() {
                        "up" => LinkState::Up,
                        "down" => LinkState::Down,
                        _ => LinkState::Unknown,
                    }
                })
                .unwrap_or(LinkState::Unknown);

            // Get RX/TX bytes
            let rx_bytes = fs::read_to_string(iface_path.join("statistics/rx_bytes"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let tx_bytes = fs::read_to_string(iface_path.join("statistics/tx_bytes"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            interfaces.push(NetworkInterface {
                name,
                iface_type,
                driver,
                mac,
                state,
                ip_addrs: Vec::new(),
                rx_bytes,
                tx_bytes,
                hardware_model: None,
            });
        }
    }

    // Enrich with IP addresses from ip addr
    if let Ok(output) = Command::new("ip")
        .args(["-o", "addr", "show"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let iface_name = parts[1].trim_end_matches(':');
                    if parts[2] == "inet" || parts[2] == "inet6" {
                        let addr = parts[3].to_string();
                        if let Some(iface) = interfaces.iter_mut().find(|i| i.name == iface_name) {
                            // Only add IPv4 and non-link-local IPv6
                            if parts[2] == "inet" || !addr.starts_with("fe80:") {
                                if iface.ip_addrs.len() < 3 {
                                    iface.ip_addrs.push(addr);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    interfaces
}

/// Get network summary for status display
pub fn get_network_summary() -> NetworkSummary {
    let interfaces = get_interfaces();

    let wifi: Vec<_> = interfaces.iter()
        .filter(|i| i.iface_type == InterfaceType::WiFi)
        .collect();
    let ethernet: Vec<_> = interfaces.iter()
        .filter(|i| i.iface_type == InterfaceType::Ethernet)
        .collect();

    NetworkSummary {
        total_interfaces: wifi.len() + ethernet.len(),
        wifi_interfaces: wifi.iter().map(|i| i.name.clone()).collect(),
        ethernet_interfaces: ethernet.iter().map(|i| i.name.clone()).collect(),
        wifi_up: wifi.iter().filter(|i| i.state == LinkState::Up).count(),
        ethernet_up: ethernet.iter().filter(|i| i.state == LinkState::Up).count(),
    }
}

/// Get interface by name
pub fn get_interface(name: &str) -> Option<NetworkInterface> {
    get_interfaces().into_iter().find(|i| i.name == name)
}

/// Get interfaces by type (wifi, ethernet, etc)
pub fn get_interfaces_by_type(iface_type: InterfaceType) -> Vec<NetworkInterface> {
    get_interfaces().into_iter()
        .filter(|i| i.iface_type == iface_type)
        .collect()
}

/// Get WiFi specific info using iw (if available)
#[derive(Debug, Clone, Default)]
pub struct WiFiInfo {
    pub ssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub frequency: Option<String>,
    pub bitrate: Option<String>,
}

pub fn get_wifi_info(iface: &str) -> WiFiInfo {
    let mut info = WiFiInfo::default();

    // Try iw dev <iface> link
    if let Ok(output) = Command::new("iw")
        .args(["dev", iface, "link"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("SSID:") {
                    info.ssid = Some(line.trim_start_matches("SSID:").trim().to_string());
                } else if line.starts_with("signal:") {
                    if let Some(dbm) = line.split_whitespace().nth(1) {
                        info.signal_dbm = dbm.parse().ok();
                    }
                } else if line.starts_with("freq:") {
                    info.frequency = Some(line.trim_start_matches("freq:").trim().to_string());
                } else if line.starts_with("tx bitrate:") {
                    info.bitrate = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
            }
        }
    }

    info
}

/// Traffic statistics for an interface
#[derive(Debug, Clone, Default)]
pub struct TrafficStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

pub fn get_traffic_stats(iface: &str) -> TrafficStats {
    let mut stats = TrafficStats::default();
    let base = format!("/sys/class/net/{}/statistics", iface);

    stats.rx_bytes = fs::read_to_string(format!("{}/rx_bytes", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    stats.tx_bytes = fs::read_to_string(format!("{}/tx_bytes", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    stats.rx_packets = fs::read_to_string(format!("{}/rx_packets", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    stats.tx_packets = fs::read_to_string(format!("{}/tx_packets", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    stats.rx_errors = fs::read_to_string(format!("{}/rx_errors", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    stats.tx_errors = fs::read_to_string(format!("{}/tx_errors", base))
        .ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);

    stats
}

/// Format bytes as human readable
pub fn format_traffic(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Bluetooth info (simplified)
#[derive(Debug, Clone)]
pub struct BluetoothInfo {
    pub name: String,
    pub address: String,
    pub powered: bool,
    pub discoverable: bool,
}

pub fn get_bluetooth_info() -> Option<BluetoothInfo> {
    // Try bluetoothctl show
    if let Ok(output) = Command::new("bluetoothctl")
        .args(["show"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut name = String::new();
            let mut address = String::new();
            let mut powered = false;
            let mut discoverable = false;

            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Controller") {
                    address = line.split_whitespace().nth(1).unwrap_or("").to_string();
                } else if line.starts_with("Name:") {
                    name = line.trim_start_matches("Name:").trim().to_string();
                } else if line.starts_with("Powered:") {
                    powered = line.contains("yes");
                } else if line.starts_with("Discoverable:") {
                    discoverable = line.contains("yes");
                }
            }

            if !address.is_empty() {
                return Some(BluetoothInfo {
                    name,
                    address,
                    powered,
                    discoverable,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_type_from_name() {
        assert_eq!(InterfaceType::from_name("lo"), InterfaceType::Loopback);
        assert_eq!(InterfaceType::from_name("wlp0s20f3"), InterfaceType::WiFi);
        assert_eq!(InterfaceType::from_name("enp3s0"), InterfaceType::Ethernet);
        assert_eq!(InterfaceType::from_name("eth0"), InterfaceType::Ethernet);
        assert_eq!(InterfaceType::from_name("wlan0"), InterfaceType::WiFi);
        assert_eq!(InterfaceType::from_name("br0"), InterfaceType::Bridge);
        assert_eq!(InterfaceType::from_name("veth123"), InterfaceType::Virtual);
    }

    #[test]
    fn test_get_interfaces() {
        let interfaces = get_interfaces();
        // Should at least have some interface (even if just virtual)
        // This test is more about not crashing than specific results
        for iface in &interfaces {
            assert!(!iface.name.is_empty());
        }
    }

    #[test]
    fn test_format_traffic() {
        assert_eq!(format_traffic(500), "500 B");
        assert_eq!(format_traffic(1536), "1.5 KiB");
        assert_eq!(format_traffic(1572864), "1.5 MiB");
        assert_eq!(format_traffic(1610612736), "1.5 GiB");
    }

    #[test]
    fn test_network_summary() {
        let summary = get_network_summary();
        // Just ensure it doesn't crash and returns something
        let compact = summary.format_compact();
        assert!(!compact.is_empty());
    }
}
