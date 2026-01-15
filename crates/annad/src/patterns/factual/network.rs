//! Network patterns.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use super::super::contains_word;
use super::FactualPattern;

pub fn match_network(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[FactualPattern] = &[
        // IP address
        (&["ip", "address"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["my", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["show", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        (&["what", "ip"], "IP address query", "network", &["ip -4 addr show | grep inet | grep -v 127.0.0.1"]),
        // Network interfaces
        (&["network", "interface"], "network interfaces query", "network", &["ip link show"]),
        (&["list", "interface"], "network interfaces query", "network", &["ip link show"]),
        // DNS
        (&["dns", "server"], "DNS server query", "network", &["resolvectl status | head -20"]),
        (&["nameserver"], "DNS server query", "network", &["cat /etc/resolv.conf"]),
        // Gateway
        (&["gateway"], "gateway query", "network", &["ip route | grep default"]),
        (&["default", "route"], "default route query", "network", &["ip route | grep default"]),
        // Connection status
        (&["network", "status"], "network status query", "network", &["nmcli general status"]),
        (&["connected", "network"], "network connection query", "network", &["nmcli connection show --active"]),
        // Ports
        (&["listening", "port"], "listening ports query", "network", &["ss -tlnp 2>/dev/null | head -20"]),
        (&["open", "port"], "open ports query", "network", &["ss -tlnp 2>/dev/null | head -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| contains_word(q, kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}
