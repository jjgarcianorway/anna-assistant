//! Network department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_network_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::NetHealth,
        IntentMapping {
            intent: CanonicalIntent::NetHealth,
            department: Department::Network,
            required_probes: vec!["ip_addr", "ip_route"],
            optional_probes: vec!["nmcli_status", "ss_listen"],
            can_answer_from_probes: false, // "Health" needs synthesis
            description: "Network health overview",
        },
    );

    mappings.insert(
        CanonicalIntent::DnsHealth,
        IntentMapping {
            intent: CanonicalIntent::DnsHealth,
            department: Department::Network,
            required_probes: vec!["resolvectl_status"],
            optional_probes: vec!["dig_test", "getent_hosts_test"],
            can_answer_from_probes: true,
            description: "DNS resolution health",
        },
    );

    mappings.insert(
        CanonicalIntent::WifiStatus,
        IntentMapping {
            intent: CanonicalIntent::WifiStatus,
            department: Department::Network,
            required_probes: vec!["iw_link", "nmcli_wifi"],
            optional_probes: vec!["iwconfig"],
            can_answer_from_probes: true,
            description: "WiFi connection status",
        },
    );

    mappings.insert(
        CanonicalIntent::RouteStatus,
        IntentMapping {
            intent: CanonicalIntent::RouteStatus,
            department: Department::Network,
            required_probes: vec!["ip_route", "ip_rule"],
            optional_probes: vec!["traceroute_gateway"],
            can_answer_from_probes: true,
            description: "Routing table status",
        },
    );
}
