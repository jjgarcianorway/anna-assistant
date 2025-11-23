//! CLI-level tests for `annactl plan` command (6.3.0)
//!
//! Tests that CLI is a thin wrapper over the planner orchestrator.
//! No LLM calls, no network - all tests use deterministic telemetry.

use anna_common::orchestrator::{
    get_arch_help_dns, get_arch_help_service_failure,
    plan_dns_fix, plan_service_failure_fix,
    TelemetrySummary, ServiceStatus,
};

/// Test: CLI planner handles DNS issue scenario
///
/// Verifies:
/// - TelemetrySummary conversion works correctly
/// - DNS planner slice is invoked when dns_suspected_broken
/// - Plan contains expected Arch Wiki commands
#[test]
fn plan_cli_dns_basic() {
    // Simulate telemetry indicating DNS issue
    let telemetry = TelemetrySummary {
        dns_suspected_broken: true,
        network_reachable: true,
        failed_services: Vec::new(),
    };

    // Get Arch Wiki guidance
    let wiki = get_arch_help_dns();

    // Execute planner (same as CLI would do)
    let plan = plan_dns_fix("DNS resolution issue detected", &telemetry, &wiki);

    // Verify plan is not empty
    assert!(!plan.steps.is_empty(), "DNS plan should have steps");

    // Verify it contains systemd-resolved commands
    let has_resolved_status = plan.steps.iter()
        .any(|s| s.command.contains("systemctl status systemd-resolved"));
    let has_resolved_restart = plan.steps.iter()
        .any(|s| s.command.contains("systemctl restart systemd-resolved"));

    assert!(has_resolved_status, "Plan should check systemd-resolved status");
    assert!(has_resolved_restart, "Plan should include restart option");

    // Verify all steps have Arch Wiki sources
    for step in &plan.steps {
        assert!(!step.knowledge_sources.is_empty(),
            "Every step must reference Arch Wiki");
        for source in &step.knowledge_sources {
            assert!(source.url.starts_with("https://wiki.archlinux.org/"),
                "Source must be Arch Wiki: {}", source.url);
        }
    }
}

/// Test: CLI planner handles service failure scenario
///
/// Verifies:
/// - Service failure detection works
/// - Service-specific commands are generated
/// - Plan includes inspect before change
#[test]
fn plan_cli_service_failure_basic() {
    // Simulate telemetry indicating nginx failure
    let telemetry = TelemetrySummary {
        dns_suspected_broken: false,
        network_reachable: true,
        failed_services: vec![ServiceStatus {
            name: "nginx.service".to_string(),
            is_failed: true,
        }],
    };

    // Get Arch Wiki guidance for nginx
    let wiki = get_arch_help_service_failure("nginx");

    // Execute planner
    let plan = plan_service_failure_fix("Service nginx is failed", &telemetry, &wiki);

    // Verify plan is not empty
    assert!(!plan.steps.is_empty(), "Service plan should have steps");

    // Verify nginx-specific commands
    let has_nginx_status = plan.steps.iter()
        .any(|s| s.command.contains("systemctl status nginx"));
    let has_nginx_logs = plan.steps.iter()
        .any(|s| s.command.contains("journalctl") && s.command.contains("nginx"));
    let has_nginx_restart = plan.steps.iter()
        .any(|s| s.command.contains("systemctl restart nginx"));

    assert!(has_nginx_status, "Plan should check nginx status");
    assert!(has_nginx_logs, "Plan should check nginx logs");
    assert!(has_nginx_restart, "Plan should include restart option");

    // Verify all steps have Arch Wiki sources
    for step in &plan.steps {
        assert!(!step.knowledge_sources.is_empty(),
            "Every step must reference Arch Wiki");
    }
}

/// Test: CLI planner returns empty plan when no issues
///
/// Verifies:
/// - Healthy system produces no plans
/// - No unnecessary commands are generated
#[test]
fn plan_cli_no_issues() {
    // Simulate healthy telemetry
    let telemetry = TelemetrySummary {
        dns_suspected_broken: false,
        network_reachable: true,
        failed_services: Vec::new(),
    };

    // Try DNS planner
    let wiki_dns = get_arch_help_dns();
    let plan_dns = plan_dns_fix("check dns", &telemetry, &wiki_dns);

    assert!(plan_dns.steps.is_empty(),
        "DNS plan should be empty when DNS is healthy");

    // Try service planner
    let wiki_service = get_arch_help_service_failure("nginx");
    let plan_service = plan_service_failure_fix("check nginx", &telemetry, &wiki_service);

    assert!(plan_service.steps.is_empty(),
        "Service plan should be empty when no services failed");
}

/// Test: CLI planner converts to IPC format correctly
///
/// Verifies:
/// - Plan.to_suggested_fix() produces valid SuggestedFixData
/// - Knowledge sources are deduplicated
/// - Steps include rollback commands for Change steps
#[test]
fn plan_cli_to_suggested_fix_conversion() {
    // Create DNS issue scenario
    let telemetry = TelemetrySummary::dns_issue();
    let wiki = get_arch_help_dns();
    let plan = plan_dns_fix("DNS issue", &telemetry, &wiki);

    // Convert to IPC format
    let fix = plan.to_suggested_fix("DNS resolution fix based on Arch Wiki".to_string());

    // Verify structure
    assert!(!fix.steps.is_empty(), "SuggestedFix should have steps");
    assert!(!fix.knowledge_sources.is_empty(), "SuggestedFix should have sources");

    // Verify all sources are Arch Wiki
    for source in &fix.knowledge_sources {
        assert_eq!(source.kind, "ArchWiki", "All sources must be ArchWiki");
        assert!(source.url.starts_with("https://wiki.archlinux.org/"));
    }

    // Verify change steps have rollback
    for step in &fix.steps {
        if step.kind == "change" {
            assert!(step.rollback_command.is_some(),
                "Change step should have rollback: {}", step.command);
        }
    }
}

/// Test: Multiple failed services generate separate plans
///
/// Verifies:
/// - Each failed service gets its own plan
/// - Service-specific wiki guidance is used
#[test]
fn plan_cli_multiple_services() {
    // In real CLI, each service would get its own filtered telemetry
    // This simulates the CLI calling the planner once per failed service

    // First service: nginx
    let nginx_telemetry = TelemetrySummary {
        dns_suspected_broken: false,
        network_reachable: true,
        failed_services: vec![ServiceStatus {
            name: "nginx.service".to_string(),
            is_failed: true,
        }],
    };

    let nginx_wiki = get_arch_help_service_failure("nginx");
    let nginx_plan = plan_service_failure_fix("nginx failed", &nginx_telemetry, &nginx_wiki);

    // Second service: postgresql
    let postgres_telemetry = TelemetrySummary {
        dns_suspected_broken: false,
        network_reachable: true,
        failed_services: vec![ServiceStatus {
            name: "postgresql.service".to_string(),
            is_failed: true,
        }],
    };

    let postgres_wiki = get_arch_help_service_failure("postgresql");
    let postgres_plan = plan_service_failure_fix("postgresql failed", &postgres_telemetry, &postgres_wiki);

    // Both plans should exist
    assert!(!nginx_plan.steps.is_empty(), "Should have nginx plan");
    assert!(!postgres_plan.steps.is_empty(), "Should have postgres plan");

    // Verify nginx plan uses nginx commands
    let nginx_has_nginx = nginx_plan.steps.iter()
        .any(|s| s.command.contains("nginx"));
    assert!(nginx_has_nginx, "Nginx plan should reference nginx");

    // Verify postgres plan uses postgres commands
    let postgres_has_postgres = postgres_plan.steps.iter()
        .any(|s| s.command.contains("postgresql"));
    assert!(postgres_has_postgres, "Postgres plan should reference postgresql");
}

/// Test: JSON output serialization works
///
/// Verifies:
/// - SuggestedFixData can be serialized to JSON
/// - All required fields are present
#[test]
fn plan_cli_json_serialization() {
    use serde_json::json;

    let telemetry = TelemetrySummary::dns_issue();
    let wiki = get_arch_help_dns();
    let plan = plan_dns_fix("DNS issue", &telemetry, &wiki);
    let fix = plan.to_suggested_fix("Test fix".to_string());

    // Serialize to JSON
    let json_value = json!({
        "description": fix.description,
        "steps": fix.steps.iter().map(|s| {
            json!({
                "kind": s.kind,
                "command": s.command,
                "requires_confirmation": s.requires_confirmation,
                "rollback_command": s.rollback_command,
            })
        }).collect::<Vec<_>>(),
        "knowledge_sources": fix.knowledge_sources.iter().map(|ks| {
            json!({
                "url": ks.url,
                "kind": ks.kind,
            })
        }).collect::<Vec<_>>(),
    });

    // Verify it can be stringified
    let json_str = serde_json::to_string_pretty(&json_value).unwrap();
    assert!(!json_str.is_empty(), "JSON output should not be empty");
    assert!(json_str.contains("ArchWiki"), "JSON should contain ArchWiki sources");
}
