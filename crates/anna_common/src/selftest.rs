//! Self-test - Built-in capability verification (6.3.1)
//!
//! Runs offline, deterministic tests proving what Anna can actually do.
//! No LLM calls, no network - uses static Arch Wiki data and synthetic telemetry.

use crate::orchestrator::{
    get_arch_help_dns, get_arch_help_service_failure,
    plan_dns_fix, plan_service_failure_fix,
    PlanStepKind, TelemetrySummary,
};

/// Result of a single self-test scenario
#[derive(Debug, Clone)]
pub struct SelftestResult {
    pub scenario: String,
    pub passed: bool,
    pub details: String,
}

impl SelftestResult {
    fn pass(scenario: &str, details: &str) -> Self {
        Self {
            scenario: scenario.to_string(),
            passed: true,
            details: details.to_string(),
        }
    }

    fn fail(scenario: &str, reason: &str) -> Self {
        Self {
            scenario: scenario.to_string(),
            passed: false,
            details: reason.to_string(),
        }
    }
}

/// Run all self-tests and return results
///
/// This function runs completely offline using static Arch Wiki data
/// and synthetic telemetry fixtures. No network or LLM required.
pub fn run_selftests() -> Vec<SelftestResult> {
    vec![
        test_dns_scenario(),
        test_service_failure_scenario(),
        test_healthy_system_safety(),
    ]
}

/// Test: DNS resolution troubleshooting scenario
fn test_dns_scenario() -> SelftestResult {
    let scenario = "DNS / name resolution";

    // Synthetic telemetry: DNS broken, network up
    let telemetry = TelemetrySummary::dns_issue();
    let wiki = get_arch_help_dns();

    // Run planner
    let plan = plan_dns_fix("DNS self-test", &telemetry, &wiki);

    // Verify plan structure
    if plan.steps.is_empty() {
        return SelftestResult::fail(scenario, "Plan is empty when DNS issue detected");
    }

    // Check: Inspect before Change
    let first_change_idx = plan.steps.iter()
        .position(|s| s.kind == PlanStepKind::Change);

    if let Some(change_idx) = first_change_idx {
        let has_inspect_before = plan.steps[..change_idx].iter()
            .any(|s| s.kind == PlanStepKind::Inspect);

        if !has_inspect_before {
            return SelftestResult::fail(scenario, "Change step before Inspect step");
        }
    }

    // Check: All Change steps require confirmation
    for step in &plan.steps {
        if step.kind == PlanStepKind::Change && !step.requires_confirmation {
            return SelftestResult::fail(
                scenario,
                &format!("Change step without confirmation: {}", step.command),
            );
        }
    }

    // Check: All steps have Arch Wiki sources
    for step in &plan.steps {
        if step.knowledge_sources.is_empty() {
            return SelftestResult::fail(
                scenario,
                &format!("Step without knowledge sources: {}", step.command),
            );
        }

        for source in &step.knowledge_sources {
            if !source.url.starts_with("https://wiki.archlinux.org/") {
                return SelftestResult::fail(
                    scenario,
                    &format!("Non-Arch Wiki source: {}", source.url),
                );
            }
        }
    }

    // Count step types
    let inspect_count = plan.steps.iter()
        .filter(|s| s.kind == PlanStepKind::Inspect)
        .count();
    let change_count = plan.steps.iter()
        .filter(|s| s.kind == PlanStepKind::Change)
        .count();

    SelftestResult::pass(
        scenario,
        &format!(
            "plan produced: {} inspect + {} change, Arch Wiki: Systemd-resolved",
            inspect_count, change_count
        ),
    )
}

/// Test: Service failure troubleshooting scenario
fn test_service_failure_scenario() -> SelftestResult {
    let scenario = "Service failure (systemd)";

    // Synthetic telemetry: nginx service failed
    let telemetry = TelemetrySummary::with_failed_service("nginx");
    let wiki = get_arch_help_service_failure("nginx");

    // Run planner
    let plan = plan_service_failure_fix("Service self-test", &telemetry, &wiki);

    // Verify plan structure
    if plan.steps.is_empty() {
        return SelftestResult::fail(scenario, "Plan is empty when service failed");
    }

    // Check: Inspect before Change
    let first_change_idx = plan.steps.iter()
        .position(|s| s.kind == PlanStepKind::Change);

    if let Some(change_idx) = first_change_idx {
        let has_inspect_before = plan.steps[..change_idx].iter()
            .any(|s| s.kind == PlanStepKind::Inspect);

        if !has_inspect_before {
            return SelftestResult::fail(scenario, "Change step before Inspect step");
        }
    }

    // Check: All Change steps require confirmation
    for step in &plan.steps {
        if step.kind == PlanStepKind::Change && !step.requires_confirmation {
            return SelftestResult::fail(
                scenario,
                &format!("Change step without confirmation: {}", step.command),
            );
        }
    }

    // Check: Change steps have rollback
    for step in &plan.steps {
        if step.kind == PlanStepKind::Change && step.rollback_command.is_none() {
            return SelftestResult::fail(
                scenario,
                &format!("Change step without rollback: {}", step.command),
            );
        }
    }

    // Check: All steps have Arch Wiki sources
    for step in &plan.steps {
        if step.knowledge_sources.is_empty() {
            return SelftestResult::fail(
                scenario,
                &format!("Step without knowledge sources: {}", step.command),
            );
        }

        for source in &step.knowledge_sources {
            if !source.url.starts_with("https://wiki.archlinux.org/") {
                return SelftestResult::fail(
                    scenario,
                    &format!("Non-Arch Wiki source: {}", source.url),
                );
            }
        }
    }

    // Count step types
    let inspect_count = plan.steps.iter()
        .filter(|s| s.kind == PlanStepKind::Inspect)
        .count();
    let change_count = plan.steps.iter()
        .filter(|s| s.kind == PlanStepKind::Change)
        .count();

    SelftestResult::pass(
        scenario,
        &format!(
            "plan produced: {} inspect + {} change, Arch Wiki: Systemd",
            inspect_count, change_count
        ),
    )
}

/// Test: Healthy system should not propose changes
fn test_healthy_system_safety() -> SelftestResult {
    let scenario = "Healthy system safety check";

    // Synthetic telemetry: all healthy
    let telemetry = TelemetrySummary::healthy();

    // Try DNS planner
    let dns_wiki = get_arch_help_dns();
    let dns_plan = plan_dns_fix("Healthy self-test", &telemetry, &dns_wiki);

    if !dns_plan.steps.is_empty() {
        return SelftestResult::fail(
            scenario,
            "DNS planner proposed steps when system is healthy",
        );
    }

    // Try service planner
    let service_wiki = get_arch_help_service_failure("nginx");
    let service_plan = plan_service_failure_fix("Healthy self-test", &telemetry, &service_wiki);

    if !service_plan.steps.is_empty() {
        return SelftestResult::fail(
            scenario,
            "Service planner proposed steps when no services failed",
        );
    }

    SelftestResult::pass(scenario, "no changes proposed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selftests_all_pass() {
        let results = run_selftests();

        // Should have 3 scenarios
        assert_eq!(results.len(), 3, "Should run 3 self-test scenarios");

        // All should pass
        for result in &results {
            assert!(
                result.passed,
                "Scenario '{}' failed: {}",
                result.scenario,
                result.details
            );
        }
    }

    #[test]
    fn test_selftest_result_format() {
        let pass = SelftestResult::pass("Test scenario", "details here");
        assert!(pass.passed);
        assert_eq!(pass.scenario, "Test scenario");
        assert_eq!(pass.details, "details here");

        let fail = SelftestResult::fail("Test scenario", "reason here");
        assert!(!fail.passed);
        assert_eq!(fail.scenario, "Test scenario");
        assert_eq!(fail.details, "reason here");
    }
}
