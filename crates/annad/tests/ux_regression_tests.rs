//! UX regression tests for v0.45.x stabilization (annad side).
//!
//! These tests ensure stable behavior for routing and service desk.

use anna_shared::rpc::{ProbeResult, SpecialistDomain};
use anna_shared::transcript::Transcript;

// === Timeout Response Tests ===

#[test]
fn test_timeout_response_has_answer() {
    use annad::service_desk::create_timeout_response;

    let result = create_timeout_response(
        "test-id".to_string(),
        "specialist",
        None,
        vec![],
        Transcript::default(),
        SpecialistDomain::System,
    );

    // v0.45.x: Timeout response MUST have non-empty answer
    assert!(!result.answer.is_empty(), "Timeout response must have answer");
    assert!(
        result.answer.contains("Timeout"),
        "Answer should mention timeout"
    );
}

#[test]
fn test_timeout_response_never_asks_rephrase() {
    use annad::service_desk::create_timeout_response;

    let result = create_timeout_response(
        "test-id".to_string(),
        "probe",
        None,
        vec![],
        Transcript::default(),
        SpecialistDomain::System,
    );

    // v0.45.x: NEVER ask to rephrase
    assert!(!result.needs_clarification, "Timeout should not need clarification");
    assert!(
        result.clarification_question.is_none(),
        "Timeout should not have clarification question"
    );
}

#[test]
fn test_timeout_response_with_evidence() {
    use annad::service_desk::create_timeout_response;

    // Create some probe results
    let probe_results = vec![ProbeResult {
        command: "free -b".to_string(),
        stdout: "total: 16000000000\nused: 8000000000".to_string(),
        stderr: String::new(),
        exit_code: 0,
        timing_ms: 50,
    }];

    let result = create_timeout_response(
        "test-id".to_string(),
        "specialist",
        None,
        probe_results,
        Transcript::default(),
        SpecialistDomain::System,
    );

    // v0.45.x: Should include evidence summary
    assert!(
        result.answer.contains("Evidence") || result.answer.contains("free"),
        "Answer should summarize available evidence"
    );
    // Higher reliability when evidence exists
    assert!(result.reliability_score > 20, "Should have higher score with evidence");
}

#[test]
fn test_timeout_response_low_reliability_without_evidence() {
    use annad::service_desk::create_timeout_response;

    let result = create_timeout_response(
        "test-id".to_string(),
        "probe",
        None,
        vec![], // No evidence
        Transcript::default(),
        SpecialistDomain::System,
    );

    // Without evidence, reliability should be low (but not zero)
    assert!(result.reliability_score <= 40, "Should have low score without evidence");
    assert!(result.reliability_score > 0, "Should have non-zero score");
}

// === Deterministic Answer Gating Tests ===

#[test]
fn test_unknown_query_not_deterministic() {
    use annad::router::{get_route, QueryClass};

    let route = get_route("some random gibberish query xyz");

    // Unknown queries should NOT be deterministic
    if route.class == QueryClass::Unknown {
        assert!(
            !route.can_answer_deterministically(),
            "Unknown queries must not be deterministic"
        );
    }
}

#[test]
fn test_memory_usage_is_deterministic() {
    use annad::router::get_route;

    let route = get_route("memory usage");

    // Memory usage should be deterministic (typed query)
    assert!(
        route.can_answer_deterministically(),
        "Memory usage should be deterministic"
    );
}

#[test]
fn test_disk_usage_is_deterministic() {
    use annad::router::get_route;

    let route = get_route("disk usage");

    assert!(
        route.can_answer_deterministically(),
        "Disk usage should be deterministic"
    );
}

#[test]
fn test_system_health_summary_not_deterministic() {
    use annad::router::get_route;

    let route = get_route("system summary");

    // v0.45.x: SystemHealthSummary needs LLM interpretation
    assert!(
        !route.can_answer_deterministically(),
        "System summary should NOT be deterministic (needs LLM)"
    );
}

#[test]
fn test_service_status_is_deterministic() {
    use annad::router::get_route;

    let route = get_route("is nginx running");

    assert!(
        route.can_answer_deterministically(),
        "Service status should be deterministic"
    );
}

#[test]
fn test_help_is_deterministic() {
    use annad::router::get_route;

    let route = get_route("help");

    assert!(
        route.can_answer_deterministically(),
        "Help should be deterministic"
    );
}

// === Route Capability Tests ===

#[test]
fn test_route_has_capability() {
    use annad::router::get_route;

    let route = get_route("memory usage");

    // All routes should have capability
    assert!(
        route.capability.required_evidence.is_empty() || !route.capability.spine_probes.is_empty(),
        "If evidence required, should have spine probes"
    );
}

#[test]
fn test_deterministic_route_probes() {
    use annad::router::get_route;

    let route = get_route("disk space");

    // Deterministic routes should have probes
    if route.can_answer_deterministically() {
        assert!(
            !route.probes.is_empty(),
            "Deterministic routes should have probes"
        );
    }
}
