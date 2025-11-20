use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Comprehensive network monitoring information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMonitoring {
    /// List of active network interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// IPv4 status and connectivity
    pub ipv4_status: IpVersionStatus,
    /// IPv6 status and connectivity
    pub ipv6_status: IpVersionStatus,
    /// DNSSEC validation status
    pub dnssec_status: DnssecStatus,
    /// Network latency measurements
    pub latency: LatencyMetrics,
    /// Packet loss statistics
    pub packet_loss: PacketLossStats,
    /// System routing table
    pub routes: Vec<Route>,
    /// Active firewall rules
    pub firewall_rules: FirewallRules,
}

/// Individual network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "wlan0", "enp0s3")
    pub name: String,
    /// Interface type (Ethernet, WiFi, Loopback, Virtual, etc.)
    pub interface_type: InterfaceType,
    /// Whether the interface is currently up/active
    pub is_up: bool,
    /// MAC address
    pub mac_address: Option<String>,
    /// IPv4 addresses assigned to this interface
    pub ipv4_addresses: Vec<String>,
    /// IPv6 addresses assigned to this interface
    pub ipv6_addresses: Vec<String>,
    /// MTU size
    pub mtu: Option<u32>,
    /// Link speed in Mbps (for physical interfaces)
    pub speed_mbps: Option<u32>,
    /// Address configuration method
    pub config_method: AddressConfig,
    /// Statistics for this interface
    pub stats: InterfaceStats,
}

/// Type of network interface
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceType {
    Ethernet,
    WiFi,
    Loopback,
    Virtual,
    Bridge,
    Tunnel,
    Unknown,
}

/// Address configuration method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AddressConfig {
    /// DHCP-assigned address
    DHCP,
    /// Static/manual configuration
    Static,
    /// Link-local address only
    LinkLocal,
    /// Unknown or mixed
    Unknown,
}

/// Interface statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Bytes received
    pub rx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Receive drops
    pub rx_dropped: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Transmit drops
    pub tx_dropped: u64,
}

/// IP version connectivity status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpVersionStatus {
    /// Whether this IP version is enabled on the system
    pub enabled: bool,
    /// Whether this IP version has connectivity
    pub has_connectivity: bool,
    /// Default gateway for this IP version
    pub default_gateway: Option<String>,
    /// Number of addresses configured
    pub address_count: usize,
}

/// DNSSEC validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnssecStatus {
    /// Whether DNSSEC validation is enabled
    pub enabled: bool,
    /// Resolver used (systemd-resolved, unbound, etc.)
    pub resolver: Option<String>,
    /// DNSSEC validation mode (if known)
    pub validation_mode: Option<String>,
}

/// Network latency measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Latency to default gateway in milliseconds
    pub gateway_latency_ms: Option<f64>,
    /// Latency to primary DNS server in milliseconds
    pub dns_latency_ms: Option<f64>,
    /// Latency to internet (via ping to well-known host) in milliseconds
    pub internet_latency_ms: Option<f64>,
    /// Whether latency measurements succeeded
    pub measurement_successful: bool,
}

/// Packet loss statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossStats {
    /// Packet loss percentage to default gateway
    pub gateway_loss_percent: Option<f64>,
    /// Packet loss percentage to primary DNS
    pub dns_loss_percent: Option<f64>,
    /// Packet loss percentage to internet
    pub internet_loss_percent: Option<f64>,
    /// Whether measurements completed
    pub measurement_successful: bool,
}

/// Routing table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Destination network (CIDR notation)
    pub destination: String,
    /// Gateway IP address
    pub gateway: Option<String>,
    /// Output interface
    pub interface: String,
    /// Route metric/priority
    pub metric: Option<u32>,
    /// Route protocol (kernel, boot, static, dhcp, etc.)
    pub protocol: Option<String>,
}

/// Firewall rules information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRules {
    /// Firewall type (iptables, nftables, ufw, firewalld)
    pub firewall_type: Option<String>,
    /// Whether firewall is active
    pub is_active: bool,
    /// Number of active rules
    pub rule_count: u32,
    /// Default policy for INPUT chain
    pub default_input_policy: Option<String>,
    /// Default policy for OUTPUT chain
    pub default_output_policy: Option<String>,
    /// Default policy for FORWARD chain
    pub default_forward_policy: Option<String>,
    /// Summary of open ports
    pub open_ports: Vec<PortRule>,
}

/// Port rule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRule {
    /// Port number
    pub port: u16,
    /// Protocol (TCP, UDP)
    pub protocol: String,
    /// Source restriction (if any)
    pub source: Option<String>,
    /// Action (ACCEPT, DROP, REJECT)
    pub action: String,
}

impl NetworkMonitoring {
    /// Detect comprehensive network monitoring information
    pub fn detect() -> Self {
        let interfaces = detect_network_interfaces();
        let ipv4_status = detect_ipv4_status(&interfaces);
        let ipv6_status = detect_ipv6_status(&interfaces);
        let dnssec_status = detect_dnssec_status();
        let latency = measure_latency(&ipv4_status, &ipv6_status);
        let packet_loss = measure_packet_loss(&ipv4_status, &ipv6_status);
        let routes = get_routing_table();
        let firewall_rules = get_firewall_rules();

        NetworkMonitoring {
            interfaces,
            ipv4_status,
            ipv6_status,
            dnssec_status,
            latency,
            packet_loss,
            routes,
            firewall_rules,
        }
    }
}

/// Detect all network interfaces and their configuration
fn detect_network_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    // Get list of interfaces from /sys/class/net
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let if_name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback for now (we'll add it separately if needed)
            if if_name == "lo" {
                continue;
            }

            let interface_type = detect_interface_type(&if_name);
            let is_up = check_interface_up(&if_name);
            let mac_address = get_mac_address(&if_name);
            let (ipv4_addresses, ipv6_addresses) = get_ip_addresses(&if_name);
            let mtu = get_mtu(&if_name);
            let speed_mbps = get_link_speed(&if_name);
            let config_method = detect_address_config(&if_name, &ipv4_addresses, &ipv6_addresses);
            let stats = get_interface_stats(&if_name);

            interfaces.push(NetworkInterface {
                name: if_name,
                interface_type,
                is_up,
                mac_address,
                ipv4_addresses,
                ipv6_addresses,
                mtu,
                speed_mbps,
                config_method,
                stats,
            });
        }
    }

    interfaces
}

/// Detect interface type from name and sysfs
fn detect_interface_type(if_name: &str) -> InterfaceType {
    // Check for wireless
    let wireless_path = format!("/sys/class/net/{}/wireless", if_name);
    if std::path::Path::new(&wireless_path).exists() {
        return InterfaceType::WiFi;
    }

    // Check for virtual/bridge/tunnel via uevent
    let uevent_path = format!("/sys/class/net/{}/uevent", if_name);
    if let Ok(content) = fs::read_to_string(&uevent_path) {
        if content.contains("DEVTYPE=bridge") {
            return InterfaceType::Bridge;
        }
        if content.contains("DEVTYPE=tun") || content.contains("DEVTYPE=tap") {
            return InterfaceType::Tunnel;
        }
    }

    // Check if it's a virtual device
    let device_link = format!("/sys/class/net/{}/device", if_name);
    if !std::path::Path::new(&device_link).exists() {
        return InterfaceType::Virtual;
    }

    // Default to Ethernet for physical devices
    InterfaceType::Ethernet
}

/// Check if interface is up
fn check_interface_up(if_name: &str) -> bool {
    let operstate_path = format!("/sys/class/net/{}/operstate", if_name);
    fs::read_to_string(&operstate_path)
        .map(|s| s.trim() == "up")
        .unwrap_or(false)
}

/// Get MAC address
fn get_mac_address(if_name: &str) -> Option<String> {
    let address_path = format!("/sys/class/net/{}/address", if_name);
    fs::read_to_string(&address_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Get IP addresses (both IPv4 and IPv6)
fn get_ip_addresses(if_name: &str) -> (Vec<String>, Vec<String>) {
    let mut ipv4_addrs = Vec::new();
    let mut ipv6_addrs = Vec::new();

    // Use `ip addr show` to get addresses
    if let Ok(output) = Command::new("ip").args(["addr", "show", if_name]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("inet ") {
                // IPv4 address
                if let Some(addr) = trimmed.split_whitespace().nth(1) {
                    ipv4_addrs.push(addr.to_string());
                }
            } else if trimmed.starts_with("inet6 ") {
                // IPv6 address
                if let Some(addr) = trimmed.split_whitespace().nth(1) {
                    ipv6_addrs.push(addr.to_string());
                }
            }
        }
    }

    (ipv4_addrs, ipv6_addrs)
}

/// Get MTU
fn get_mtu(if_name: &str) -> Option<u32> {
    let mtu_path = format!("/sys/class/net/{}/mtu", if_name);
    fs::read_to_string(&mtu_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Get link speed in Mbps
fn get_link_speed(if_name: &str) -> Option<u32> {
    let speed_path = format!("/sys/class/net/{}/speed", if_name);
    fs::read_to_string(&speed_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Detect address configuration method (DHCP vs static)
fn detect_address_config(
    if_name: &str,
    ipv4_addrs: &[String],
    ipv6_addrs: &[String],
) -> AddressConfig {
    // Check if NetworkManager is managing this interface
    if let Ok(output) = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "DEVICE,TYPE,METHOD",
            "connection",
            "show",
            "--active",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && parts[0] == if_name {
                let method = parts[2].to_lowercase();
                if method.contains("auto") || method.contains("dhcp") {
                    return AddressConfig::DHCP;
                } else if method.contains("manual") || method.contains("static") {
                    return AddressConfig::Static;
                }
            }
        }
    }

    // Check systemd-networkd if NM didn't match
    let networkd_path = format!("/run/systemd/netif/leases/{}", if_name);
    if std::path::Path::new(&networkd_path).exists() {
        return AddressConfig::DHCP;
    }

    // If we have addresses, assume static unless proven otherwise
    if !ipv4_addrs.is_empty() || !ipv6_addrs.is_empty() {
        // Check if addresses are link-local only
        let all_link_local = ipv4_addrs.iter().all(|a| a.starts_with("169.254."))
            && ipv6_addrs
                .iter()
                .all(|a| a.to_lowercase().starts_with("fe80:"));

        if all_link_local {
            return AddressConfig::LinkLocal;
        }

        // Default to Static if we can't determine
        return AddressConfig::Static;
    }

    AddressConfig::Unknown
}

/// Get interface statistics from /sys
fn get_interface_stats(if_name: &str) -> InterfaceStats {
    let read_stat = |stat_name: &str| -> u64 {
        let path = format!("/sys/class/net/{}/statistics/{}", if_name, stat_name);
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    };

    InterfaceStats {
        rx_bytes: read_stat("rx_bytes"),
        rx_packets: read_stat("rx_packets"),
        rx_errors: read_stat("rx_errors"),
        rx_dropped: read_stat("rx_dropped"),
        tx_bytes: read_stat("tx_bytes"),
        tx_packets: read_stat("tx_packets"),
        tx_errors: read_stat("tx_errors"),
        tx_dropped: read_stat("tx_dropped"),
    }
}

/// Detect IPv4 status
fn detect_ipv4_status(interfaces: &[NetworkInterface]) -> IpVersionStatus {
    let enabled = std::path::Path::new("/proc/sys/net/ipv4").exists();
    let address_count = interfaces.iter().map(|i| i.ipv4_addresses.len()).sum();
    let has_connectivity = interfaces.iter().any(|i| {
        i.is_up
            && i.ipv4_addresses
                .iter()
                .any(|a| !a.starts_with("169.254.") && !a.starts_with("127."))
    });

    let default_gateway = get_default_gateway_v4();

    IpVersionStatus {
        enabled,
        has_connectivity,
        default_gateway,
        address_count,
    }
}

/// Detect IPv6 status
fn detect_ipv6_status(interfaces: &[NetworkInterface]) -> IpVersionStatus {
    let enabled = std::path::Path::new("/proc/sys/net/ipv6").exists();
    let address_count = interfaces.iter().map(|i| i.ipv6_addresses.len()).sum();
    let has_connectivity = interfaces.iter().any(|i| {
        i.is_up
            && i.ipv6_addresses.iter().any(|a| {
                let lower = a.to_lowercase();
                !lower.starts_with("fe80:") && !lower.starts_with("::1")
            })
    });

    let default_gateway = get_default_gateway_v6();

    IpVersionStatus {
        enabled,
        has_connectivity,
        default_gateway,
        address_count,
    }
}

/// Get default gateway for IPv4
fn get_default_gateway_v4() -> Option<String> {
    if let Ok(output) = Command::new("ip")
        .args(["-4", "route", "show", "default"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(gw) = line.split_whitespace().nth(2) {
                return Some(gw.to_string());
            }
        }
    }
    None
}

/// Get default gateway for IPv6
fn get_default_gateway_v6() -> Option<String> {
    if let Ok(output) = Command::new("ip")
        .args(["-6", "route", "show", "default"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(gw) = line.split_whitespace().nth(2) {
                return Some(gw.to_string());
            }
        }
    }
    None
}

/// Detect DNSSEC status
fn detect_dnssec_status() -> DnssecStatus {
    // Check if systemd-resolved is running
    let systemd_resolved_running = Command::new("systemctl")
        .args(["is-active", "systemd-resolved"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if systemd_resolved_running {
        // Check DNSSEC status from resolved
        if let Ok(output) = Command::new("resolvectl").arg("status").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let enabled = stdout.contains("DNSSEC: yes") || stdout.contains("DNSSEC NTA:");
            let validation_mode = if stdout.contains("allow-downgrade") {
                Some("allow-downgrade".to_string())
            } else if stdout.contains("yes") {
                Some("yes".to_string())
            } else {
                None
            };

            return DnssecStatus {
                enabled,
                resolver: Some("systemd-resolved".to_string()),
                validation_mode,
            };
        }
    }

    // Check for unbound
    if Command::new("which")
        .arg("unbound")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        // Could parse unbound config here
        return DnssecStatus {
            enabled: false, // Would need to parse config
            resolver: Some("unbound".to_string()),
            validation_mode: None,
        };
    }

    DnssecStatus {
        enabled: false,
        resolver: None,
        validation_mode: None,
    }
}

/// Measure network latency to various targets
fn measure_latency(ipv4_status: &IpVersionStatus, ipv6_status: &IpVersionStatus) -> LatencyMetrics {
    let gateway_latency_ms = ipv4_status
        .default_gateway
        .as_ref()
        .and_then(|gw| ping_host(gw, 3));

    let dns_latency_ms = None; // Would need to get DNS server and ping it

    let internet_latency_ms = ping_host("8.8.8.8", 3);

    let measurement_successful = gateway_latency_ms.is_some() || internet_latency_ms.is_some();

    LatencyMetrics {
        gateway_latency_ms,
        dns_latency_ms,
        internet_latency_ms,
        measurement_successful,
    }
}

/// Measure packet loss to various targets
fn measure_packet_loss(
    ipv4_status: &IpVersionStatus,
    ipv6_status: &IpVersionStatus,
) -> PacketLossStats {
    let gateway_loss_percent = ipv4_status
        .default_gateway
        .as_ref()
        .and_then(|gw| ping_loss(gw, 10));

    let dns_loss_percent = None; // Would need DNS server

    let internet_loss_percent = ping_loss("8.8.8.8", 10);

    let measurement_successful = gateway_loss_percent.is_some() || internet_loss_percent.is_some();

    PacketLossStats {
        gateway_loss_percent,
        dns_loss_percent,
        internet_loss_percent,
        measurement_successful,
    }
}

/// Ping a host and return average latency in ms
fn ping_host(host: &str, count: u32) -> Option<f64> {
    let output = Command::new("ping")
        .args(["-c", &count.to_string(), "-W", "1", host])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse "rtt min/avg/max/mdev = X/Y/Z/W ms"
    for line in stdout.lines() {
        if line.contains("rtt min/avg/max") || line.contains("round-trip") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let values: Vec<&str> = parts[1].trim().split('/').collect();
                if values.len() >= 2 {
                    if let Ok(avg) = values[1].trim().parse::<f64>() {
                        return Some(avg);
                    }
                }
            }
        }
    }

    None
}

/// Ping a host and return packet loss percentage
fn ping_loss(host: &str, count: u32) -> Option<f64> {
    let output = Command::new("ping")
        .args(["-c", &count.to_string(), "-W", "1", host])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse "X% packet loss"
    for line in stdout.lines() {
        if line.contains("packet loss") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (_i, part) in parts.iter().enumerate() {
                if part.ends_with('%') {
                    if let Ok(loss) = part.trim_end_matches('%').parse::<f64>() {
                        return Some(loss);
                    }
                }
            }
        }
    }

    None
}

/// Get routing table
fn get_routing_table() -> Vec<Route> {
    let mut routes = Vec::new();

    // Get IPv4 routes
    if let Ok(output) = Command::new("ip").args(["-4", "route", "show"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(route) = parse_route_line(line) {
                routes.push(route);
            }
        }
    }

    // Get IPv6 routes
    if let Ok(output) = Command::new("ip").args(["-6", "route", "show"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(route) = parse_route_line(line) {
                routes.push(route);
            }
        }
    }

    routes
}

/// Parse a route line from `ip route` output
fn parse_route_line(line: &str) -> Option<Route> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let destination = parts[0].to_string();
    let mut gateway = None;
    let mut interface = String::new();
    let mut metric = None;
    let mut protocol = None;

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "via" if i + 1 < parts.len() => {
                gateway = Some(parts[i + 1].to_string());
                i += 2;
            }
            "dev" if i + 1 < parts.len() => {
                interface = parts[i + 1].to_string();
                i += 2;
            }
            "metric" if i + 1 < parts.len() => {
                metric = parts[i + 1].parse().ok();
                i += 2;
            }
            "proto" if i + 1 < parts.len() => {
                protocol = Some(parts[i + 1].to_string());
                i += 2;
            }
            _ => i += 1,
        }
    }

    Some(Route {
        destination,
        gateway,
        interface,
        metric,
        protocol,
    })
}

/// Get firewall rules
fn get_firewall_rules() -> FirewallRules {
    // Try nftables first
    if let Ok(output) = Command::new("nft").args(["list", "ruleset"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let rule_count = stdout
                .lines()
                .filter(|l| {
                    l.trim().starts_with("ip ")
                        || l.trim().starts_with("tcp ")
                        || l.trim().starts_with("udp ")
                })
                .count() as u32;

            return FirewallRules {
                firewall_type: Some("nftables".to_string()),
                is_active: true,
                rule_count,
                default_input_policy: None,
                default_output_policy: None,
                default_forward_policy: None,
                open_ports: Vec::new(), // Would need more parsing
            };
        }
    }

    // Try iptables
    if let Ok(output) = Command::new("iptables").args(["-L", "-n"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let rule_count = stdout
                .lines()
                .filter(|l| {
                    l.starts_with("ACCEPT") || l.starts_with("DROP") || l.starts_with("REJECT")
                })
                .count() as u32;

            // Parse default policies
            let mut default_input = None;
            let mut default_output = None;
            let mut default_forward = None;

            for line in stdout.lines() {
                if line.starts_with("Chain INPUT") {
                    if let Some(policy) = line.split("policy").nth(1) {
                        default_input =
                            Some(policy.split_whitespace().next().unwrap_or("").to_string());
                    }
                }
                if line.starts_with("Chain OUTPUT") {
                    if let Some(policy) = line.split("policy").nth(1) {
                        default_output =
                            Some(policy.split_whitespace().next().unwrap_or("").to_string());
                    }
                }
                if line.starts_with("Chain FORWARD") {
                    if let Some(policy) = line.split("policy").nth(1) {
                        default_forward =
                            Some(policy.split_whitespace().next().unwrap_or("").to_string());
                    }
                }
            }

            return FirewallRules {
                firewall_type: Some("iptables".to_string()),
                is_active: true,
                rule_count,
                default_input_policy: default_input,
                default_output_policy: default_output,
                default_forward_policy: default_forward,
                open_ports: Vec::new(), // Would need more parsing
            };
        }
    }

    // No active firewall detected
    FirewallRules {
        firewall_type: None,
        is_active: false,
        rule_count: 0,
        default_input_policy: None,
        default_output_policy: None,
        default_forward_policy: None,
        open_ports: Vec::new(),
    }
}
