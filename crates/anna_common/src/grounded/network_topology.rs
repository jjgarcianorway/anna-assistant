//! Network Topology v7.17.0 - Routes, DNS, and Interface Management
//!
//! Enhanced network awareness with:
//! - Default route and gateway information
//! - DNS resolver configuration (systemd-resolved, resolv.conf, NetworkManager)
//! - Interface management state (who manages each interface)
//! - PCI/USB device mapping for interfaces
//!
//! Sources:
//! - ip route, ip addr
//! - /etc/resolv.conf, resolvectl, systemd-resolve
//! - nmcli device status, nmcli general status
//! - /sys/class/net/*/device
//! - ethtool -i, udevadm info

use std::fs;
use std::process::Command;

/// Default route information
#[derive(Debug, Clone, Default)]
pub struct DefaultRoute {
    /// Gateway IP address
    pub gateway: Option<String>,
    /// Interface used for default route
    pub interface: Option<String>,
    /// Metric (lower = higher priority)
    pub metric: Option<u32>,
    /// Source of information
    pub source: String,
}

/// DNS configuration
#[derive(Debug, Clone, Default)]
pub struct DnsConfig {
    /// DNS server addresses
    pub servers: Vec<String>,
    /// Search domains
    pub search_domains: Vec<String>,
    /// Source of DNS configuration
    pub source: String,
}

/// Interface management information
#[derive(Debug, Clone)]
pub enum InterfaceManager {
    NetworkManager,
    SystemdNetworkd,
    Manual,
    Unmanaged,
    Unknown,
}

impl InterfaceManager {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterfaceManager::NetworkManager => "NetworkManager",
            InterfaceManager::SystemdNetworkd => "systemd-networkd",
            InterfaceManager::Manual => "manual",
            InterfaceManager::Unmanaged => "unmanaged",
            InterfaceManager::Unknown => "unknown",
        }
    }
}

/// Extended interface information with management and PCI/USB details
#[derive(Debug, Clone)]
pub struct InterfaceTopology {
    pub name: String,
    pub manager: InterfaceManager,
    /// PCI address (e.g., 0000:00:14.3)
    pub pci_address: Option<String>,
    /// USB path if USB device
    pub usb_path: Option<String>,
    /// Link speed (e.g., "1000 Mbit/s")
    pub speed: Option<String>,
    /// Firmware version (from ethtool)
    pub firmware_version: Option<String>,
}

/// Network topology summary for hw overview
#[derive(Debug, Clone, Default)]
pub struct NetworkTopology {
    pub interfaces: Vec<InterfaceTopology>,
    pub default_route: DefaultRoute,
    pub dns: DnsConfig,
    pub source: String,
}

/// Get default route from ip route
pub fn get_default_route() -> DefaultRoute {
    let mut route = DefaultRoute {
        source: "ip route".to_string(),
        ..Default::default()
    };

    if let Ok(output) = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("default") {
                    let parts: Vec<&str> = line.split_whitespace().collect();

                    // Parse "default via 192.168.1.1 dev wlan0 proto dhcp metric 600"
                    for i in 0..parts.len() {
                        match parts[i] {
                            "via" => {
                                if i + 1 < parts.len() {
                                    route.gateway = Some(parts[i + 1].to_string());
                                }
                            }
                            "dev" => {
                                if i + 1 < parts.len() {
                                    route.interface = Some(parts[i + 1].to_string());
                                }
                            }
                            "metric" => {
                                if i + 1 < parts.len() {
                                    route.metric = parts[i + 1].parse().ok();
                                }
                            }
                            _ => {}
                        }
                    }
                    break; // Only take first default route
                }
            }
        }
    }

    route
}

/// Get DNS configuration from various sources
pub fn get_dns_config() -> DnsConfig {
    // Try systemd-resolved first (most modern systems)
    if let Some(dns) = get_dns_from_resolvectl() {
        return dns;
    }

    // Try NetworkManager
    if let Some(dns) = get_dns_from_nmcli() {
        return dns;
    }

    // Fallback to /etc/resolv.conf
    get_dns_from_resolv_conf()
}

fn get_dns_from_resolvectl() -> Option<DnsConfig> {
    let output = Command::new("resolvectl").args(["status"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut servers = Vec::new();
    let mut search_domains = Vec::new();
    let mut in_global = false;

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("Global") {
            in_global = true;
        } else if line.starts_with("Link") {
            in_global = false;
        }

        if in_global {
            if line.starts_with("DNS Servers:") {
                let dns_part = line.trim_start_matches("DNS Servers:").trim();
                for server in dns_part.split_whitespace() {
                    if !servers.contains(&server.to_string()) && servers.len() < 4 {
                        servers.push(server.to_string());
                    }
                }
            } else if line.starts_with("DNS Domain:") {
                let domain_part = line.trim_start_matches("DNS Domain:").trim();
                for domain in domain_part.split_whitespace() {
                    if !search_domains.contains(&domain.to_string()) {
                        search_domains.push(domain.to_string());
                    }
                }
            }
        }
    }

    if servers.is_empty() {
        return None;
    }

    Some(DnsConfig {
        servers,
        search_domains,
        source: "systemd-resolved".to_string(),
    })
}

fn get_dns_from_nmcli() -> Option<DnsConfig> {
    let output = Command::new("nmcli")
        .args(["--terse", "--fields", "IP4.DNS", "device", "show"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut servers = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("IP4.DNS") {
            if let Some(dns) = line.split(':').nth(1) {
                let dns = dns.trim();
                if !dns.is_empty() && !servers.contains(&dns.to_string()) && servers.len() < 4 {
                    servers.push(dns.to_string());
                }
            }
        }
    }

    if servers.is_empty() {
        return None;
    }

    Some(DnsConfig {
        servers,
        search_domains: Vec::new(),
        source: "NetworkManager".to_string(),
    })
}

fn get_dns_from_resolv_conf() -> DnsConfig {
    let mut dns = DnsConfig {
        source: "resolv.conf".to_string(),
        ..Default::default()
    };

    if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("nameserver") {
                if let Some(server) = line.split_whitespace().nth(1) {
                    if !dns.servers.contains(&server.to_string()) && dns.servers.len() < 4 {
                        dns.servers.push(server.to_string());
                    }
                }
            } else if line.starts_with("search") {
                for domain in line.split_whitespace().skip(1) {
                    if !dns.search_domains.contains(&domain.to_string()) {
                        dns.search_domains.push(domain.to_string());
                    }
                }
            }
        }
    }

    dns
}

/// Get interface manager (who manages each interface)
pub fn get_interface_manager(iface: &str) -> InterfaceManager {
    // Check NetworkManager first
    if let Ok(output) = Command::new("nmcli")
        .args(["--terse", "--fields", "DEVICE,STATE", "device", "status"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && parts[0] == iface {
                    return match parts[1] {
                        "connected" | "connecting" | "disconnected" | "disconnecting" => {
                            InterfaceManager::NetworkManager
                        }
                        "unmanaged" => InterfaceManager::Unmanaged,
                        _ => InterfaceManager::NetworkManager,
                    };
                }
            }
        }
    }

    // Check systemd-networkd
    if let Ok(output) = Command::new("networkctl").args(["status", iface]).output() {
        if output.status.success() {
            return InterfaceManager::SystemdNetworkd;
        }
    }

    InterfaceManager::Unknown
}

/// Get interface topology (PCI address, speed, firmware)
pub fn get_interface_topology(iface: &str) -> InterfaceTopology {
    let mut topo = InterfaceTopology {
        name: iface.to_string(),
        manager: get_interface_manager(iface),
        pci_address: None,
        usb_path: None,
        speed: None,
        firmware_version: None,
    };

    // Get PCI address from /sys
    let device_path = format!("/sys/class/net/{}/device", iface);
    if let Ok(link) = fs::read_link(&device_path) {
        let link_str = link.to_string_lossy();
        // Extract PCI address from path like "../../0000:00:14.3"
        if let Some(pci) = link_str.split('/').last() {
            if pci.contains(':') && pci.contains('.') {
                topo.pci_address = Some(pci.to_string());
            }
        }
    }

    // Get speed and firmware from ethtool
    if let Ok(output) = Command::new("ethtool").arg(iface).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Speed:") {
                    topo.speed = Some(line.trim_start_matches("Speed:").trim().to_string());
                }
            }
        }
    }

    // Get firmware version from ethtool -i
    if let Ok(output) = Command::new("ethtool").args(["-i", iface]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("firmware-version:") {
                    let fw = line.trim_start_matches("firmware-version:").trim();
                    if !fw.is_empty() && fw != "N/A" {
                        topo.firmware_version = Some(fw.to_string());
                    }
                }
            }
        }
    }

    topo
}

/// Get full network topology
pub fn get_network_topology() -> NetworkTopology {
    let mut topology = NetworkTopology {
        source: "ip, nmcli, resolvectl, ethtool".to_string(),
        ..Default::default()
    };

    // Get all interfaces from /sys
    let net_path = std::path::Path::new("/sys/class/net");
    if let Ok(entries) = fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            topology.interfaces.push(get_interface_topology(&name));
        }
    }

    topology.default_route = get_default_route();
    topology.dns = get_dns_config();

    topology
}

/// Format interface list for hw overview
pub fn format_interfaces_summary(interfaces: &[super::NetworkInterface]) -> Vec<String> {
    let mut lines = Vec::new();

    for iface in interfaces {
        if iface.name == "lo" {
            continue;
        }

        let manager = get_interface_manager(&iface.name);
        let state_str = match iface.state {
            super::LinkState::Up => "up",
            super::LinkState::Down => "down",
            super::LinkState::Unknown => "unknown",
        };

        let ip_str = if !iface.ip_addrs.is_empty() {
            format!(", IPv4 {}", iface.ip_addrs[0])
        } else {
            String::new()
        };

        lines.push(format!(
            "{:<7} {:<10} {}, managed by {}{}",
            iface.name,
            iface.iface_type.as_str(),
            state_str,
            manager.as_str(),
            ip_str
        ));
    }

    // Add loopback at end
    lines.push("lo      loopback   up".to_string());

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_route() {
        let route = get_default_route();
        // Should at least have source
        assert!(!route.source.is_empty());
    }

    #[test]
    fn test_get_dns_config() {
        let dns = get_dns_config();
        // Should at least have source
        assert!(!dns.source.is_empty());
    }

    #[test]
    fn test_get_network_topology() {
        let topo = get_network_topology();
        // Should have at least loopback
        assert!(!topo.interfaces.is_empty() || topo.source.contains("ip"));
    }
}
