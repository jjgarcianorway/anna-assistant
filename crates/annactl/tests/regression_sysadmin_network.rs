//! Regression tests for network diagnostics (Beta.265)
//!
//! Tests cover:
//! - Interface collision detection (Ethernet vs WiFi)
//! - Route mismatch scenarios
//! - Priority inversion (slow Ethernet outranking fast WiFi)
//! - Connectivity degradation (packet loss, latency, DNS)
//! - Deterministic answer generation

use annactl::net_diagnostics::*;
use annactl::sysadmin_answers::{compose_network_conflict_answer, compose_network_routing_answer};
use anna_common::network_monitoring::*;

/// Helper: Create mock interface
fn mock_interface(
    name: &str,
    iface_type: InterfaceType,
    speed_mbps: Option<u32>,
    is_up: bool,
    has_errors: bool,
) -> NetworkInterface {
    NetworkInterface {
        name: name.to_string(),
        interface_type: iface_type,
        is_up,
        mac_address: Some("00:11:22:33:44:55".to_string()),
        ipv4_addresses: if is_up {
            vec!["192.168.1.100/24".to_string()]
        } else {
            vec![]
        },
        ipv6_addresses: vec![],
        mtu: Some(1500),
        speed_mbps,
        config_method: AddressConfig::DHCP,
        stats: InterfaceStats {
            rx_bytes: 1000000,
            rx_packets: 1000,
            rx_errors: if has_errors { 50 } else { 0 },
            rx_dropped: 0,
            tx_bytes: 500000,
            tx_packets: 500,
            tx_errors: if has_errors { 25 } else { 0 },
            tx_dropped: 0,
        },
    }
}

/// Helper: Create mock route
fn mock_route(destination: &str, interface: &str, metric: Option<u32>) -> Route {
    Route {
        destination: destination.to_string(),
        gateway: Some("192.168.1.1".to_string()),
        interface: interface.to_string(),
        metric,
        protocol: Some("dhcp".to_string()),
    }
}

/// Helper: Create mock network monitoring data
fn mock_network_monitoring(
    interfaces: Vec<NetworkInterface>,
    routes: Vec<Route>,
    packet_loss: Option<f64>,
    latency: Option<f64>,
) -> NetworkMonitoring {
    NetworkMonitoring {
        interfaces,
        ipv4_status: IpVersionStatus {
            enabled: true,
            has_connectivity: true,
            default_gateway: Some("192.168.1.1".to_string()),
            address_count: 1,
        },
        ipv6_status: IpVersionStatus {
            enabled: false,
            has_connectivity: false,
            default_gateway: None,
            address_count: 0,
        },
        dnssec_status: DnssecStatus {
            enabled: false,
            resolver: None,
            validation_mode: None,
        },
        latency: LatencyMetrics {
            gateway_latency_ms: Some(1.0),
            dns_latency_ms: None,
            internet_latency_ms: latency,
            measurement_successful: true,
        },
        packet_loss: PacketLossStats {
            gateway_loss_percent: None,
            dns_loss_percent: None,
            internet_loss_percent: packet_loss,
            measurement_successful: true,
        },
        routes,
        firewall_rules: FirewallRules {
            firewall_type: Some("nftables".to_string()),
            is_active: true,
            rule_count: 10,
            default_input_policy: Some("DROP".to_string()),
            default_output_policy: Some("ACCEPT".to_string()),
            default_forward_policy: Some("DROP".to_string()),
            open_ports: vec![],
        },
    }
}

#[test]
fn test_slow_ethernet_outranks_fast_wifi() {
    // Scenario: 100 Mbps Ethernet taking priority over 866 Mbps WiFi
    let interfaces = vec![
        mock_interface("eth0", InterfaceType::Ethernet, Some(100), true, false),
        mock_interface("wlan0", InterfaceType::WiFi, Some(866), true, false),
    ];

    let routes = vec![mock_route("default", "eth0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect Ethernet slower than WiFi collision
    assert!(diagnostics.interface_collision.is_some());
    let collision = diagnostics.interface_collision.as_ref().unwrap();
    assert_eq!(collision.collision_type, CollisionType::EthernetSlowerThanWiFi);
    assert_eq!(collision.severity, DiagnosticSeverity::Critical);

    // Test answer composer
    let answer = compose_network_conflict_answer(&diagnostics);
    assert!(answer.contains("[SUMMARY]"));
    assert!(answer.contains("critical"));
    assert!(answer.contains("Ethernet is slower than WiFi"));
    assert!(answer.contains("[DETAILS]"));
    assert!(answer.contains("100 Mbps"));
    assert!(answer.contains("866 Mbps"));
    assert!(answer.contains("[COMMANDS]"));
    assert!(answer.contains("ethtool"));
}

#[test]
fn test_duplicate_default_routes() {
    // Scenario: Multiple default routes causing routing ambiguity
    let interfaces = vec![
        mock_interface("eth0", InterfaceType::Ethernet, Some(1000), true, false),
        mock_interface("wlan0", InterfaceType::WiFi, Some(300), true, false),
    ];

    let routes = vec![
        mock_route("default", "eth0", Some(100)),
        mock_route("default", "wlan0", Some(200)),
    ];

    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect duplicate default routes
    assert!(diagnostics.interface_collision.is_some());
    let collision = diagnostics.interface_collision.as_ref().unwrap();
    assert_eq!(collision.collision_type, CollisionType::DuplicateDefaultRoutes);
    assert_eq!(collision.severity, DiagnosticSeverity::Critical);

    // Test answer composer
    let answer = compose_network_conflict_answer(&diagnostics);
    assert!(answer.contains("[SUMMARY]"));
    assert!(answer.contains("Multiple default routes"));
    assert!(answer.contains("[COMMANDS]"));
    assert!(answer.contains("ip route"));
}

#[test]
fn test_priority_mismatch_detection() {
    // Scenario: Fast WiFi should have priority but slow Ethernet has it
    let interfaces = vec![
        mock_interface("eth0", InterfaceType::Ethernet, Some(100), true, false),
        mock_interface("wlan0", InterfaceType::WiFi, Some(866), true, false),
    ];

    let routes = vec![mock_route("default", "eth0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect priority mismatch (WiFi should be preferred)
    assert!(diagnostics.priority_mismatch.is_some());
    let mismatch = diagnostics.priority_mismatch.unwrap();
    assert_eq!(mismatch.expected_interface, "wlan0");
    assert_eq!(mismatch.actual_interface, "eth0");
    assert!(mismatch.expected_rank > mismatch.actual_rank);
}

#[test]
fn test_high_packet_loss_detection() {
    // Scenario: 25% packet loss to internet
    let interfaces = vec![mock_interface("wlan0", InterfaceType::WiFi, Some(300), true, false)];

    let routes = vec![mock_route("default", "wlan0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, Some(25.0), None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect high packet loss
    assert!(diagnostics.connectivity_degradation.is_some());
    let degradation = diagnostics.connectivity_degradation.as_ref().unwrap();
    assert_eq!(degradation.degradation_type, DegradationType::HighPacketLoss);
    assert_eq!(degradation.severity, DiagnosticSeverity::Warning);

    // Test routing answer composer
    let answer = compose_network_routing_answer(&diagnostics);
    assert!(answer.contains("[SUMMARY]"));
    assert!(answer.contains("degraded"));
    assert!(answer.contains("[DETAILS]"));
    assert!(answer.contains("25.0%"));
    assert!(answer.contains("[COMMANDS]"));
    assert!(answer.contains("ping"));
}

#[test]
fn test_high_latency_detection() {
    // Scenario: 350ms latency to internet
    let interfaces = vec![mock_interface("wlan0", InterfaceType::WiFi, Some(300), true, false)];

    let routes = vec![mock_route("default", "wlan0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, None, Some(350.0));
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect high latency
    assert!(diagnostics.connectivity_degradation.is_some());
    let degradation = diagnostics.connectivity_degradation.as_ref().unwrap();
    assert_eq!(degradation.degradation_type, DegradationType::HighLatency);
    assert_eq!(degradation.severity, DiagnosticSeverity::Warning);

    // Test answer
    let answer = compose_network_routing_answer(&diagnostics);
    assert!(answer.contains("350.0ms"));
}

#[test]
fn test_interface_error_rate_detection() {
    // Scenario: Interface with high error rate
    let interfaces = vec![mock_interface(
        "wlan0",
        InterfaceType::WiFi,
        Some(300),
        true,
        true, // has_errors = true
    )];

    let routes = vec![mock_route("default", "wlan0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect interface errors
    assert!(diagnostics.connectivity_degradation.is_some());
    let degradation = diagnostics.connectivity_degradation.unwrap();
    assert_eq!(
        degradation.degradation_type,
        DegradationType::InterfaceFlapping
    );
}

#[test]
fn test_healthy_single_interface() {
    // Scenario: Single gigabit Ethernet, no issues
    let interfaces = vec![mock_interface(
        "eth0",
        InterfaceType::Ethernet,
        Some(1000),
        true,
        false,
    )];

    let routes = vec![mock_route("default", "eth0", Some(100))];

    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should be healthy
    assert_eq!(diagnostics.health_status, NetworkHealthStatus::Healthy);
    assert!(diagnostics.interface_collision.is_none());
    assert!(diagnostics.connectivity_degradation.is_none());
    assert!(diagnostics.routing_issues.is_empty());

    // Test answer
    let answer = compose_network_conflict_answer(&diagnostics);
    assert!(answer.contains("[SUMMARY]"));
    assert!(answer.contains("none") || answer.contains("no interface collision"));
}

#[test]
fn test_missing_ipv6_route() {
    // Scenario: IPv4 default route exists but no IPv6 route
    let interfaces = vec![mock_interface(
        "eth0",
        InterfaceType::Ethernet,
        Some(1000),
        true,
        false,
    )];

    let routes = vec![mock_route("0.0.0.0/0", "eth0", Some(100))];

    let mut net_mon = mock_network_monitoring(interfaces, routes, None, None);
    net_mon.ipv6_status.enabled = true; // IPv6 enabled but no route

    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    // Should detect missing IPv6 route
    assert!(!diagnostics.routing_issues.is_empty());
    let has_ipv6_issue = diagnostics
        .routing_issues
        .iter()
        .any(|issue| matches!(issue.issue_type, RoutingIssueType::MissingIpv6Route));
    assert!(has_ipv6_issue);

    // Test answer
    let answer = compose_network_routing_answer(&diagnostics);
    assert!(answer.contains("IPv6"));
}

#[test]
fn test_answer_format_consistency() {
    // Verify all answers follow [SUMMARY] + [DETAILS] + [COMMANDS] format
    let interfaces = vec![mock_interface("eth0", InterfaceType::Ethernet, Some(100), true, false)];
    let routes = vec![mock_route("default", "eth0", Some(100))];
    let net_mon = mock_network_monitoring(interfaces, routes, None, None);
    let diagnostics = NetworkDiagnostics::analyze(&net_mon);

    let conflict_answer = compose_network_conflict_answer(&diagnostics);
    let routing_answer = compose_network_routing_answer(&diagnostics);

    // Both must have the three sections
    for answer in &[conflict_answer, routing_answer] {
        assert!(answer.contains("[SUMMARY]"), "Missing [SUMMARY] section");
        assert!(answer.contains("[DETAILS]"), "Missing [DETAILS] section");
        assert!(answer.contains("[COMMANDS]"), "Missing [COMMANDS] section");

        // SUMMARY must come first
        let summary_pos = answer.find("[SUMMARY]").unwrap();
        let details_pos = answer.find("[DETAILS]").unwrap();
        let commands_pos = answer.find("[COMMANDS]").unwrap();
        assert!(summary_pos < details_pos);
        assert!(details_pos < commands_pos);
    }
}

#[test]
fn test_commands_are_deterministic() {
    // Verify same input produces same commands
    let interfaces = vec![
        mock_interface("eth0", InterfaceType::Ethernet, Some(100), true, false),
        mock_interface("wlan0", InterfaceType::WiFi, Some(866), true, false),
    ];
    let routes = vec![mock_route("default", "eth0", Some(100))];

    let net_mon1 = mock_network_monitoring(interfaces.clone(), routes.clone(), None, None);
    let net_mon2 = mock_network_monitoring(interfaces, routes, None, None);

    let diagnostics1 = NetworkDiagnostics::analyze(&net_mon1);
    let diagnostics2 = NetworkDiagnostics::analyze(&net_mon2);

    let answer1 = compose_network_conflict_answer(&diagnostics1);
    let answer2 = compose_network_conflict_answer(&diagnostics2);

    // Should produce identical output
    assert_eq!(answer1, answer2);
}
