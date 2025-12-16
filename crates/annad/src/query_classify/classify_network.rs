//! Network query classification patterns (v0.0.803).
//!
//! Interfaces, ports, DNS, gateway, connectivity, wireless, bonding.

use crate::router::QueryClass;

/// Classify network queries.
/// Returns Some if matched, None otherwise.
pub fn classify_network(q: &str) -> Option<QueryClass> {
    // v0.0.124: Network connectivity - MUST come before NetworkInterfaces
    // v0.0.803: Added "network status" pattern and reordered
    if q.contains("am i online")
        || q.contains("internet connection")
        || q.contains("check internet")
        || q.contains("network connectivity")
        || q.contains("network status")
        || q.contains("connected to internet")
        || q.contains("online?")
        || q.contains("can i reach")
        || (q.contains("ping") && !q.contains("pinging"))
    {
        return Some(QueryClass::NetworkConnectivity);
    }

    // Network interfaces
    if q.contains("network")
        || q.contains("interface")
        || q.contains("ip ")
        || q.contains("ip?")
        || q.contains("ips")
        || q.contains("wifi")
        || q.contains("ethernet")
        || q.contains("wlan")
    {
        return Some(QueryClass::NetworkInterfaces);
    }

    // v0.0.125: Listening ports
    // v0.0.789: Added "using port" pattern for queries like "what's using port 3000"
    if q.contains("listening port")
        || q.contains("open port")
        || q.contains("port listen")
        || q.contains("network port")
        || q.contains("what ports")
        || q.contains("using port")
        || q.contains("on port")
        || q.trim() == "ss"
        || q.trim() == "netstat"
        || (q.contains("port") && q.contains("open"))
        || (q.contains("port") && q.contains("3000"))
        || (q.contains("port") && q.contains("8080"))
        || (q.contains("port") && q.contains("80"))
        || (q.contains("port") && q.contains("443"))
    {
        return Some(QueryClass::ListeningPorts);
    }

    // v0.0.126: DNS servers
    if q.contains("dns server")
        || q.contains("nameserver")
        || q.contains("resolv.conf")
        || q.contains("dns config")
        || (q.contains("what") && q.contains("dns"))
        || (q.contains("which") && q.contains("dns"))
    {
        return Some(QueryClass::DnsServers);
    }

    // v0.0.126: Default gateway
    if q.contains("default gateway")
        || q.contains("gateway ip")
        || q.contains("default route")
        || (q.contains("what") && q.contains("gateway"))
        || (q.contains("my") && q.contains("gateway"))
        || q.trim() == "gateway"
    {
        return Some(QueryClass::DefaultGateway);
    }

    // v0.0.130: Network namespaces
    if q.contains("network namespace")
        || q.contains("netns")
        || q.trim() == "ip netns"
        || (q.contains("namespace") && q.contains("network"))
    {
        return Some(QueryClass::NetworkNamespaces);
    }

    // v0.0.132: IP routes
    if q.contains("ip route")
        || q.contains("routing table")
        || q.trim() == "route"
        || (q.contains("show") && q.contains("route"))
        || (q.contains("list") && q.contains("route"))
    {
        return Some(QueryClass::IpRoutes);
    }

    // v0.0.132: ARP table
    if q.contains("arp table")
        || q.contains("arp cache")
        || q.trim() == "arp"
        || q.contains("ip neigh")
        || (q.contains("neighbor") && q.contains("cache"))
    {
        return Some(QueryClass::ArpTable);
    }

    // v0.0.135: Wireless networks
    if q.contains("wifi network")
        || q.contains("wireless network")
        || q.contains("available network")
        || q.contains("wifi scan")
        || (q.contains("show") && q.contains("wifi"))
    {
        return Some(QueryClass::WirelessNetworks);
    }

    // v0.0.139: Network bonding
    if q.contains("network bond")
        || q.contains("bond interface")
        || q.contains("bond0")
        || q.contains("link aggregation")
        || q.contains("lacp")
        || (q.contains("ethernet") && q.contains("bond"))
    {
        return Some(QueryClass::NetworkBonding);
    }

    // v0.0.141: Network stats
    if q.contains("network stat")
        || q.contains("rx bytes")
        || q.contains("tx bytes")
        || q.contains("packet count")
        || q.contains("network traffic")
        || (q.contains("interface") && q.contains("statistic"))
    {
        return Some(QueryClass::NetworkStats);
    }

    None
}
