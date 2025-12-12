//! Network query matching (v0.0.462).

use super::recipes::builtin_recipes;
use super::types::{NetworkFeature, NetworkRecipe};

/// Detect if a query is about network troubleshooting
pub fn detect_feature(query: &str) -> Option<NetworkFeature> {
    let lower = query.to_lowercase();

    // First check if it's even a network query
    if !is_network_query(&lower) {
        return None;
    }

    // Find all matching keywords and return the feature with the longest match
    let mut best_match: Option<(NetworkFeature, usize)> = None;

    for feature in all_features() {
        for keyword in feature.keywords() {
            if lower.contains(keyword) {
                let keyword_len = keyword.len();
                if best_match.is_none() || keyword_len > best_match.unwrap().1 {
                    best_match = Some((feature, keyword_len));
                }
            }
        }
    }

    best_match.map(|(f, _)| f)
}

/// Match a query to a recipe
pub fn match_query(query: &str) -> Option<NetworkRecipe> {
    let feature = detect_feature(query)?;

    builtin_recipes()
        .into_iter()
        .find(|r| r.feature == feature)
}

/// Check if query is about network troubleshooting
fn is_network_query(query: &str) -> bool {
    let network_indicators = [
        "network",
        "ping",
        "dns",
        "traceroute",
        "port",
        "firewall",
        "connectivity",
        "connection",
        "wifi",
        "wireless",
        "ip address",
        "interface",
        "bandwidth",
        "speed test",
        "netstat",
        "routing",
        "gateway",
        "ssl",
        "certificate",
        "curl",
        "wget",
        "vpn",
        "wireguard",
        "nmap",
        "arp",
    ];

    network_indicators.iter().any(|k| query.contains(k))
}

/// Get all network features
fn all_features() -> Vec<NetworkFeature> {
    vec![
        NetworkFeature::TestConnectivity,
        NetworkFeature::DnsLookup,
        NetworkFeature::TraceRoute,
        NetworkFeature::CheckPorts,
        NetworkFeature::InterfaceInfo,
        NetworkFeature::BandwidthTest,
        NetworkFeature::FirewallStatus,
        NetworkFeature::ListeningPorts,
        NetworkFeature::SslCertCheck,
        NetworkFeature::HttpTest,
        NetworkFeature::ArpTable,
        NetworkFeature::RoutingTable,
        NetworkFeature::NetworkStats,
        NetworkFeature::WifiDiagnostics,
        NetworkFeature::VpnStatus,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ping() {
        assert_eq!(
            detect_feature("ping test network"),
            Some(NetworkFeature::TestConnectivity)
        );
    }

    #[test]
    fn test_detect_dns() {
        assert_eq!(
            detect_feature("dns lookup for domain"),
            Some(NetworkFeature::DnsLookup)
        );
    }

    #[test]
    fn test_detect_traceroute() {
        assert_eq!(
            detect_feature("traceroute to server"),
            Some(NetworkFeature::TraceRoute)
        );
    }

    #[test]
    fn test_not_network_query() {
        assert_eq!(detect_feature("how much disk space"), None);
        assert_eq!(detect_feature("install htop"), None);
    }

    #[test]
    fn test_match_query() {
        let recipe = match_query("ping test connectivity");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, NetworkFeature::TestConnectivity);
    }
}
