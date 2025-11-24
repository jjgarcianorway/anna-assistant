//! Regression tests for network remediation (Beta.268)
//!
//! Tests cover:
//! - Priority mismatch remediation routing
//! - Duplicate route remediation routing
//! - Packet loss quality remediation routing
//! - Latency quality remediation routing
//! - Interface error remediation routing
//! - Healthy network (no remediation)
//! - Canonical format validation
//! - Deterministic routing from brain insights

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use annactl::sysadmin_answers::{
    compose_network_priority_fix_answer,
    compose_network_route_fix_answer,
    compose_network_quality_fix_answer,
    route_network_remediation,
};

/// Helper: Create brain analysis with specific network insight
fn create_brain_with_insight(
    rule_id: &str,
    severity: &str,
    summary: &str,
    details: &str,
    evidence: &str,
) -> BrainAnalysisData {
    BrainAnalysisData {
        insights: vec![DiagnosticInsightData {
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
            summary: summary.to_string(),
            details: details.to_string(),
            commands: vec![],
            citations: vec![],
            evidence: evidence.to_string(),
        }],
        timestamp: chrono::Utc::now().to_rfc3339(),
        formatted_output: String::new(),
        critical_count: if severity == "critical" { 1 } else { 0 },
        warning_count: if severity == "warning" { 1 } else { 0 },
        proactive_issues: vec![],
        proactive_health_score: 100,
    }
}

#[test]
fn test_priority_mismatch_routes_to_correct_composer() {
    let brain = create_brain_with_insight(
        "network_priority_mismatch",
        "critical",
        "Slow Ethernet (100 Mbps) taking priority over faster WiFi (866 Mbps)",
        "Interface eth0 (Ethernet, 100 Mbps) is currently taking routing priority over wlan0 (WiFi, 866 Mbps).",
        "Ethernet eth0 (100 Mbps) has default route, WiFi wlan0 (866 Mbps) does not",
    );

    let result = route_network_remediation(&brain);
    assert!(result.is_some(), "Should route priority mismatch to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Network priority issue detected"), "Should have correct summary");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("eth0"), "Should mention Ethernet interface");
    assert!(answer.contains("100 Mbps"), "Should mention Ethernet speed");
    assert!(answer.contains("wlan0"), "Should mention WiFi interface");
    assert!(answer.contains("866 Mbps"), "Should mention WiFi speed");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("nmcli connection down eth0"), "Should have disconnect command");
}

#[test]
fn test_duplicate_routes_routes_to_correct_composer() {
    let brain = create_brain_with_insight(
        "duplicate_default_routes",
        "critical",
        "2 duplicate default routes detected",
        "Multiple default routes exist on interfaces: eth0, wlan0. This can cause unpredictable routing behavior.",
        "Both eth0 and wlan0 have default routes",
    );

    let result = route_network_remediation(&brain);
    assert!(result.is_some(), "Should route duplicate routes to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Duplicate default routes detected"), "Should have correct summary");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("unpredictable"), "Should explain routing issue");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("ip route del default"), "Should have route deletion command");
    assert!(answer.contains("systemctl restart NetworkManager"), "Should have NetworkManager restart");
}

#[test]
fn test_missing_route_routes_to_correct_composer() {
    let brain = create_brain_with_insight(
        "missing_default_route",
        "critical",
        "No default route configured",
        "System has no default route, external connectivity will fail.",
        "No default route found in routing table",
    );

    // Note: missing_default_route not in current Beta.267, but testing composer directly
    let answer = compose_network_route_fix_answer("missing", vec![]);

    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Routing configuration issue detected"), "Should have correct summary");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("missing default route"), "Should explain issue");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("systemctl restart NetworkManager"), "Should have NetworkManager restart");
}

#[test]
fn test_packet_loss_routes_to_quality_composer() {
    let brain = create_brain_with_insight(
        "high_packet_loss",
        "critical",
        "High packet loss detected: 35%",
        "Network connection experiencing significant packet loss.",
        "35% packet loss to internet",
    );

    let result = route_network_remediation(&brain);
    assert!(result.is_some(), "Should route packet loss to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("packet loss"), "Should mention packet loss");
    assert!(answer.contains("35"), "Should include percentage value");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("connectivity problems"), "Should explain impact");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("ping"), "Should have ping diagnostic command");
}

#[test]
fn test_latency_routes_to_quality_composer() {
    let brain = create_brain_with_insight(
        "high_latency",
        "warning",
        "High latency detected: 350ms",
        "Network connection has elevated latency.",
        "350ms latency to gateway",
    );

    let result = route_network_remediation(&brain);
    assert!(result.is_some(), "Should route latency to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("latency"), "Should mention latency");
    assert!(answer.contains("350"), "Should include latency value");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("High latency"), "Should explain issue");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("traceroute"), "Should have traceroute command");
}

#[test]
fn test_interface_errors_quality_composer() {
    // Test the quality composer directly with errors scenario
    let answer = compose_network_quality_fix_answer("errors", 0.0, Some("eth0"));

    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Network interface errors detected"), "Should have correct summary");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("eth0"), "Should mention interface");
    assert!(answer.contains("hardware or driver"), "Should explain error cause");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("ethtool eth0"), "Should have ethtool command");
}

#[test]
fn test_healthy_network_no_remediation() {
    let brain = BrainAnalysisData {
        insights: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
        proactive_health_score: 100,
    };

    let result = route_network_remediation(&brain);
    assert!(result.is_none(), "Healthy network should not trigger remediation");
}

#[test]
fn test_canonical_format_validation() {
    // Test that all three composers produce canonical format
    let priority_answer = compose_network_priority_fix_answer("eth0", 100, "wlan0", 866);
    let route_answer = compose_network_route_fix_answer("duplicate", vec!["eth0", "wlan0"]);
    let quality_answer = compose_network_quality_fix_answer("packet_loss", 35.0, None);

    // All must have [SUMMARY], [DETAILS], [COMMANDS]
    for (name, answer) in [
        ("priority", priority_answer),
        ("route", route_answer),
        ("quality", quality_answer),
    ] {
        assert!(
            answer.starts_with("[SUMMARY]"),
            "{} answer should start with [SUMMARY]",
            name
        );
        assert!(
            answer.contains("[DETAILS]"),
            "{} answer should contain [DETAILS]",
            name
        );
        assert!(
            answer.contains("[COMMANDS]"),
            "{} answer should contain [COMMANDS]",
            name
        );

        // Commands section should have actual commands (not empty)
        let commands_section = answer.split("[COMMANDS]").nth(1).expect("Commands section should exist");
        assert!(
            commands_section.contains("ip ") ||
            commands_section.contains("nmcli ") ||
            commands_section.contains("systemctl ") ||
            commands_section.contains("ethtool ") ||
            commands_section.contains("ping "),
            "{} answer should contain actual commands",
            name
        );

        // No LLM-style preamble
        assert!(
            !answer.contains("I recommend") &&
            !answer.contains("You might want to") &&
            !answer.contains("Let me help"),
            "{} answer should not contain LLM-style language",
            name
        );
    }
}
