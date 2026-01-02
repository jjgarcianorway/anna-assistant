//! Network-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn network_rules() -> Vec<ProbeRule> {
    vec![
        ProbeRule {
            intent_id: "network.ip",
            keywords: &["ip", "address"],
            negative_keywords: &[],
            probes: &["network_addrs"],
            description: "IP addresses",
        },
        ProbeRule {
            intent_id: "network.ports",
            keywords: &["listening", "port"],
            negative_keywords: &[],
            probes: &["listening_ports"],
            description: "Listening ports",
        },
        ProbeRule {
            intent_id: "network.dns",
            keywords: &["dns"],
            negative_keywords: &[],
            probes: &["dns_servers"],
            description: "DNS servers",
        },
        ProbeRule {
            intent_id: "network.connection",
            keywords: &["internet", "connection"],
            negative_keywords: &[],
            probes: &["ping_check", "network_addrs", "dns_servers"],
            description: "Internet connection status",
        },
        ProbeRule {
            intent_id: "network.wifi",
            keywords: &["wifi"],
            negative_keywords: &[],
            probes: &["wireless_networks", "network_addrs"],
            description: "WiFi status",
        },
    ]
}
