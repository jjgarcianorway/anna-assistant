//! Unified result signals builder (v0.0.75).
//!
//! Single source of truth for reliability signals to prevent inconsistencies.

use serde::{Deserialize, Serialize};

use crate::rpc::ProbeResult;
use crate::trace::EvidenceKind;

/// Outcome type for signal calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Answer verified with high confidence
    Verified,
    /// Answer requires clarification
    Clarification,
    /// Answer failed validation
    Failed,
    /// Request timed out
    Timeout,
    /// Deterministic answer (no LLM)
    Deterministic,
}

/// Evidence summary for signal calculation
#[derive(Debug, Clone, Default)]
pub struct EvidenceSummary {
    /// Number of probes planned
    pub probes_planned: usize,
    /// Number of probes executed
    pub probes_executed: usize,
    /// Number of probes with valid output
    pub probes_valid: usize,
    /// Evidence kinds found
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Whether hardware snapshot was used
    pub hardware_used: bool,
    /// Whether translator was used
    pub translator_used: bool,
    /// Translator confidence (0.0-1.0)
    pub translator_confidence: f32,
}

impl EvidenceSummary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from probe results
    pub fn from_probes(probes: &[ProbeResult], planned: usize) -> Self {
        let probes_executed = probes.len();
        let probes_valid = probes.iter().filter(|p| p.exit_code == 0).count();

        Self {
            probes_planned: planned,
            probes_executed,
            probes_valid,
            evidence_kinds: Vec::new(),
            hardware_used: false,
            translator_used: false,
            translator_confidence: 0.0,
        }
    }

    /// Add evidence kinds
    pub fn with_evidence_kinds(mut self, kinds: Vec<EvidenceKind>) -> Self {
        self.evidence_kinds = kinds;
        self
    }

    /// Mark hardware snapshot used
    pub fn with_hardware(mut self) -> Self {
        self.hardware_used = true;
        self
    }

    /// Set translator info
    pub fn with_translator(mut self, confidence: f32) -> Self {
        self.translator_used = true;
        self.translator_confidence = confidence;
        self
    }

    /// Has any valid evidence
    pub fn has_evidence(&self) -> bool {
        self.probes_valid > 0 || self.hardware_used || !self.evidence_kinds.is_empty()
    }

    /// Probe coverage ratio
    pub fn probe_coverage(&self) -> f32 {
        if self.probes_planned == 0 {
            return 1.0; // No probes needed
        }
        self.probes_valid as f32 / self.probes_planned as f32
    }
}

/// Unified reliability signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSignals {
    /// Translator was confident in classification
    pub translator_confident: bool,
    /// Probes covered required evidence
    pub probe_coverage: bool,
    /// Answer is grounded in evidence
    pub answer_grounded: bool,
    /// No invention detected
    pub no_invention: bool,
    /// No clarification needed
    pub clarification_not_needed: bool,
    /// Reliability score (0-100)
    pub score: u8,
    /// Score explanation
    pub explanation: String,
}

impl ResultSignals {
    /// Build signals from evidence and outcome
    pub fn build(evidence: &EvidenceSummary, outcome: Outcome) -> Self {
        let translator_confident = evidence.translator_confidence >= 0.7;

        let probe_coverage = evidence.probe_coverage() >= 0.5;

        // Grounded = has valid evidence
        let answer_grounded = evidence.has_evidence();

        // No invention for deterministic or verified with evidence
        let no_invention = match outcome {
            Outcome::Deterministic => true,
            Outcome::Verified => evidence.has_evidence(),
            Outcome::Clarification => true, // Asking is not inventing
            _ => false,
        };

        // Clarification not needed (unless that's the outcome)
        let clarification_not_needed = outcome != Outcome::Clarification;

        // Calculate score
        let score = Self::calculate_score(
            translator_confident,
            probe_coverage,
            answer_grounded,
            no_invention,
            clarification_not_needed,
            outcome,
            evidence,
        );

        let explanation = Self::build_explanation(
            translator_confident,
            probe_coverage,
            answer_grounded,
            no_invention,
            outcome,
            evidence,
        );

        Self {
            translator_confident,
            probe_coverage,
            answer_grounded,
            no_invention,
            clarification_not_needed,
            score,
            explanation,
        }
    }

    fn calculate_score(
        translator_confident: bool,
        probe_coverage: bool,
        answer_grounded: bool,
        no_invention: bool,
        clarification_not_needed: bool,
        outcome: Outcome,
        evidence: &EvidenceSummary,
    ) -> u8 {
        // Base score by outcome
        let base: u8 = match outcome {
            Outcome::Verified => 80,
            Outcome::Deterministic => 85,
            Outcome::Clarification => 60,
            Outcome::Failed => 30,
            Outcome::Timeout => 20,
        };

        // Adjust based on signals
        let mut score: u8 = base;

        if !no_invention {
            score = score.saturating_sub(40); // Hard penalty
        }

        if answer_grounded {
            score = score.saturating_add(5);
        } else if evidence.probes_planned > 0 {
            score = score.saturating_sub(10);
        }

        if probe_coverage {
            score = score.saturating_add(5);
        }

        if translator_confident {
            score = score.saturating_add(5);
        }

        if !clarification_not_needed {
            score = score.min(70); // Cap for clarification
        }

        score.min(100)
    }

    fn build_explanation(
        translator_confident: bool,
        _probe_coverage: bool,
        answer_grounded: bool,
        no_invention: bool,
        outcome: Outcome,
        evidence: &EvidenceSummary,
    ) -> String {
        let mut parts = Vec::new();

        // Outcome description
        parts.push(match outcome {
            Outcome::Verified => "Verified answer".to_string(),
            Outcome::Deterministic => "Deterministic answer".to_string(),
            Outcome::Clarification => "Needs clarification".to_string(),
            Outcome::Failed => "Failed to verify".to_string(),
            Outcome::Timeout => "Request timed out".to_string(),
        });

        // Evidence summary
        if evidence.probes_executed > 0 {
            parts.push(format!(
                "{}/{} probes succeeded",
                evidence.probes_valid, evidence.probes_executed
            ));
        }

        // Signal flags
        if !answer_grounded && evidence.probes_planned > 0 {
            parts.push("ungrounded".to_string());
        }

        if !no_invention {
            parts.push("invention detected".to_string());
        }

        if !translator_confident && evidence.translator_used {
            parts.push("low translator confidence".to_string());
        }

        parts.join(", ")
    }

}

/// Builder for common scenarios
impl ResultSignals {
    /// Deterministic answer with evidence
    pub fn deterministic_with_evidence(evidence: &EvidenceSummary) -> Self {
        Self::build(evidence, Outcome::Deterministic)
    }

    /// Clarification required with evidence
    pub fn clarification_with_evidence(evidence: &EvidenceSummary) -> Self {
        Self::build(evidence, Outcome::Clarification)
    }

    /// Timeout with partial evidence
    pub fn timeout_with_evidence(evidence: &EvidenceSummary) -> Self {
        Self::build(evidence, Outcome::Timeout)
    }

    /// Failed verification
    pub fn failed(evidence: &EvidenceSummary) -> Self {
        Self::build(evidence, Outcome::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_with_evidence_is_grounded() {
        let evidence = EvidenceSummary::new()
            .with_evidence_kinds(vec![EvidenceKind::ToolExists]);

        let signals = ResultSignals::deterministic_with_evidence(&evidence);

        assert!(signals.answer_grounded);
        assert!(signals.no_invention);
        assert!(signals.score >= 80);
    }

    #[test]
    fn test_clarification_with_evidence_is_grounded() {
        let evidence = EvidenceSummary::from_probes(
            &[ProbeResult {
                command: "test".to_string(),
                exit_code: 0,
                stdout: "output".to_string(),
                stderr: String::new(),
                timing_ms: 100,
            }],
            1,
        );

        let signals = ResultSignals::clarification_with_evidence(&evidence);

        assert!(signals.answer_grounded);
        assert!(signals.no_invention);
        // Clarification caps score at 70
        assert!(signals.score <= 70);
    }

    #[test]
    fn test_no_evidence_not_grounded() {
        let evidence = EvidenceSummary::from_probes(&[], 2);

        let signals = ResultSignals::build(&evidence, Outcome::Verified);

        assert!(!signals.answer_grounded);
        assert!(!signals.probe_coverage);
    }

    #[test]
    fn test_invention_penalty() {
        let evidence = EvidenceSummary::new();

        // Failed outcome without evidence = invention
        let signals = ResultSignals::failed(&evidence);

        assert!(!signals.no_invention);
        assert!(signals.score < 50);
    }

    #[test]
    fn test_timeout_preserves_evidence() {
        let evidence = EvidenceSummary::from_probes(
            &[ProbeResult {
                command: "test".to_string(),
                exit_code: 0,
                stdout: "output".to_string(),
                stderr: String::new(),
                timing_ms: 100,
            }],
            1,
        );

        let signals = ResultSignals::timeout_with_evidence(&evidence);

        // Even timeout should show grounded if we have evidence
        assert!(signals.answer_grounded);
    }
}
