//! Network configuration detection
//!
//! Detects network configuration:
//! - DNS resolver configuration
//! - Network manager (NetworkManager vs systemd-networkd)
//! - Wi-Fi power save settings

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Network manager type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkManager {
    /// NetworkManager
    NetworkManager,
    /// systemd-networkd
    SystemdNetworkd,
    /// Both are installed
    Both,
    /// Neither is active
    None,
    /// Unknown
    Unknown,
}

/// DNS resolver type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnsResolver {
    /// systemd-resolved
    SystemdResolved,
    /// dnsmasq
    Dnsmasq,
    /// Static /etc/resolv.conf
    Static,
    /// Unknown
    Unknown,
}

/// Network configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Active network manager
    pub network_manager: NetworkManager,
    /// DNS resolver type
    pub dns_resolver: DnsResolver,
    /// DNS servers
    pub dns_servers: Vec<String>,
    /// Wi-Fi power save status (if Wi-Fi is present)
    pub wifi_power_save: Option<bool>,
    /// Wi-Fi interface names
    pub wifi_interfaces: Vec<String>,
}

impl NetworkConfig {
    /// Detect network configuration
    pub fn detect() -> Self {
        let network_manager = detect_network_manager();
        let dns_resolver = detect_dns_resolver();
        let dns_servers = detect_dns_servers();
        let wifi_interfaces = detect_wifi_interfaces();
        let wifi_power_save = if !wifi_interfaces.is_empty() {
            detect_wifi_power_save(&wifi_interfaces[0])
        } else {
            None
        };

        Self {
            network_manager,
            dns_resolver,
            dns_servers,
            wifi_power_save,
            wifi_interfaces,
        }
    }
}

/// Detect active network manager
fn detect_network_manager() -> NetworkManager {
    let nm_active = is_networkmanager_active();
    let systemd_networkd_active = is_systemd_networkd_active();

    match (nm_active, systemd_networkd_active) {
        (true, true) => NetworkManager::Both,
        (true, false) => NetworkManager::NetworkManager,
        (false, true) => NetworkManager::SystemdNetworkd,
        (false, false) => NetworkManager::None,
    }
}

/// Check if NetworkManager is active
fn is_networkmanager_active() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg("NetworkManager")
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return status == "active";
        }
    }
    false
}

/// Check if systemd-networkd is active
fn is_systemd_networkd_active() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg("systemd-networkd")
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return status == "active";
        }
    }
    false
}

/// Detect DNS resolver type
fn detect_dns_resolver() -> DnsResolver {
    // Check if systemd-resolved is active
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg("systemd-resolved")
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if status == "active" {
                return DnsResolver::SystemdResolved;
            }
        }
    }

    // Check if dnsmasq is running
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("dnsmasq").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return DnsResolver::Dnsmasq;
        }
    }

    // Check if /etc/resolv.conf is a symlink to systemd-resolved
    if let Ok(link) = fs::read_link("/etc/resolv.conf") {
        let link_str = link.to_string_lossy();
        if link_str.contains("systemd/resolve") {
            return DnsResolver::SystemdResolved;
        }
    }

    // Otherwise, it's static
    DnsResolver::Static
}

/// Detect DNS servers
fn detect_dns_servers() -> Vec<String> {
    let mut dns_servers = Vec::new();

    // Try systemd-resolved first
    if let Ok(output) = Command::new("resolvectl").arg("status").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("DNS Servers:") {
                    if let Some(servers) = line.split(':').nth(1) {
                        for server in servers.split_whitespace() {
                            dns_servers.push(server.to_string());
                        }
                    }
                } else if line.trim().starts_with("Current DNS Server:") {
                    if let Some(server) = line.split(':').nth(1) {
                        let server = server.trim().to_string();
                        if !dns_servers.contains(&server) {
                            dns_servers.push(server);
                        }
                    }
                }
            }
        }
    }

    // Fallback: parse /etc/resolv.conf
    if dns_servers.is_empty() {
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("nameserver") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        dns_servers.push(server.to_string());
                    }
                }
            }
        }
    }

    dns_servers
}

/// Detect Wi-Fi interfaces
fn detect_wifi_interfaces() -> Vec<String> {
    let mut wifi_interfaces = Vec::new();

    // Use iw to list wireless devices
    if let Ok(output) = Command::new("iw").arg("dev").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("Interface") {
                    if let Some(iface) = line.split_whitespace().nth(1) {
                        wifi_interfaces.push(iface.to_string());
                    }
                }
            }
        }
    }

    // Fallback: check /sys/class/net for wireless devices
    if wifi_interfaces.is_empty() {
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let iface_name = entry.file_name().to_string_lossy().to_string();
                let wireless_path = format!("/sys/class/net/{}/wireless", iface_name);
                if fs::metadata(&wireless_path).is_ok() {
                    wifi_interfaces.push(iface_name);
                }
            }
        }
    }

    wifi_interfaces
}

/// Detect Wi-Fi power save setting for an interface
fn detect_wifi_power_save(interface: &str) -> Option<bool> {
    // Use iw to check power save
    if let Ok(output) = Command::new("iw")
        .arg("dev")
        .arg(interface)
        .arg("get")
        .arg("power_save")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Power save: on") {
                return Some(true);
            } else if stdout.contains("Power save: off") {
                return Some(false);
            }
        }
    }

    None
}

impl std::fmt::Display for NetworkManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkManager::NetworkManager => write!(f, "NetworkManager"),
            NetworkManager::SystemdNetworkd => write!(f, "systemd-networkd"),
            NetworkManager::Both => write!(f, "Both (NetworkManager + systemd-networkd)"),
            NetworkManager::None => write!(f, "None"),
            NetworkManager::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for DnsResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsResolver::SystemdResolved => write!(f, "systemd-resolved"),
            DnsResolver::Dnsmasq => write!(f, "dnsmasq"),
            DnsResolver::Static => write!(f, "Static /etc/resolv.conf"),
            DnsResolver::Unknown => write!(f, "Unknown"),
        }
    }
}
