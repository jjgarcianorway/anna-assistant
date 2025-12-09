//! Network answer functions (v0.0.175).
//!
//! DNS, gateway, connectivity, routes, ARP, bonding, namespaces, stats.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer network connectivity query using ping
pub fn answer_network_connectivity(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ping_check")?;

    let answer = if probe.exit_code == 0 {
        let output = probe.stdout.trim();
        let latency = output
            .lines()
            .find(|line| line.contains("time="))
            .and_then(|line| line.split("time=").nth(1).and_then(|s| s.split_whitespace().next()));

        if let Some(lat) = latency {
            format!("Online - ping to 8.8.8.8: {} ms", lat)
        } else {
            "Online - network connectivity confirmed".to_string()
        }
    } else {
        "Offline - cannot reach 8.8.8.8 (Google DNS)".to_string()
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer DNS servers query
pub fn answer_dns_servers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "dns_servers")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No DNS servers configured in /etc/resolv.conf".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let mut servers = Vec::new();
    for line in output.lines() {
        if let Some(ip) = line.strip_prefix("nameserver ") {
            servers.push(ip.trim());
        }
    }

    let answer = if servers.is_empty() {
        "No DNS servers configured.".to_string()
    } else {
        format!("DNS servers: {}", servers.join(", "))
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: servers.len(),
        route_class: route_class.to_string(),
    })
}

/// Answer default gateway query
pub fn answer_default_gateway(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "default_gateway")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No default gateway configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let parts: Vec<&str> = output.split_whitespace().collect();
    let gateway = parts.get(2).unwrap_or(&"unknown");
    let interface = parts
        .iter()
        .position(|&p| p == "dev")
        .and_then(|i| parts.get(i + 1))
        .unwrap_or(&"unknown");

    Some(DeterministicResult {
        answer: format!("Default gateway: {} (via {})", gateway, interface),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer listening ports query using ss
pub fn answer_listening_ports(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "listening_ports")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No listening ports found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let port_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Listening ports ({}):\n{}", port_count, output),
        grounded: true,
        parsed_data_count: port_count,
        route_class: route_class.to_string(),
    })
}

/// Answer IP routes query
pub fn answer_ip_routes(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ip_routes")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No IP routes found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let route_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("IP routing table ({} routes):\n```\n{}\n```", route_count, output),
        grounded: true,
        parsed_data_count: route_count,
        route_class: route_class.to_string(),
    })
}

/// Answer ARP table query
pub fn answer_arp_table(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "arp_table")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No ARP entries found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let entry_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("ARP table ({} entries):\n```\n{}\n```", entry_count, output),
        grounded: true,
        parsed_data_count: entry_count,
        route_class: route_class.to_string(),
    })
}

/// Answer network namespaces query
pub fn answer_network_namespaces(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "network_namespaces")?;

    let output = probe.stdout.trim();
    if output.contains("No network namespaces") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No network namespaces configured.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let ns_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Network namespaces ({}):\n{}", ns_count, output),
        grounded: true,
        parsed_data_count: ns_count,
        route_class: route_class.to_string(),
    })
}

/// Answer network bonding query
pub fn answer_network_bonding(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "network_bonding")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.contains("No network bonding") || output.is_empty() {
        ("No network bonding configured on this system.".to_string(), 0)
    } else {
        (format!("Network bonding status:\n```\n{}\n```", output), 1)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer network statistics query using /proc/net/dev
pub fn answer_network_stats(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "network_stats")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("Network statistics not available.".to_string(), 0)
    } else {
        let mut interfaces: Vec<String> = Vec::new();
        for line in output.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let iface = parts[0].trim_end_matches(':');
                let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                let rx_mb = rx_bytes as f64 / 1_000_000.0;
                let tx_mb = tx_bytes as f64 / 1_000_000.0;
                interfaces.push(format!("  {}: RX {:.1}MB, TX {:.1}MB", iface, rx_mb, tx_mb));
            }
        }
        if interfaces.is_empty() {
            ("No network interface statistics found.".to_string(), 0)
        } else {
            (format!("Network interface statistics:\n{}", interfaces.join("\n")), interfaces.len())
        }
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer hosts file query
pub fn answer_hosts_file(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "hosts_file")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No non-comment entries found in /etc/hosts.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("/etc/hosts ({} entries):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer wireless networks query
pub fn answer_wireless_networks(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "wireless_networks")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "WiFi scanning not available or no wireless interface found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let network_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Available wireless networks ({}):\n```\n{}\n```", network_count, output),
        grounded: true,
        parsed_data_count: network_count,
        route_class: route_class.to_string(),
    })
}
