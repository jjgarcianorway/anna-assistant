//! ACTS v1 - Anna Capability Test Suite
//!
//! Tests core planner capabilities for DNS and service failure scenarios
//! Enforces Arch Wiki-only policy and safety guarantees

use anna_common::orchestrator::{
    get_arch_help_dns, get_arch_help_service_failure,
    plan_dns_fix, plan_service_failure_fix,
    PlanStepKind, TelemetrySummary,
};

#[test]
fn acts_dns_fix_basic() {
    // Input: DNS broken, network up
    let telemetry = TelemetrySummary::dns_issue();
    let wiki = get_arch_help_dns();

    // Execute planner
    let plan = plan_dns_fix("I cannot reach example.com but internet works", &telemetry, &wiki);

    // Verify plan structure
    assert!(!plan.steps.is_empty(), "Plan should have steps");

    // Must have Inspect and Change steps
    let has_inspect = plan.steps.iter().any(|s| s.kind == PlanStepKind::Inspect);
    let has_change = plan.steps.iter().any(|s| s.kind == PlanStepKind::Change);

    assert!(has_inspect, "Plan must have Inspect steps");
    assert!(has_change, "Plan must have Change steps");

    // Inspect must come before Change
    let first_change_idx = plan.steps.iter()
        .position(|s| s.kind == PlanStepKind::Change);

    if let Some(change_idx) = first_change_idx {
        let has_inspect_before = plan.steps[..change_idx].iter()
            .any(|s| s.kind == PlanStepKind::Inspect);

        assert!(has_inspect_before, "Inspect steps must come before Change");
    }

    // All Change steps must require confirmation
    for step in &plan.steps {
        if step.kind == PlanStepKind::Change {
            assert!(step.requires_confirmation,
                "Change step must require confirmation: {}", step.command);
        }
    }

    // All steps must have Arch Wiki sources
    for step in &plan.steps {
        assert!(!step.knowledge_sources.is_empty(),
            "Step must have knowledge sources: {}", step.command);

        for source in &step.knowledge_sources {
            assert!(source.url.starts_with("https://wiki.archlinux.org/"),
                "Source must be Arch Wiki: {}", source.url);
        }
    }
}

#[test]
fn acts_dns_no_change_if_network_down() {
    // Input: DNS broken, network DOWN
    let telemetry = TelemetrySummary {
        dns_suspected_broken: true,
        network_reachable: false,  // Network is down
        failed_services: Vec::new(),
    };
    let wiki = get_arch_help_dns();

    // Execute planner
    let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

    // Should have no Change steps when network is unreachable
    let has_change = plan.steps.iter().any(|s| s.kind == PlanStepKind::Change);

    assert!(!has_change,
        "Should not propose DNS changes when network is unreachable");
}

#[test]
fn acts_service_failure_fix_basic() {
    // Input: nginx service failed
    let telemetry = TelemetrySummary::with_failed_service("nginx");
    let wiki = get_arch_help_service_failure("nginx");

    // Execute planner
    let plan = plan_service_failure_fix("My nginx service keeps failing", &telemetry, &wiki);

    // Verify plan structure
    assert!(!plan.steps.is_empty(), "Plan should have steps");

    // Must have Inspect and Change steps
    let has_inspect = plan.steps.iter().any(|s| s.kind == PlanStepKind::Inspect);
    let has_change = plan.steps.iter().any(|s| s.kind == PlanStepKind::Change);

    assert!(has_inspect, "Plan must have Inspect steps");
    assert!(has_change, "Plan must have Change steps");

    // Inspect must come before Change
    let first_change_idx = plan.steps.iter()
        .position(|s| s.kind == PlanStepKind::Change);

    if let Some(change_idx) = first_change_idx {
        let has_inspect_before = plan.steps[..change_idx].iter()
            .any(|s| s.kind == PlanStepKind::Inspect);

        assert!(has_inspect_before, "Inspect steps must come before Change");
    }

    // Verify service-specific commands exist
    let has_status = plan.steps.iter()
        .any(|s| s.command.contains("systemctl status") && s.command.contains("nginx"));
    let has_logs = plan.steps.iter()
        .any(|s| s.command.contains("journalctl") && s.command.contains("nginx"));

    assert!(has_status, "Plan should check service status");
    assert!(has_logs, "Plan should check service logs");

    // All Change steps must require confirmation and have rollback
    for step in &plan.steps {
        if step.kind == PlanStepKind::Change {
            assert!(step.requires_confirmation,
                "Change step must require confirmation: {}", step.command);
            assert!(step.rollback_command.is_some(),
                "Change step should have rollback: {}", step.command);
        }
    }

    // All steps must have Arch Wiki sources
    for step in &plan.steps {
        assert!(!step.knowledge_sources.is_empty(),
            "Step must have knowledge sources: {}", step.command);

        for source in &step.knowledge_sources {
            assert!(source.url.starts_with("https://wiki.archlinux.org/"),
                "Source must be Arch Wiki: {}", source.url);
        }
    }
}

#[test]
fn acts_service_no_change_if_no_failed_services() {
    // Input: No failed services
    let telemetry = TelemetrySummary::healthy();
    let wiki = get_arch_help_service_failure("nginx");

    // Execute planner
    let plan = plan_service_failure_fix("check nginx", &telemetry, &wiki);

    // Should have no Change steps when no services are failed
    let has_change = plan.steps.iter().any(|s| s.kind == PlanStepKind::Change);

    assert!(!has_change,
        "Should not propose service changes when no services are failed");
}
