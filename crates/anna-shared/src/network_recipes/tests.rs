//! Tests for network recipes (v0.0.462).

use super::*;

#[test]
fn test_detect_connectivity() {
    // "ping" is the key indicator
    assert_eq!(
        detect_feature("ping google.com"),
        Some(NetworkFeature::TestConnectivity)
    );
    assert_eq!(
        detect_feature("test network connection"),
        Some(NetworkFeature::TestConnectivity)
    );
}

#[test]
fn test_detect_dns() {
    // Must include "dns" indicator for query to be recognized
    assert_eq!(
        detect_feature("dns lookup for domain"),
        Some(NetworkFeature::DnsLookup)
    );
    assert_eq!(
        detect_feature("dns name resolution"),
        Some(NetworkFeature::DnsLookup)
    );
}

#[test]
fn test_detect_traceroute() {
    assert_eq!(
        detect_feature("traceroute to server"),
        Some(NetworkFeature::TraceRoute)
    );
    assert_eq!(
        detect_feature("trace network path"),
        Some(NetworkFeature::TraceRoute)
    );
}

#[test]
fn test_detect_ports() {
    assert_eq!(
        detect_feature("check port 80 open"),
        Some(NetworkFeature::CheckPorts)
    );
    assert_eq!(
        detect_feature("nmap port scan"),
        Some(NetworkFeature::CheckPorts)
    );
}

#[test]
fn test_detect_interface() {
    assert_eq!(
        detect_feature("show network interface info"),
        Some(NetworkFeature::InterfaceInfo)
    );
    assert_eq!(
        detect_feature("my ip address"),
        Some(NetworkFeature::InterfaceInfo)
    );
}

#[test]
fn test_detect_bandwidth() {
    assert_eq!(
        detect_feature("network speed test"),
        Some(NetworkFeature::BandwidthTest)
    );
    assert_eq!(
        detect_feature("check bandwidth usage"),
        Some(NetworkFeature::BandwidthTest)
    );
}

#[test]
fn test_detect_firewall() {
    assert_eq!(
        detect_feature("check firewall status"),
        Some(NetworkFeature::FirewallStatus)
    );
    assert_eq!(
        detect_feature("firewall iptables rules"),
        Some(NetworkFeature::FirewallStatus)
    );
}

#[test]
fn test_detect_listening() {
    assert_eq!(
        detect_feature("show listening port"),
        Some(NetworkFeature::ListeningPorts)
    );
    assert_eq!(
        detect_feature("netstat command"),
        Some(NetworkFeature::ListeningPorts)
    );
}

#[test]
fn test_detect_ssl() {
    assert_eq!(
        detect_feature("check ssl certificate"),
        Some(NetworkFeature::SslCertCheck)
    );
    assert_eq!(
        detect_feature("certificate expire date"),
        Some(NetworkFeature::SslCertCheck)
    );
}

#[test]
fn test_detect_http() {
    assert_eq!(
        detect_feature("curl http request"),
        Some(NetworkFeature::HttpTest)
    );
    assert_eq!(
        detect_feature("wget test"),
        Some(NetworkFeature::HttpTest)
    );
}

#[test]
fn test_detect_arp() {
    assert_eq!(
        detect_feature("show arp table"),
        Some(NetworkFeature::ArpTable)
    );
    assert_eq!(
        detect_feature("arp cache entries"),
        Some(NetworkFeature::ArpTable)
    );
}

#[test]
fn test_detect_routing() {
    assert_eq!(
        detect_feature("show routing table"),
        Some(NetworkFeature::RoutingTable)
    );
    assert_eq!(
        detect_feature("check default gateway"),
        Some(NetworkFeature::RoutingTable)
    );
}

#[test]
fn test_detect_stats() {
    assert_eq!(
        detect_feature("network stat traffic"),
        Some(NetworkFeature::NetworkStats)
    );
    assert_eq!(
        detect_feature("network packet statistics"),
        Some(NetworkFeature::NetworkStats)
    );
}

#[test]
fn test_detect_wifi() {
    assert_eq!(
        detect_feature("wifi signal strength"),
        Some(NetworkFeature::WifiDiagnostics)
    );
    assert_eq!(
        detect_feature("wireless connection"),
        Some(NetworkFeature::WifiDiagnostics)
    );
}

#[test]
fn test_detect_vpn() {
    assert_eq!(
        detect_feature("vpn status check"),
        Some(NetworkFeature::VpnStatus)
    );
    assert_eq!(
        detect_feature("wireguard connection"),
        Some(NetworkFeature::VpnStatus)
    );
}

#[test]
fn test_not_network_query() {
    assert_eq!(detect_feature("how much disk space"), None);
    assert_eq!(detect_feature("install htop"), None);
    assert_eq!(detect_feature("kubernetes pods"), None);
    assert_eq!(detect_feature("backup database"), None);
}

#[test]
fn test_match_query_returns_recipe() {
    let recipe = match_query("ping test connectivity");
    assert!(recipe.is_some());
    let recipe = recipe.unwrap();
    assert_eq!(recipe.feature, NetworkFeature::TestConnectivity);
    assert!(!recipe.commands.is_empty());
    assert!(!recipe.answer_template.is_empty());
}

#[test]
fn test_all_features_have_recipes() {
    let recipes = builtin_recipes();
    let features: Vec<NetworkFeature> = recipes.iter().map(|r| r.feature).collect();

    assert!(features.contains(&NetworkFeature::TestConnectivity));
    assert!(features.contains(&NetworkFeature::DnsLookup));
    assert!(features.contains(&NetworkFeature::TraceRoute));
    assert!(features.contains(&NetworkFeature::CheckPorts));
    assert!(features.contains(&NetworkFeature::InterfaceInfo));
    assert!(features.contains(&NetworkFeature::BandwidthTest));
    assert!(features.contains(&NetworkFeature::FirewallStatus));
    assert!(features.contains(&NetworkFeature::ListeningPorts));
    assert!(features.contains(&NetworkFeature::SslCertCheck));
    assert!(features.contains(&NetworkFeature::HttpTest));
    assert!(features.contains(&NetworkFeature::ArpTable));
    assert!(features.contains(&NetworkFeature::RoutingTable));
    assert!(features.contains(&NetworkFeature::NetworkStats));
    assert!(features.contains(&NetworkFeature::WifiDiagnostics));
    assert!(features.contains(&NetworkFeature::VpnStatus));
}

#[test]
fn test_feature_display_names() {
    assert_eq!(
        NetworkFeature::TestConnectivity.display_name(),
        "test connectivity"
    );
    assert_eq!(NetworkFeature::DnsLookup.display_name(), "DNS lookup");
    assert_eq!(
        NetworkFeature::FirewallStatus.display_name(),
        "firewall status"
    );
}

#[test]
fn test_recipe_builder() {
    let recipe = NetworkRecipe::new(NetworkFeature::TestConnectivity, "Test")
        .with_command("ping -c 4 8.8.8.8")
        .with_example("ping google.com")
        .with_answer("test answer")
        .with_note("test note")
        .with_requires("iputils");

    assert_eq!(recipe.feature, NetworkFeature::TestConnectivity);
    assert_eq!(recipe.commands, vec!["ping -c 4 8.8.8.8"]);
    assert_eq!(recipe.example, Some("ping google.com".to_string()));
    assert_eq!(recipe.answer_template, "test answer");
    assert_eq!(recipe.notes, vec!["test note"]);
    assert!(recipe.requires.contains(&"iputils".to_string()));
}

#[test]
fn test_dns_recipe_has_multiple_commands() {
    let recipes = builtin_recipes();
    let dns_recipe = recipes
        .iter()
        .find(|r| r.feature == NetworkFeature::DnsLookup)
        .unwrap();

    assert!(dns_recipe.commands.len() >= 3);
    assert!(dns_recipe.commands.iter().any(|c| c.contains("dig")));
    assert!(dns_recipe.commands.iter().any(|c| c.contains("nslookup")));
}

#[test]
fn test_ssl_recipe_uses_openssl() {
    let recipes = builtin_recipes();
    let ssl_recipe = recipes
        .iter()
        .find(|r| r.feature == NetworkFeature::SslCertCheck)
        .unwrap();

    assert!(ssl_recipe.commands.iter().any(|c| c.contains("openssl")));
}
