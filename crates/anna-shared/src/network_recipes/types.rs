//! Network troubleshooting recipe types (v0.0.462).

use serde::{Deserialize, Serialize};

/// Network troubleshooting features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkFeature {
    /// Test connectivity (ping)
    TestConnectivity,
    /// DNS lookup
    DnsLookup,
    /// Trace route
    TraceRoute,
    /// Check open ports
    CheckPorts,
    /// Network interface info
    InterfaceInfo,
    /// Check bandwidth/speed
    BandwidthTest,
    /// Firewall status
    FirewallStatus,
    /// Check listening ports
    ListeningPorts,
    /// SSL certificate check
    SslCertCheck,
    /// HTTP request test
    HttpTest,
    /// ARP table
    ArpTable,
    /// Routing table
    RoutingTable,
    /// Network statistics
    NetworkStats,
    /// WiFi diagnostics
    WifiDiagnostics,
    /// VPN status
    VpnStatus,
}

impl NetworkFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            NetworkFeature::TestConnectivity => "test connectivity",
            NetworkFeature::DnsLookup => "DNS lookup",
            NetworkFeature::TraceRoute => "trace route",
            NetworkFeature::CheckPorts => "check ports",
            NetworkFeature::InterfaceInfo => "interface info",
            NetworkFeature::BandwidthTest => "bandwidth test",
            NetworkFeature::FirewallStatus => "firewall status",
            NetworkFeature::ListeningPorts => "listening ports",
            NetworkFeature::SslCertCheck => "SSL certificate check",
            NetworkFeature::HttpTest => "HTTP request test",
            NetworkFeature::ArpTable => "ARP table",
            NetworkFeature::RoutingTable => "routing table",
            NetworkFeature::NetworkStats => "network statistics",
            NetworkFeature::WifiDiagnostics => "WiFi diagnostics",
            NetworkFeature::VpnStatus => "VPN status",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            NetworkFeature::TestConnectivity => {
                &["ping", "test connectivity", "network connection", "can't connect"]
            }
            NetworkFeature::DnsLookup => {
                &["dns", "nslookup", "dig", "resolve", "domain lookup", "name resolution"]
            }
            NetworkFeature::TraceRoute => {
                &["traceroute", "tracepath", "trace route", "network path", "hops"]
            }
            NetworkFeature::CheckPorts => {
                &["port scan", "check port", "nmap", "port open", "telnet"]
            }
            NetworkFeature::InterfaceInfo => {
                &["interface", "ip address", "ifconfig", "ip addr", "network card", "nic"]
            }
            NetworkFeature::BandwidthTest => {
                &["speed test", "bandwidth", "network speed", "iperf", "throughput"]
            }
            NetworkFeature::FirewallStatus => {
                &["firewall", "iptables", "nftables", "ufw", "firewalld"]
            }
            NetworkFeature::ListeningPorts => {
                &["listening port", "netstat", "ss -l", "what's listening", "port in use"]
            }
            NetworkFeature::SslCertCheck => {
                &["ssl certificate", "tls certificate", "cert check", "openssl", "certificate expire"]
            }
            NetworkFeature::HttpTest => {
                &["curl", "wget", "http test", "http request", "api test"]
            }
            NetworkFeature::ArpTable => &["arp", "arp table", "mac address", "arp cache"],
            NetworkFeature::RoutingTable => {
                &["routing table", "route", "ip route", "gateway", "default route"]
            }
            NetworkFeature::NetworkStats => {
                &["network stat", "packet", "netstat -s", "network traffic", "bytes sent"]
            }
            NetworkFeature::WifiDiagnostics => {
                &["wifi", "wireless", "wlan", "signal strength", "iwconfig", "nmcli"]
            }
            NetworkFeature::VpnStatus => {
                &["vpn", "wireguard", "openvpn", "vpn status", "tunnel"]
            }
        }
    }
}

impl std::fmt::Display for NetworkFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A network troubleshooting recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRecipe {
    pub feature: NetworkFeature,
    pub description: String,
    pub commands: Vec<String>,
    pub example: Option<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
    /// Required tools
    pub requires: Vec<String>,
}

impl NetworkRecipe {
    pub fn new(feature: NetworkFeature, description: &str) -> Self {
        Self {
            feature,
            description: description.to_string(),
            commands: Vec::new(),
            example: None,
            answer_template: String::new(),
            notes: Vec::new(),
            requires: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }

    pub fn with_requires(mut self, tool: &str) -> Self {
        if !self.requires.contains(&tool.to_string()) {
            self.requires.push(tool.to_string());
        }
        self
    }
}
