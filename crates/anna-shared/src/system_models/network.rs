//! Network Model - Interfaces, routes, DNS, firewall rules, reachability.
//!
//! Models the network topology for understanding:
//! - What interfaces exist and their states
//! - Routing table and default gateways
//! - DNS configuration
//! - Firewall rules
//! - Reachability status

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete network model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkModel {
    /// Network interfaces
    pub interfaces: HashMap<String, NetworkInterface>,
    /// Routing table
    pub routes: Vec<Route>,
    /// DNS configuration
    pub dns: DnsConfig,
    /// Firewall rules
    pub firewall: FirewallConfig,
    /// Reachability tests
    pub reachability: HashMap<String, ReachabilityStatus>,
}

/// A network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "wlan0")
    pub name: String,
    /// Interface type
    pub if_type: InterfaceType,
    /// Operational state
    pub state: InterfaceState,
    /// IPv4 addresses
    pub ipv4_addrs: Vec<String>,
    /// IPv6 addresses
    pub ipv6_addrs: Vec<String>,
    /// MAC address
    pub mac: Option<String>,
    /// MTU
    pub mtu: u32,
    /// Link speed (Mbps, if applicable)
    pub speed: Option<u32>,
    /// Is this the default interface?
    pub is_default: bool,
    /// Associated SSID (for wireless)
    pub ssid: Option<String>,
    /// Signal strength (for wireless)
    pub signal_strength: Option<i32>,
}

/// Interface types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    Ethernet,
    Wireless,
    Loopback,
    Bridge,
    Vlan,
    Tunnel,
    Virtual,
    Unknown,
}

impl InterfaceType {
    pub fn from_name(name: &str) -> Self {
        if name == "lo" {
            InterfaceType::Loopback
        } else if name.starts_with("eth") || name.starts_with("en") {
            InterfaceType::Ethernet
        } else if name.starts_with("wl") || name.starts_with("wlan") {
            InterfaceType::Wireless
        } else if name.starts_with("br") {
            InterfaceType::Bridge
        } else if name.starts_with("vlan") || name.contains('.') {
            InterfaceType::Vlan
        } else if name.starts_with("tun") || name.starts_with("tap") {
            InterfaceType::Tunnel
        } else if name.starts_with("veth") || name.starts_with("docker") {
            InterfaceType::Virtual
        } else {
            InterfaceType::Unknown
        }
    }
}

/// Interface operational state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceState {
    Up,
    Down,
    Unknown,
    NotPresent,
    LowerLayerDown,
    Testing,
    Dormant,
}

impl InterfaceState {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "UP" => InterfaceState::Up,
            "DOWN" => InterfaceState::Down,
            "UNKNOWN" => InterfaceState::Unknown,
            "NOTPRESENT" => InterfaceState::NotPresent,
            "LOWERLAYERDOWN" => InterfaceState::LowerLayerDown,
            "TESTING" => InterfaceState::Testing,
            "DORMANT" => InterfaceState::Dormant,
            _ => InterfaceState::Unknown,
        }
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, InterfaceState::Up | InterfaceState::Dormant)
    }
}

/// A routing table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Destination network (CIDR or "default")
    pub destination: String,
    /// Gateway address (if any)
    pub gateway: Option<String>,
    /// Output interface
    pub interface: String,
    /// Route metric
    pub metric: u32,
    /// Route scope
    pub scope: RouteScope,
    /// Route protocol (how it was added)
    pub protocol: String,
}

/// Route scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteScope {
    Universe,
    Link,
    Host,
    Site,
    Nowhere,
}

impl RouteScope {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "global" | "universe" => RouteScope::Universe,
            "link" => RouteScope::Link,
            "host" => RouteScope::Host,
            "site" => RouteScope::Site,
            _ => RouteScope::Nowhere,
        }
    }
}

/// DNS configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DnsConfig {
    /// Nameservers in order of preference
    pub nameservers: Vec<String>,
    /// Search domains
    pub search_domains: Vec<String>,
    /// DNS source (systemd-resolved, NetworkManager, manual)
    pub source: DnsSource,
    /// Is DNS working?
    pub working: bool,
    /// Last test result
    pub last_test: Option<String>,
}

/// DNS configuration source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DnsSource {
    #[default]
    Unknown,
    Manual,
    NetworkManager,
    SystemdResolved,
    Dhcp,
}

/// Firewall configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirewallConfig {
    /// Firewall backend in use
    pub backend: FirewallBackend,
    /// Is firewall active?
    pub active: bool,
    /// Input chain default policy
    pub default_input: FirewallAction,
    /// Forward chain default policy
    pub default_forward: FirewallAction,
    /// Output chain default policy
    pub default_output: FirewallAction,
    /// Open ports
    pub open_ports: Vec<OpenPort>,
    /// Custom rules count
    pub rule_count: usize,
}

/// Firewall backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FirewallBackend {
    #[default]
    None,
    Iptables,
    Nftables,
    Firewalld,
    Ufw,
}

/// Firewall action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FirewallAction {
    #[default]
    Accept,
    Drop,
    Reject,
}

/// An open port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPort {
    /// Port number
    pub port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
    /// Service using this port
    pub service: Option<String>,
    /// Source restriction (if any)
    pub source: Option<String>,
}

/// Reachability test status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityStatus {
    /// Target (hostname or IP)
    pub target: String,
    /// Is reachable?
    pub reachable: bool,
    /// Latency in milliseconds
    pub latency_ms: Option<f64>,
    /// Last test time
    pub last_tested: String,
    /// Error message if unreachable
    pub error: Option<String>,
}

impl NetworkModel {
    /// Create new empty network model
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update an interface
    pub fn upsert_interface(&mut self, iface: NetworkInterface) {
        self.interfaces.insert(iface.name.clone(), iface);
    }

    /// Add a route
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Get the default gateway
    pub fn default_gateway(&self) -> Option<&Route> {
        self.routes.iter().find(|r| r.destination == "default")
    }

    /// Get the default interface
    pub fn default_interface(&self) -> Option<&NetworkInterface> {
        self.interfaces.values().find(|i| i.is_default)
    }

    /// Count unreachable targets
    pub fn count_unreachable(&self) -> usize {
        self.reachability.values().filter(|r| !r.reachable).count()
    }

    /// Check if internet is reachable
    pub fn has_internet(&self) -> bool {
        self.reachability
            .values()
            .any(|r| r.reachable && is_internet_target(&r.target))
    }

    /// Get all interfaces with IPs
    pub fn interfaces_with_ip(&self) -> Vec<&NetworkInterface> {
        self.interfaces
            .values()
            .filter(|i| !i.ipv4_addrs.is_empty() || !i.ipv6_addrs.is_empty())
            .collect()
    }

    /// Update reachability status
    pub fn update_reachability(&mut self, target: &str, reachable: bool, latency_ms: Option<f64>) {
        self.reachability.insert(
            target.to_string(),
            ReachabilityStatus {
                target: target.to_string(),
                reachable,
                latency_ms,
                last_tested: chrono::Utc::now().to_rfc3339(),
                error: if reachable {
                    None
                } else {
                    Some("Unreachable".to_string())
                },
            },
        );
    }

    /// Diagnose common network issues
    pub fn diagnose(&self) -> Vec<NetworkIssue> {
        let mut issues = Vec::new();

        // Check for no interfaces up
        let up_interfaces: Vec<_> = self
            .interfaces
            .values()
            .filter(|i| i.state.is_operational() && i.if_type != InterfaceType::Loopback)
            .collect();

        if up_interfaces.is_empty() {
            issues.push(NetworkIssue {
                severity: NetworkIssueSeverity::Critical,
                component: "interfaces".to_string(),
                description: "No network interfaces are up".to_string(),
                suggestion: "Check physical connections and run 'ip link set <iface> up'".to_string(),
            });
        }

        // Check for no default gateway
        if self.default_gateway().is_none() {
            issues.push(NetworkIssue {
                severity: NetworkIssueSeverity::High,
                component: "routing".to_string(),
                description: "No default gateway configured".to_string(),
                suggestion: "Add default route or check DHCP".to_string(),
            });
        }

        // Check DNS
        if self.dns.nameservers.is_empty() {
            issues.push(NetworkIssue {
                severity: NetworkIssueSeverity::High,
                component: "dns".to_string(),
                description: "No DNS nameservers configured".to_string(),
                suggestion: "Check /etc/resolv.conf or systemd-resolved".to_string(),
            });
        } else if !self.dns.working {
            issues.push(NetworkIssue {
                severity: NetworkIssueSeverity::High,
                component: "dns".to_string(),
                description: "DNS resolution is not working".to_string(),
                suggestion: "Test with 'dig google.com' or try different nameservers".to_string(),
            });
        }

        // Check reachability
        if !self.has_internet() && !self.reachability.is_empty() {
            issues.push(NetworkIssue {
                severity: NetworkIssueSeverity::High,
                component: "connectivity".to_string(),
                description: "Internet is not reachable".to_string(),
                suggestion: "Check gateway reachability and ISP status".to_string(),
            });
        }

        issues
    }
}

/// Check if a target is an internet target
fn is_internet_target(target: &str) -> bool {
    let internet_targets = [
        "8.8.8.8",
        "1.1.1.1",
        "google.com",
        "cloudflare.com",
        "archlinux.org",
    ];
    internet_targets.iter().any(|t| target.contains(t))
}

/// A diagnosed network issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIssue {
    pub severity: NetworkIssueSeverity,
    pub component: String,
    pub description: String,
    pub suggestion: String,
}

/// Network issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkIssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl NetworkInterface {
    /// Create a new interface
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            if_type: InterfaceType::from_name(name),
            state: InterfaceState::Unknown,
            ipv4_addrs: Vec::new(),
            ipv6_addrs: Vec::new(),
            mac: None,
            mtu: 1500,
            speed: None,
            is_default: false,
            ssid: None,
            signal_strength: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_type_detection() {
        assert_eq!(InterfaceType::from_name("eth0"), InterfaceType::Ethernet);
        assert_eq!(InterfaceType::from_name("enp0s3"), InterfaceType::Ethernet);
        assert_eq!(InterfaceType::from_name("wlan0"), InterfaceType::Wireless);
        assert_eq!(InterfaceType::from_name("wlp2s0"), InterfaceType::Wireless);
        assert_eq!(InterfaceType::from_name("lo"), InterfaceType::Loopback);
        assert_eq!(InterfaceType::from_name("br0"), InterfaceType::Bridge);
    }

    #[test]
    fn test_diagnose_no_gateway() {
        let model = NetworkModel::new();
        let issues = model.diagnose();

        assert!(issues.iter().any(|i| i.component == "routing"));
    }

    #[test]
    fn test_reachability_update() {
        let mut model = NetworkModel::new();
        model.update_reachability("8.8.8.8", true, Some(15.5));

        assert!(model.reachability.get("8.8.8.8").unwrap().reachable);
        assert!(model.has_internet());
    }
}
