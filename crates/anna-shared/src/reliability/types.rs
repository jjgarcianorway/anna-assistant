//! Reliability input/output types.
//! v0.0.119: Extracted from reliability.rs for modularity.

use serde::{Deserialize, Serialize};

use super::reasons::ReliabilityReason;
use crate::trace::{FallbackUsed, SpecialistOutcome};

/// Probe execution health state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeHealth {
    /// All planned probes succeeded
    AllOk,
    /// Some probes succeeded, some failed/timed out
    Partial,
    /// No probes succeeded (all failed/timed out)
    None,
    /// No probes were needed/planned
    NotNeeded,
}

/// Input to reliability computation (all the raw signals)
#[derive(Debug, Clone, Default)]
pub struct ReliabilityInput {
    // Probe signals
    pub planned_probes: usize,
    pub succeeded_probes: usize,
    pub failed_probes: usize,
    pub timed_out_probes: usize,

    // Translator signals
    pub translator_confidence: f32,
    pub translator_used: bool,

    // Answer quality signals
    pub answer_grounded: bool,
    pub no_invention: bool,

    // Grounding signals
    pub grounding_ratio: f32,
    pub total_claims: u32,

    // Evidence signals
    pub evidence_required: bool,

    // Resource signals
    pub prompt_truncated: bool,
    pub transcript_capped: bool,

    // Budget signals
    pub budget_exceeded: bool,
    pub exceeded_stage: Option<String>,
    pub stage_budget_ms: u64,
    pub stage_elapsed_ms: u64,

    // Deterministic path
    pub used_deterministic: bool,
    pub parsed_data_count: usize,

    // Fallback context
    pub used_deterministic_fallback: bool,
    pub fallback_route_class: String,
    pub evidence_kinds: Vec<String>,

    // Trace context
    pub specialist_outcome: Option<SpecialistOutcome>,
    pub fallback_used: Option<FallbackUsed>,
}

impl ReliabilityInput {
    /// Derive answer_grounded from grounding report.
    pub fn derive_answer_grounded(&mut self) {
        self.answer_grounded = self.total_claims > 0 && self.grounding_ratio >= 0.5;
    }

    /// Set grounding from a GroundingReport and derive answer_grounded.
    pub fn set_grounding(&mut self, total_claims: u32, verified_claims: u32) {
        self.total_claims = total_claims;
        self.grounding_ratio = if total_claims == 0 {
            0.0
        } else {
            verified_claims as f32 / total_claims as f32
        };
        self.derive_answer_grounded();
    }

    // === Builder methods for testing ===

    pub fn with_evidence_required(mut self, required: bool) -> Self {
        self.evidence_required = required;
        self
    }

    pub fn with_planned_probes(mut self, count: usize) -> Self {
        self.planned_probes = count;
        self
    }

    pub fn with_succeeded_probes(mut self, count: usize) -> Self {
        self.succeeded_probes = count;
        self
    }

    pub fn with_total_claims(mut self, count: u32) -> Self {
        self.total_claims = count;
        self
    }

    pub fn with_verified_claims(mut self, count: u32) -> Self {
        if self.total_claims > 0 {
            self.grounding_ratio = count as f32 / self.total_claims as f32;
        }
        self
    }

    pub fn with_answer_grounded(mut self, grounded: bool) -> Self {
        self.answer_grounded = grounded;
        self
    }

    pub fn with_no_invention(mut self, no_invention: bool) -> Self {
        self.no_invention = no_invention;
        self
    }

    pub fn with_translator_confidence(mut self, confidence: u8) -> Self {
        self.translator_confidence = confidence as f32 / 100.0;
        self.translator_used = true;
        self
    }
}

/// Breakdown item for debug mode
#[derive(Debug, Clone)]
pub struct ScoreComponent {
    pub name: &'static str,
    pub delta: i8,
    pub reason: Option<ReliabilityReason>,
}

/// Output of reliability computation
#[derive(Debug, Clone)]
pub struct ReliabilityOutput {
    pub score: u8,
    pub reasons: Vec<ReliabilityReason>,
    pub breakdown: Vec<ScoreComponent>,
    pub probe_health: ProbeHealth,
    pub probe_coverage_ratio: f32,
}

impl ReliabilityOutput {
    /// Get the highest-priority reason (for user display when score < 80)
    pub fn primary_reason(&self) -> Option<&ReliabilityReason> {
        self.reasons.iter().min_by_key(|r| r.priority())
    }

    /// Get user-facing explanation string (if score < threshold)
    pub fn explanation(&self, score_threshold: u8) -> Option<String> {
        if self.score >= score_threshold {
            return None;
        }
        self.primary_reason().map(|r| r.explanation().to_string())
    }
}
