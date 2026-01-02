//! Integration tests for deterministic answerer - Reliability scoring.

use anna_shared::rpc::ReliabilitySignals;

// === Reliability scoring tests ===

#[test]
fn test_deterministic_scoring_with_probes() {
    // When probes succeed and deterministic answer is generated,
    // reliability score should be > 20 even if translator timed out
    let signals = ReliabilitySignals {
        translator_confident: false,    // Translator timed out
        probe_coverage: true,           // All probes succeeded
        answer_grounded: true,          // Deterministic = always grounded
        no_invention: true,             // Deterministic = never invents
        clarification_not_needed: true, // We produced an answer
    };

    // Score should be 80 (4 signals * 20)
    assert_eq!(signals.score(), 80);
    assert!(
        signals.score() > 20,
        "Score should be > 20 when probes succeed"
    );
}

#[test]
fn test_reliability_without_translator() {
    // Deterministic answer without translator should still have good score
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    assert_eq!(signals.score(), 80);
}

#[test]
fn test_reliability_with_all_signals() {
    // Maximum reliability when all signals are positive
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    assert_eq!(signals.score(), 100);
}

#[test]
fn test_reliability_with_no_signals() {
    // Minimum reliability when no signals are positive
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: false,
        clarification_not_needed: false,
    };

    assert_eq!(signals.score(), 0);
}
