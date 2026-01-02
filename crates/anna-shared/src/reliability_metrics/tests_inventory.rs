//! Inventory, scenario, display, and conversion tests for reliability metrics.
//!
//! Tests for:
//! - Model and probe inventory management
//! - End-to-end scenario tests
//! - Stats display formatting
//! - Outcome conversion from legacy types

use super::*;

// === Model and Probe Inventory Tests ===

/// Test: Model inventory deduplication.
#[test]
fn test_model_inventory_no_duplicates() {
    let mut inv = ModelInventory::new();

    // Add same model with different cases
    inv.add_discovered("Qwen2.5:7b", ModelOwner::User);
    inv.add_discovered("qwen2.5:7b", ModelOwner::User);
    inv.add_discovered("QWEN2.5:7B", ModelOwner::User);

    // Should only have 1 model
    assert_eq!(inv.discovered_count(), 1);
}

/// Test: Model inventory ownership tracking.
#[test]
fn test_model_inventory_ownership() {
    let mut inv = ModelInventory::new();

    inv.add_discovered("user_model:1b", ModelOwner::User);
    inv.add_anna_installed("anna_model:1b");

    assert_eq!(inv.user_installed_count(), 1);
    assert_eq!(inv.anna_installed_count(), 1);
    assert_eq!(inv.discovered_count(), 2);
}

/// Test: Probe inventory.
#[test]
fn test_probe_inventory() {
    let inv = default_probe_inventory();

    // Should have common probes
    assert!(inv.get("df").is_some());
    assert!(inv.get("free").is_some());
    assert!(inv.get("systemctl_failed").is_some());

    // Display should work
    let display = inv.display();
    assert!(display.contains("df"));
    assert!(display.contains("probes available"));
}

// === End-to-end Scenario Tests ===

/// Scenario: "is this a laptop or a desktop?"
/// Without proper hardware probes, should be CLARIFICATION_NEEDED or FAILED_PROBES.
#[test]
fn test_scenario_laptop_or_desktop() {
    // If probes fail (e.g., can't determine chassis type)
    let cond_probe_fail = OutcomeConditions {
        probes_failed: true,
        ..Default::default()
    };
    assert_eq!(cond_probe_fail.determine(), CanonicalOutcome::FailedProbes);

    // If we can't answer with evidence, should ask clarification
    let cond_clarify = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        clarification_asked: true,
        answer_rendered: false,
        ..Default::default()
    };
    assert_eq!(
        cond_clarify.determine(),
        CanonicalOutcome::ClarificationNeeded
    );

    // Should NEVER be "verified 90%" without evidence
    let cond_no_evidence = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.0, // No evidence
        ..Default::default()
    };
    // Low evidence = partial, not verified
    assert_eq!(
        cond_no_evidence.determine(),
        CanonicalOutcome::AnsweredPartial
    );
    assert!(!cond_no_evidence.determine().is_resolved());
}

/// Scenario: "what is my vim setup?"
/// Needs .vimrc probe; if fails, should not claim success.
#[test]
fn test_scenario_vim_setup() {
    // Probe success with evidence
    let cond_success = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.9, // Has .vimrc content
        ..Default::default()
    };
    assert_eq!(cond_success.determine(), CanonicalOutcome::AnsweredVerified);

    // Probe failed (no .vimrc)
    let cond_probe_fail = OutcomeConditions {
        probes_failed: true,
        ..Default::default()
    };
    assert_eq!(cond_probe_fail.determine(), CanonicalOutcome::FailedProbes);
}

/// Scenario: "is nano installed?"
/// Simple yes/no with evidence from `which nano`.
#[test]
fn test_scenario_nano_installed() {
    // Probe worked, evidence supports answer
    let cond_yes = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 1.0, // which nano returned path
        ..Default::default()
    };
    assert_eq!(cond_yes.determine(), CanonicalOutcome::AnsweredVerified);

    // Probe worked, no nano found - still verified (evidence supports "no")
    let cond_no = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 1.0, // which nano returned empty
        ..Default::default()
    };
    assert_eq!(cond_no.determine(), CanonicalOutcome::AnsweredVerified);
}

// === Display and Conversion Tests ===

/// Test: Stats display format.
#[test]
fn test_stats_display_format() {
    let mut stats = ReliabilityStats::new();
    stats.record_outcome(CanonicalOutcome::AnsweredVerified, Some("storage"));
    stats.record_outcome(CanonicalOutcome::FailedTimeout, Some("network"));

    let display = stats.display();

    // Must contain section headers
    assert!(display.contains("[requests]"));
    assert!(display.contains("[latency]"));
    assert!(display.contains("[reliability]"));

    // Must contain actual counts
    assert!(display.contains("total_requests"));
    assert!(display.contains("answered_verified"));
    assert!(display.contains("failed_timeout"));

    // Must contain rates
    assert!(display.contains("verified_rate"));
    assert!(display.contains("failure_rate"));
}

/// Test: Conversion from old ticket_integrity outcome.
#[test]
fn test_from_ticket_integrity_outcome() {
    use crate::ticket_integrity::outcome::TicketOutcome as TIO;

    assert_eq!(
        from_ticket_integrity_outcome(TIO::Answered),
        CanonicalOutcome::AnsweredVerified
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ParseError),
        CanonicalOutcome::FailedParse
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ProbeError),
        CanonicalOutcome::FailedProbes
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ClarificationPending),
        CanonicalOutcome::ClarificationNeeded
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::Cancelled),
        CanonicalOutcome::AbortedByUser
    );
}

/// Test: Conversion from old ticket_state outcome.
#[test]
fn test_from_ticket_state_outcome() {
    use crate::ticket_state::TicketOutcome as TSO;

    assert_eq!(
        from_ticket_state_outcome(TSO::Success),
        CanonicalOutcome::AnsweredVerified
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::Partial),
        CanonicalOutcome::AnsweredPartial
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorParse),
        CanonicalOutcome::FailedParse
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorTimeout),
        CanonicalOutcome::FailedTimeout
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorTool),
        CanonicalOutcome::FailedProbes
    );
}
