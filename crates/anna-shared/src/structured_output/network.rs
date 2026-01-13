//! Network structured output - parse ip and ss JSON output.

use super::ParseResult;
use serde::{Deserialize, Serialize};

/// Network interface from `ip -j addr`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface index
    pub ifindex: u32,
    /// Interface name (e.g., "eth0", "wlan0")
    pub ifname: String,
    /// Flags (e.g., ["UP", "BROADCAST", "RUNNING"])
    #[serde(default)]
    pub flags: Vec<String>,
    /// MTU
    #[serde(default)]
    pub mtu: u32,
    /// Queue discipline
    #[serde(default)]
    pub qdisc: String,
    /// Operational state
    #[serde(default)]
    pub operstate: String,
    /// Link type
    #[serde(default)]
    pub link_type: String,
    /// MAC address
    #[serde(default)]
    pub address: Option<String>,
    /// Broadcast address
    #[serde(default)]
    pub broadcast: Option<String>,
    /// IP addresses
    #[serde(default)]
    pub addr_info: Vec<IpAddress>,
}

impl NetworkInterface {
    /// Check if interface is up
    pub fn is_up(&self) -> bool {
        self.flags.iter().any(|f| f == "UP")
            || self.operstate.to_lowercase() == "up"
    }

    /// Get the primary IPv4 address
    pub fn ipv4(&self) -> Option<&str> {
        self.addr_info
            .iter()
            .find(|a| a.family == "inet")
            .map(|a| a.local.as_str())
    }

    /// Get the primary IPv6 address
    pub fn ipv6(&self) -> Option<&str> {
        self.addr_info
            .iter()
            .find(|a| a.family == "inet6" && !a.local.starts_with("fe80"))
            .map(|a| a.local.as_str())
    }
}

/// IP address info from `ip -j addr`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddress {
    /// Address family ("inet" or "inet6")
    pub family: String,
    /// Local IP address
    pub local: String,
    /// Prefix length
    #[serde(default)]
    pub prefixlen: u8,
    /// Scope
    #[serde(default)]
    pub scope: String,
    /// Address label
    #[serde(default)]
    pub label: Option<String>,
    /// Preferred lifetime
    #[serde(default)]
    pub preferred_life_time: Option<u64>,
    /// Valid lifetime
    #[serde(default)]
    pub valid_life_time: Option<u64>,
}

/// Socket info from `ss -j`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketInfo {
    /// Protocol (tcp, udp, etc.)
    #[serde(default)]
    pub protocol: String,
    /// State (LISTEN, ESTABLISHED, etc.)
    #[serde(default)]
    pub state: String,
    /// Receive queue size
    #[serde(default, rename = "recv-q")]
    pub recv_q: u64,
    /// Send queue size
    #[serde(default, rename = "send-q")]
    pub send_q: u64,
    /// Local address
    #[serde(default)]
    pub local: SocketAddress,
    /// Peer address
    #[serde(default)]
    pub peer: SocketAddress,
    /// Process info
    #[serde(default)]
    pub process: Option<String>,
}

/// Socket address
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocketAddress {
    /// IP address or hostname
    #[serde(default)]
    pub address: String,
    /// Port number
    #[serde(default)]
    pub port: u16,
}

impl SocketInfo {
    /// Check if this is a listening socket
    pub fn is_listening(&self) -> bool {
        self.state.to_uppercase() == "LISTEN"
    }

    /// Check if this is an established connection
    pub fn is_established(&self) -> bool {
        self.state.to_uppercase() == "ESTAB" || self.state.to_uppercase() == "ESTABLISHED"
    }
}

/// Parse `ip -j addr` output
pub fn parse_ip_output(output: &str) -> ParseResult<Vec<NetworkInterface>> {
    super::parse_json(output)
}

/// Parse `ss -j` output
pub fn parse_ss_output(output: &str) -> ParseResult<Vec<SocketInfo>> {
    // ss -j wraps output in a root object
    #[derive(Deserialize)]
    struct SsOutput {
        #[serde(default)]
        tcp: Vec<SocketInfo>,
        #[serde(default)]
        udp: Vec<SocketInfo>,
        #[serde(default, rename = "TCP")]
        tcp_caps: Vec<SocketInfo>,
        #[serde(default, rename = "UDP")]
        udp_caps: Vec<SocketInfo>,
    }

    match super::parse_json::<SsOutput>(output) {
        ParseResult::Ok(ss) => {
            let mut all = Vec::new();
            all.extend(ss.tcp);
            all.extend(ss.udp);
            all.extend(ss.tcp_caps);
            all.extend(ss.udp_caps);
            ParseResult::Ok(all)
        }
        ParseResult::RawText(t) => ParseResult::RawText(t),
        ParseResult::ParseError(e) => ParseResult::ParseError(e),
        ParseResult::CommandError(e) => ParseResult::CommandError(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip_output() {
        let json = r#"[
            {
                "ifindex": 1,
                "ifname": "lo",
                "flags": ["LOOPBACK", "UP", "LOWER_UP"],
                "mtu": 65536,
                "qdisc": "noqueue",
                "operstate": "UNKNOWN",
                "link_type": "loopback",
                "address": "00:00:00:00:00:00",
                "broadcast": "00:00:00:00:00:00",
                "addr_info": [
                    {
                        "family": "inet",
                        "local": "127.0.0.1",
                        "prefixlen": 8,
                        "scope": "host"
                    }
                ]
            }
        ]"#;

        let result = parse_ip_output(json);
        assert!(result.is_ok());

        let interfaces = result.ok().unwrap();
        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].ifname, "lo");
        assert!(interfaces[0].is_up());
        assert_eq!(interfaces[0].ipv4(), Some("127.0.0.1"));
    }
}
