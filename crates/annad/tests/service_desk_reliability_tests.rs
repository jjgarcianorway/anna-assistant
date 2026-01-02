//! Integration tests for service desk reliability signals.
//!
//! These tests verify:
//! - Reliability scoring (deterministic)
//! - Signal calculation

use anna_shared::rpc::ReliabilitySignals;

// === Helper Functions ===

fn make_signals(confident: bool, coverage: bool, grounded: bool) -> ReliabilitySignals {
    ReliabilitySignals {
        translator_confident: confident,
        probe_coverage: coverage,
        answer_grounded: grounded,
        no_invention: true,
        clarification_not_needed: true,
    }
}

// === Reliability Signals Tests ===

#[test]
fn test_reliability_score_calculation() {
    // All signals true = 100
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };
    assert_eq!(signals.score(), 100);

    // All signals false = 0
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: false,
        clarification_not_needed: false,
    };
    assert_eq!(signals.score(), 0);

    // Each signal = 20 points
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: false,
        clarification_not_needed: false,
    };
    assert_eq!(signals.score(), 20);
}

#[test]
fn test_reliability_score_deterministic() {
    // Same inputs must always produce same score
    let signals1 = make_signals(true, true, false);
    let signals2 = make_signals(true, true, false);

    assert_eq!(signals1.score(), signals2.score());
    assert_eq!(signals1.score(), 80); // confident + coverage + no_invention + no_clarify
}
