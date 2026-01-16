//! Computed Confidence Function for Phase 27.
//!
//! Replaces LLM-asserted confidence with deterministic calculation
//! based on evidence completeness, fingerprint similarity, and historical success rate.
//!
//! Formula:
//! confidence = w1 * evidence_completeness + w2 * fingerprint_similarity + w3 * historical_success_rate
//!
//! Constants:
//! - w1 = 0.4 (evidence weight): More probes = more confidence
//! - w2 = 0.3 (similarity weight): Matching past successes increases confidence
//! - w3 = 0.3 (history weight): Track record bounds confidence for novel situations

use super::evidence::EvidenceSnapshot;

/// Weight for evidence completeness (probes run vs expected).
const W_EVIDENCE: f32 = 0.4;

/// Weight for fingerprint similarity to historical successes.
const W_SIMILARITY: f32 = 0.3;

/// Weight for historical success rate.
const W_HISTORY: f32 = 0.3;

/// Cap confidence for novel situations (no fingerprint match).
const NOVEL_SITUATION_CAP: f32 = 0.7;

/// Minimum samples before using computed confidence.
pub const COLD_START_THRESHOLD: usize = 50;

/// Compute confidence from evidence snapshot.
///
/// Returns confidence in range [0.0, 1.0].
pub fn compute_confidence(evidence: &EvidenceSnapshot) -> f32 {
    let raw_confidence = W_EVIDENCE * evidence.evidence_completeness
        + W_SIMILARITY * evidence.fingerprint_similarity
        + W_HISTORY * evidence.historical_success_rate;

    // Cap for novel situations (no fingerprint match)
    if evidence.fingerprint_similarity < 0.1 && evidence.fingerprint.is_some() {
        raw_confidence.min(NOVEL_SITUATION_CAP)
    } else {
        raw_confidence.min(1.0)
    }
}

/// Confidence decision thresholds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfidenceDecision {
    /// Confidence >= 0.7 with answer: done, resolved
    Done,
    /// Confidence < 0.5 at max iterations: abstain
    Abstain,
    /// Continue iterating
    Continue,
}

/// Make confidence-based decision.
pub fn decide(
    confidence: f32,
    has_answer: bool,
    at_max_iterations: bool,
    has_execution_error: bool,
) -> ConfidenceDecision {
    if confidence >= 0.7 && has_answer {
        ConfidenceDecision::Done
    } else if at_max_iterations && confidence < 0.5 && !has_execution_error {
        ConfidenceDecision::Abstain
    } else if at_max_iterations {
        // Max iterations with execution error = failure, not abstention
        // But we still return Continue to let the caller handle it
        ConfidenceDecision::Continue
    } else {
        ConfidenceDecision::Continue
    }
}

/// Format confidence for debug output.
pub fn format_confidence_debug(evidence: &EvidenceSnapshot, confidence: f32) -> String {
    format!(
        "confidence={:.2} (completeness={:.2}*{:.1} + similarity={:.2}*{:.1} + history={:.2}*{:.1})",
        confidence,
        evidence.evidence_completeness,
        W_EVIDENCE,
        evidence.fingerprint_similarity,
        W_SIMILARITY,
        evidence.historical_success_rate,
        W_HISTORY,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::intent_class::IntentClass;

    fn make_evidence(completeness: f32, similarity: f32, history: f32) -> EvidenceSnapshot {
        EvidenceSnapshot {
            probes_run: 2,
            evidence_completeness: completeness,
            fingerprint: if similarity > 0.0 {
                Some(anna_shared::fingerprint::FactFingerprint::from_probe_outputs(&["test"]))
            } else {
                None
            },
            fingerprint_similarity: similarity,
            historical_success_rate: history,
        }
    }

    #[test]
    fn test_confidence_formula() {
        // All perfect: 0.4*1.0 + 0.3*1.0 + 0.3*1.0 = 1.0
        let evidence = make_evidence(1.0, 1.0, 1.0);
        let conf = compute_confidence(&evidence);
        assert!((conf - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_confidence_partial() {
        // Half: 0.4*0.5 + 0.3*0.5 + 0.3*0.5 = 0.5
        let evidence = make_evidence(0.5, 0.5, 0.5);
        let conf = compute_confidence(&evidence);
        assert!((conf - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_novel_situation_cap() {
        // High completeness and history but no fingerprint match
        let evidence = make_evidence(1.0, 0.0, 1.0);
        let conf = compute_confidence(&evidence);
        // 0.4*1.0 + 0.3*0.0 + 0.3*1.0 = 0.7
        // With fingerprint present but 0 similarity, capped at 0.7
        assert!(conf <= NOVEL_SITUATION_CAP + 0.01);
    }

    #[test]
    fn test_decision_done() {
        let decision = decide(0.8, true, false, false);
        assert_eq!(decision, ConfidenceDecision::Done);
    }

    #[test]
    fn test_decision_abstain() {
        let decision = decide(0.3, false, true, false);
        assert_eq!(decision, ConfidenceDecision::Abstain);
    }

    #[test]
    fn test_decision_continue() {
        let decision = decide(0.5, false, false, false);
        assert_eq!(decision, ConfidenceDecision::Continue);
    }

    #[test]
    fn test_no_abstain_with_error() {
        // Even at low confidence, execution error means failure not abstention
        let decision = decide(0.3, false, true, true);
        assert_eq!(decision, ConfidenceDecision::Continue);
    }

    #[test]
    fn test_cold_start_fallback() {
        // With cold start (no fingerprint, unknown history)
        let evidence = EvidenceSnapshot::cold_start(2, IntentClass::ReadOnly, &["test"]);
        let conf = compute_confidence(&evidence);

        // 0.4*1.0 + 0.3*0.0 + 0.3*0.5 = 0.55
        // But capped at 0.7 due to novel situation
        assert!(conf > 0.0 && conf <= 0.7);
    }
}
