//! UX regression tests for v0.45.x stabilization (annactl side).
//!
//! These tests ensure stable user-facing behavior for shared types.

use anna_shared::rpc::{ReliabilitySignals, SpecialistDomain};
use anna_shared::transcript::Actor;

// === Role Label Tests ===

#[test]
fn test_actor_you_format() {
    // v0.45.x: Actor::You should display as [you]
    let actor = Actor::You;
    let display = format!("{}", actor);
    // Actor Display impl should be lowercase
    assert_eq!(display, "you");
}

#[test]
fn test_actor_anna_format() {
    // v0.45.x: Actor::Anna should display as [anna]
    let actor = Actor::Anna;
    let display = format!("{}", actor);
    assert_eq!(display, "anna");
}

#[test]
fn test_actor_junior_format() {
    let actor = Actor::Junior;
    let display = format!("{}", actor);
    assert_eq!(display, "junior");
}

#[test]
fn test_actor_senior_format() {
    let actor = Actor::Senior;
    let display = format!("{}", actor);
    assert_eq!(display, "senior");
}

// === Probe Spine Tests ===

#[test]
fn test_route_capability_evidence_required() {
    use anna_shared::probe_spine::RouteCapability;

    // Evidence-requiring queries should have evidence_required=true
    let cap = RouteCapability {
        can_answer_deterministically: false,
        required_evidence: vec![],
        spine_probes: vec![],
        evidence_required: true,
    };

    assert!(cap.evidence_required, "RouteCapability should track evidence requirement");
}

#[test]
fn test_enforce_spine_probes_adds_minimum() {
    use anna_shared::probe_spine::{enforce_spine_probes, EvidenceKind, ProbeId, RouteCapability};

    let empty_probes: Vec<String> = vec![];
    let cap = RouteCapability {
        can_answer_deterministically: false,
        required_evidence: vec![EvidenceKind::Memory],
        spine_probes: vec![ProbeId::Free],
        evidence_required: true,
    };

    let (enforced, reason) = enforce_spine_probes(&empty_probes, &cap);

    // v0.45.x: Should add minimum probes when evidence required but none planned
    assert!(!enforced.is_empty(), "Should enforce minimum probes");
    assert!(reason.is_some(), "Should provide reason for enforcement");
}

#[test]
fn test_enforce_spine_probes_no_change_when_probes_exist() {
    use anna_shared::probe_spine::{enforce_spine_probes, EvidenceKind, ProbeId, RouteCapability};

    let existing_probes = vec!["free".to_string()];
    let cap = RouteCapability {
        can_answer_deterministically: false,
        required_evidence: vec![EvidenceKind::Memory],
        spine_probes: vec![ProbeId::Free],
        evidence_required: true,
    };

    let (enforced, reason) = enforce_spine_probes(&existing_probes, &cap);

    // Should not change when probes already exist
    assert_eq!(enforced, existing_probes, "Should not change existing probes");
    assert!(reason.is_none(), "Should not provide reason when no change");
}

// === Recipe Persistence Tests ===

#[test]
fn test_recipe_persist_requires_verified() {
    use anna_shared::recipe::should_persist_recipe;

    // Must be verified AND score >= 80
    assert!(!should_persist_recipe(false, 90), "Unverified should not persist");
    assert!(!should_persist_recipe(true, 70), "Low score should not persist");
    assert!(should_persist_recipe(true, 80), "Verified + 80 should persist");
    assert!(should_persist_recipe(true, 100), "Verified + 100 should persist");
}

#[test]
fn test_recipe_threshold_constant() {
    use anna_shared::recipe::RECIPE_PERSIST_THRESHOLD;

    assert_eq!(RECIPE_PERSIST_THRESHOLD, 80, "Threshold should be 80");
}

#[test]
fn test_recipe_persist_boundary() {
    use anna_shared::recipe::should_persist_recipe;

    // Boundary tests
    assert!(!should_persist_recipe(true, 79), "79 should not persist");
    assert!(should_persist_recipe(true, 80), "80 should persist");
    assert!(should_persist_recipe(true, 81), "81 should persist");
}

// === Reliability Signals Tests ===

#[test]
fn test_reliability_signals_default() {
    let signals = ReliabilitySignals::default();

    // Default should be conservative
    assert!(!signals.translator_confident);
    assert!(!signals.probe_coverage);
    assert!(!signals.answer_grounded);
}

// === Domain Format Tests ===

#[test]
fn test_domain_display_format() {
    let domain = SpecialistDomain::System;
    let display = format!("{}", domain);
    assert!(!display.is_empty(), "Domain should have display format");
}
