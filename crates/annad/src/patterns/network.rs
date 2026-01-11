//! Network patterns - connectivity, configuration, and diagnostic queries
//! v0.0.948: Initial network patterns for common networking tasks
//! v0.0.982: Added bandwidth/traffic monitoring patterns

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, and command templates
type NetworkPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

/// Match common network-related questions
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Connection status
    if let Some(u) = match_connection_status(q) {
        return Some(u);
    }
    // Network configuration
    if let Some(u) = match_network_config(q) {
        return Some(u);
    }
    // DNS queries
    if let Some(u) = match_dns(q) {
        return Some(u);
    }
    // VPN
    if let Some(u) = match_vpn(q) {
        return Some(u);
    }
    // Ports and connections
    if let Some(u) = match_ports(q) {
        return Some(u);
    }
    // Bandwidth and traffic monitoring
    if let Some(u) = match_bandwidth(q) {
        return Some(u);
    }
    None
}

fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

/// Connection status queries
fn match_connection_status(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // Am I connected?
        (&["am", "i", "connect"], "check internet connectivity", "network",
            &["ping -c 2 8.8.8.8", "nmcli general status", "ip addr | grep 'state UP'"]),
        (&["internet", "work"], "check if internet is working", "network",
            &["ping -c 2 google.com", "nmcli general status"]),
        (&["online"], "check online status", "network",
            &["ping -c 2 8.8.8.8", "nmcli connection show --active"]),
        // Network interfaces
        (&["network", "interface"], "list network interfaces", "network",
            &["ip link show", "nmcli device status"]),
        (&["list", "interface"], "list network interfaces", "network",
            &["ip link", "ifconfig -a 2>/dev/null || ip addr"]),
        (&["ethernet", "connect"], "check ethernet connection", "network",
            &["ip link show | grep -A1 'en\\|eth'", "nmcli device status | grep ethernet"]),
        // WiFi status
        (&["wifi", "status"], "check WiFi status", "network",
            &["nmcli device wifi list", "iwconfig 2>/dev/null | head -20"]),
        (&["wireless", "connect"], "check wireless connection", "network",
            &["nmcli device wifi list", "iw dev | head -20"]),
        (&["wifi", "signal"], "check WiFi signal strength", "network",
            &["nmcli -f SIGNAL,SSID device wifi", "iw dev wlan0 link 2>/dev/null"]),
        // Connection details
        (&["connection", "detail"], "show connection details", "network",
            &["nmcli connection show --active", "ip addr"]),
        (&["active", "connection"], "show active connections", "network",
            &["nmcli connection show --active"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Network configuration queries
fn match_network_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // IP address
        (&["my", "ip"], "show IP address", "network",
            &["ip -4 addr show | grep inet | grep -v 127.0.0.1", "hostname -I"]),
        (&["ip", "address"], "show IP addresses", "network",
            &["ip addr", "ifconfig 2>/dev/null || ip addr"]),
        (&["public", "ip"], "show public IP address", "network",
            &["curl -s ifconfig.me", "curl -s ipinfo.io/ip"]),
        (&["external", "ip"], "show external IP address", "network",
            &["curl -s ifconfig.me", "dig +short myip.opendns.com @resolver1.opendns.com"]),
        // Gateway/router
        (&["default", "gateway"], "show default gateway", "network",
            &["ip route | grep default", "route -n | grep UG"]),
        (&["router", "ip"], "show router IP", "network",
            &["ip route | grep default | awk '{print $3}'"]),
        (&["gateway"], "show gateway", "network",
            &["ip route show default"]),
        // MAC address
        (&["mac", "address"], "show MAC address", "network",
            &["ip link show | grep ether", "cat /sys/class/net/*/address"]),
        // Network speed
        (&["network", "speed"], "check network speed", "network",
            &["ethtool eth0 2>/dev/null | grep Speed", "cat /sys/class/net/*/speed 2>/dev/null"]),
        (&["link", "speed"], "check link speed", "network",
            &["ethtool $(ip route | grep default | awk '{print $5}') 2>/dev/null | grep Speed"]),
        // Network statistics
        (&["network", "stat"], "show network statistics", "network",
            &["netstat -s 2>/dev/null | head -30", "ss -s"]),
        (&["network", "traffic"], "show network traffic", "network",
            &["cat /proc/net/dev", "ip -s link"]),
        (&["bandwidth", "usage"], "check bandwidth usage", "network",
            &["nethogs -t 2>/dev/null || echo 'Install nethogs: pacman -S nethogs'", "iftop -t -s 5 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// DNS related queries
fn match_dns(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // DNS servers
        (&["dns", "server"], "show DNS servers", "dns",
            &["cat /etc/resolv.conf", "resolvectl status 2>/dev/null | grep -i dns"]),
        (&["what", "dns"], "show configured DNS", "dns",
            &["cat /etc/resolv.conf", "nmcli device show | grep DNS"]),
        (&["nameserver"], "show nameservers", "dns",
            &["cat /etc/resolv.conf | grep nameserver"]),
        // DNS lookup
        (&["lookup", "dns"], "perform DNS lookup", "dns",
            &["nslookup <domain>", "dig <domain>"]),
        (&["resolve", "domain"], "resolve domain name", "dns",
            &["nslookup <domain>", "host <domain>"]),
        (&["dig", "domain"], "dig domain", "dns",
            &["dig <domain>", "dig <domain> +short"]),
        // DNS testing
        (&["test", "dns"], "test DNS resolution", "dns",
            &["nslookup google.com", "dig google.com +short", "resolvectl query google.com 2>/dev/null"]),
        (&["dns", "work"], "check if DNS is working", "dns",
            &["nslookup google.com 2>&1", "ping -c 1 google.com"]),
        // Flush DNS
        (&["flush", "dns"], "flush DNS cache", "dns",
            &["sudo resolvectl flush-caches", "sudo systemd-resolve --flush-caches"]),
        (&["clear", "dns", "cache"], "clear DNS cache", "dns",
            &["sudo resolvectl flush-caches", "resolvectl statistics"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// VPN related queries
fn match_vpn(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // VPN status
        (&["vpn", "status"], "check VPN status", "vpn",
            &["nmcli connection show --active | grep vpn", "ip addr show tun0 2>/dev/null"]),
        (&["vpn", "connect"], "check VPN connections", "vpn",
            &["nmcli connection show | grep vpn", "ip addr | grep -E 'tun|tap'"]),
        (&["vpn", "running"], "check if VPN is running", "vpn",
            &["pgrep -a openvpn 2>/dev/null", "systemctl status openvpn* 2>/dev/null | head -10"]),
        // List VPNs
        (&["list", "vpn"], "list VPN configurations", "vpn",
            &["nmcli connection show | grep vpn", "ls /etc/openvpn/*.conf 2>/dev/null"]),
        (&["available", "vpn"], "show available VPNs", "vpn",
            &["nmcli connection show | grep vpn"]),
        // WireGuard
        (&["wireguard", "status"], "check WireGuard status", "vpn",
            &["sudo wg show", "ip addr show wg0 2>/dev/null"]),
        (&["wg", "status"], "check WireGuard", "vpn",
            &["sudo wg show", "systemctl status wg-quick@wg0 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Ports and connections queries
fn match_ports(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // Open ports
        (&["open", "port"], "list open ports", "network",
            &["ss -tlnp", "netstat -tlnp 2>/dev/null || ss -tlnp"]),
        (&["listening", "port"], "show listening ports", "network",
            &["ss -tlnp", "lsof -i -P -n | grep LISTEN 2>/dev/null | head -20"]),
        (&["what", "port"], "show what ports are open", "network",
            &["ss -tlnp | head -20", "nmap localhost 2>/dev/null | head -30"]),
        // Port usage
        (&["using", "port"], "check what's using a port", "network",
            &["ss -tlnp | grep <port>", "lsof -i :<port> 2>/dev/null"]),
        (&["process", "port"], "find process using port", "network",
            &["ss -tlnp | grep <port>", "fuser <port>/tcp 2>/dev/null"]),
        // Active connections
        (&["active", "connect"], "show active connections", "network",
            &["ss -tp | head -20", "netstat -tp 2>/dev/null | head -20"]),
        (&["established", "connect"], "show established connections", "network",
            &["ss -tp state established", "netstat -tp 2>/dev/null | grep ESTABLISHED"]),
        // Network services
        (&["network", "service"], "list network services", "network",
            &["ss -tlnp", "systemctl list-units --type=socket --state=active"]),
        // Check specific port
        (&["port", "open", "check"], "check if port is open", "network",
            &["nc -zv localhost <port> 2>&1", "ss -tln | grep <port>"]),
        // Firewall ports
        (&["firewall", "port"], "check firewall ports", "network",
            &["sudo iptables -L -n | head -30", "sudo nft list ruleset 2>/dev/null | head -30"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Bandwidth and traffic monitoring patterns
fn match_bandwidth(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NetworkPattern] = &[
        // Bandwidth usage
        (&["bandwidth", "usage"], "show bandwidth usage", "network",
            &["nethogs -t -c 3 2>/dev/null || echo 'Install nethogs: pacman -S nethogs'",
              "iftop -t -s 3 2>/dev/null || echo 'Install iftop: pacman -S iftop'"]),
        (&["using", "bandwidth"], "show what's using bandwidth", "network",
            &["nethogs -t -c 3 2>/dev/null || ss -tp | head -20",
              "iftop -t -s 3 2>/dev/null || echo 'Install iftop'"]),
        (&["network", "usage"], "show network usage", "network",
            &["ss -s", "cat /proc/net/dev", "vnstat 2>/dev/null || echo 'Install vnstat for stats'"]),
        // Traffic monitoring
        (&["network", "traffic"], "show network traffic", "network",
            &["iftop -t -s 3 2>/dev/null || ss -tp | head -20", "vnstat -l 2>/dev/null"]),
        (&["traffic", "monitor"], "monitor network traffic", "network",
            &["iftop 2>/dev/null || nethogs 2>/dev/null || echo 'Install iftop or nethogs'"]),
        // Per-process network
        (&["process", "network"], "show per-process network usage", "network",
            &["nethogs -t -c 3 2>/dev/null || ss -tp | head -30"]),
        (&["which", "process", "network"], "show which process using network", "network",
            &["nethogs -t -c 3 2>/dev/null || ss -tp"]),
        // Download/upload speed
        (&["download", "speed"], "check download speed", "network",
            &["curl -o /dev/null -w 'Speed: %{speed_download} bytes/sec' https://speed.cloudflare.com/__down?bytes=10000000 2>/dev/null"]),
        (&["upload", "speed"], "check upload speed", "network",
            &["echo 'Use: speedtest-cli or fast-cli for speed test'"]),
        (&["network", "speed"], "check network speed", "network",
            &["speedtest-cli --simple 2>/dev/null || echo 'Install speedtest-cli'"]),
        // Data usage
        (&["data", "usage"], "show data usage", "network",
            &["vnstat 2>/dev/null || cat /proc/net/dev"]),
        (&["network", "stats"], "show network statistics", "network",
            &["ss -s", "vnstat 2>/dev/null || cat /proc/net/dev"]),
        // Interface bandwidth
        (&["interface", "bandwidth"], "show interface bandwidth", "network",
            &["cat /proc/net/dev", "ip -s link"]),
        // Real-time monitoring
        (&["live", "network"], "live network monitoring", "network",
            &["watch -n1 'cat /proc/net/dev'", "bmon 2>/dev/null || iftop 2>/dev/null"]),
        (&["realtime", "network"], "realtime network monitoring", "network",
            &["iftop 2>/dev/null || bmon 2>/dev/null || watch 'ss -s'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status() {
        assert!(match_patterns("am i connected").is_some());
        assert!(match_patterns("wifi status").is_some());
        assert!(match_patterns("network interface").is_some());
    }

    #[test]
    fn test_network_config() {
        assert!(match_patterns("what is my ip").is_some());
        assert!(match_patterns("show public ip").is_some());
        assert!(match_patterns("default gateway").is_some());
        assert!(match_patterns("mac address").is_some());
    }

    #[test]
    fn test_dns() {
        assert!(match_patterns("dns servers").is_some());
        assert!(match_patterns("test dns").is_some());
        assert!(match_patterns("flush dns cache").is_some());
    }

    #[test]
    fn test_vpn() {
        assert!(match_patterns("vpn status").is_some());
        assert!(match_patterns("list vpn").is_some());
        assert!(match_patterns("wireguard status").is_some());
    }

    #[test]
    fn test_ports() {
        assert!(match_patterns("open ports").is_some());
        assert!(match_patterns("listening ports").is_some());
        assert!(match_patterns("active connections").is_some());
    }

    #[test]
    fn test_bandwidth() {
        assert!(match_patterns("bandwidth usage").is_some());
        assert!(match_patterns("using bandwidth").is_some());
        assert!(match_patterns("network traffic").is_some());
        assert!(match_patterns("network speed").is_some());
    }
}
