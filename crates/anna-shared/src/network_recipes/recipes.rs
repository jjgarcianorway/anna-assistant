//! Network troubleshooting builtin recipes (v0.0.462).

use super::types::{NetworkFeature, NetworkRecipe};

/// Get all builtin network recipes
pub fn builtin_recipes() -> Vec<NetworkRecipe> {
    vec![
        // Connectivity
        NetworkRecipe::new(NetworkFeature::TestConnectivity, "Test network connectivity")
            .with_command("ping -c 4 8.8.8.8")
            .with_command("ping -c 4 google.com")
            .with_command("ping -c 4 -6 ipv6.google.com")
            .with_answer(
                "Test connectivity with `ping -c 4 <host>`. Try IP first (8.8.8.8) to rule out DNS. \
                 If IP works but domain fails, check DNS. Use `-6` for IPv6 test.",
            )
            .with_note("ping 8.8.8.8 tests basic internet; ping domain tests DNS too"),
        // DNS
        NetworkRecipe::new(NetworkFeature::DnsLookup, "DNS lookup and diagnostics")
            .with_command("dig example.com")
            .with_command("dig +short example.com")
            .with_command("nslookup example.com")
            .with_command("dig @8.8.8.8 example.com")
            .with_command("host example.com")
            .with_answer(
                "DNS lookup: `dig example.com` (detailed) or `dig +short` (just IP). \
                 `nslookup` also works. Use `dig @8.8.8.8` to test specific DNS server.",
            )
            .with_note("Compare local DNS vs public (8.8.8.8) to diagnose DNS issues")
            .with_requires("bind-tools"),
        // Traceroute
        NetworkRecipe::new(NetworkFeature::TraceRoute, "Trace network path to host")
            .with_command("traceroute example.com")
            .with_command("tracepath example.com")
            .with_command("mtr example.com")
            .with_command("traceroute -I example.com")
            .with_answer(
                "Trace route: `traceroute <host>` shows each hop to destination. \
                 `mtr` combines ping and traceroute for continuous monitoring. \
                 Use `-I` to use ICMP instead of UDP.",
            )
            .with_note("High latency at a specific hop indicates bottleneck")
            .with_requires("traceroute"),
        // Port checking
        NetworkRecipe::new(NetworkFeature::CheckPorts, "Check if ports are open")
            .with_command("nc -zv host 80")
            .with_command("nmap -p 80,443 host")
            .with_command("telnet host 80")
            .with_command("nmap -sT -p 1-1000 host")
            .with_answer(
                "Check port: `nc -zv host port` (quick test). \
                 `nmap -p 80,443 host` for multiple ports. \
                 `telnet host port` works but nc/nmap are better.",
            )
            .with_note("Open = connection established; Closed = refused; Filtered = no response")
            .with_requires("nmap"),
        // Interface info
        NetworkRecipe::new(NetworkFeature::InterfaceInfo, "View network interface info")
            .with_command("ip addr")
            .with_command("ip -c addr")
            .with_command("ip link show")
            .with_command("ifconfig")
            .with_command("nmcli device status")
            .with_answer(
                "View interfaces: `ip addr` (modern) or `ifconfig` (legacy). \
                 `ip -c addr` adds color. `nmcli device status` shows NetworkManager view.",
            )
            .with_note("Look for UP/DOWN state and IP addresses"),
        // Bandwidth
        NetworkRecipe::new(NetworkFeature::BandwidthTest, "Test network bandwidth")
            .with_command("speedtest-cli")
            .with_command("iperf3 -c speedtest.serverius.net")
            .with_command("curl -o /dev/null -w '%{speed_download}' http://speedtest.tele2.net/10MB.zip")
            .with_answer(
                "Bandwidth test: `speedtest-cli` for internet speed. \
                 `iperf3 -c server` for LAN testing (needs server). \
                 curl download test shows actual throughput.",
            )
            .with_note("Run multiple times; results vary with server load")
            .with_requires("speedtest-cli"),
        // Firewall
        NetworkRecipe::new(NetworkFeature::FirewallStatus, "Check firewall status and rules")
            .with_command("sudo iptables -L -n -v")
            .with_command("sudo nft list ruleset")
            .with_command("sudo ufw status verbose")
            .with_command("sudo firewall-cmd --list-all")
            .with_answer(
                "Firewall status: `iptables -L -n -v` (legacy), `nft list ruleset` (modern). \
                 Ubuntu: `ufw status`. Fedora/RHEL: `firewall-cmd --list-all`.",
            )
            .with_note("Check ACCEPT/DROP policies for INPUT, OUTPUT, FORWARD chains"),
        // Listening ports
        NetworkRecipe::new(NetworkFeature::ListeningPorts, "Show listening ports")
            .with_command("ss -tulnp")
            .with_command("netstat -tulnp")
            .with_command("lsof -i -P -n")
            .with_command("ss -tulnp | grep LISTEN")
            .with_answer(
                "Listening ports: `ss -tulnp` (modern) or `netstat -tulnp`. \
                 -t=TCP, -u=UDP, -l=listening, -n=numeric, -p=process. \
                 `lsof -i` shows all network connections.",
            )
            .with_note("Run as root to see process names"),
        // SSL certificate
        NetworkRecipe::new(NetworkFeature::SslCertCheck, "Check SSL/TLS certificate")
            .with_command("openssl s_client -connect host:443 -servername host </dev/null")
            .with_command("echo | openssl s_client -connect host:443 2>/dev/null | openssl x509 -noout -dates")
            .with_command("curl -vI https://example.com 2>&1 | grep -i 'expire'")
            .with_answer(
                "SSL check: `openssl s_client -connect host:443` shows cert chain. \
                 Add `| openssl x509 -noout -dates` to see expiry dates. \
                 Use `-servername` for SNI hosts.",
            )
            .with_note("Check 'Not After' date for expiration")
            .with_requires("openssl"),
        // HTTP test
        NetworkRecipe::new(NetworkFeature::HttpTest, "Test HTTP/HTTPS requests")
            .with_command("curl -I https://example.com")
            .with_command("curl -v https://example.com")
            .with_command("curl -w '%{http_code}' -o /dev/null -s https://example.com")
            .with_command("wget --spider https://example.com")
            .with_answer(
                "HTTP test: `curl -I` for headers only, `curl -v` for verbose. \
                 `-w '%{http_code}'` shows just status code. \
                 `wget --spider` checks without downloading.",
            )
            .with_note("-k flag skips SSL verification (testing only)"),
        // ARP
        NetworkRecipe::new(NetworkFeature::ArpTable, "View ARP table")
            .with_command("ip neigh")
            .with_command("arp -a")
            .with_command("ip neigh show")
            .with_answer(
                "ARP table: `ip neigh` (modern) or `arp -a` (legacy). \
                 Shows IP-to-MAC mappings for local network devices. \
                 STALE/REACHABLE indicates cache freshness.",
            )
            .with_note("Clear entry: ip neigh del <ip> dev <interface>"),
        // Routing
        NetworkRecipe::new(NetworkFeature::RoutingTable, "View and manage routing table")
            .with_command("ip route")
            .with_command("ip route show")
            .with_command("route -n")
            .with_command("ip route get 8.8.8.8")
            .with_answer(
                "Routing table: `ip route` shows all routes. \
                 `ip route get <ip>` shows which route a packet would take. \
                 'default via X' is the default gateway.",
            )
            .with_note("Missing default route = no internet connectivity"),
        // Network stats
        NetworkRecipe::new(NetworkFeature::NetworkStats, "View network statistics")
            .with_command("ip -s link")
            .with_command("netstat -s")
            .with_command("ss -s")
            .with_command("cat /proc/net/dev")
            .with_answer(
                "Network stats: `ip -s link` shows TX/RX bytes per interface. \
                 `netstat -s` or `ss -s` shows protocol statistics. \
                 Watch for high error/drop counts.",
            )
            .with_note("High dropped packets may indicate congestion or MTU issues"),
        // WiFi
        NetworkRecipe::new(NetworkFeature::WifiDiagnostics, "WiFi diagnostics")
            .with_command("iwconfig")
            .with_command("nmcli device wifi list")
            .with_command("iw dev wlan0 link")
            .with_command("iw dev wlan0 scan")
            .with_command("nmcli radio wifi")
            .with_answer(
                "WiFi diagnostics: `iwconfig` shows connection info. \
                 `nmcli device wifi list` shows available networks. \
                 `iw dev wlan0 link` shows signal strength and bitrate.",
            )
            .with_note("Signal: >-50dBm excellent, -50 to -60 good, -60 to -70 fair")
            .with_requires("wireless_tools"),
        // VPN
        NetworkRecipe::new(NetworkFeature::VpnStatus, "Check VPN status")
            .with_command("ip link show type wireguard")
            .with_command("wg show")
            .with_command("systemctl status openvpn")
            .with_command("nmcli connection show --active")
            .with_answer(
                "VPN status: WireGuard: `wg show`. OpenVPN: `systemctl status openvpn@<config>`. \
                 NetworkManager: `nmcli connection show --active`. \
                 Check `ip route` for VPN routes.",
            )
            .with_note("VPN should add routes for tunneled traffic"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipes_exist() {
        let recipes = builtin_recipes();
        assert!(!recipes.is_empty());
    }

    #[test]
    fn test_recipes_have_commands() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.commands.is_empty(),
                "Recipe {:?} has no commands",
                recipe.feature
            );
        }
    }

    #[test]
    fn test_recipes_have_answers() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.answer_template.is_empty(),
                "Recipe {:?} has no answer",
                recipe.feature
            );
        }
    }
}
