//! Review gate types (v0.0.228).

use crate::review::{ReviewDecision, ReviewIssueKind};
use crate::trace::{FallbackUsed, SpecialistOutcome};
use serde::{Deserialize, Serialize};

/// Context for review gate decision (all deterministic signals)
#[derive(Debug, Clone, Default)]
pub struct ReviewContext {
    /// Reliability score from compute_reliability (0-100)
    pub reliability_score: u8,
    /// Grounding ratio from ANCHOR (0.0-1.0)
    pub grounding_ratio: f32,
    /// Total claims extracted
    pub total_claims: u32,
    /// Whether invention was detected (GUARD)
    pub invention_detected: bool,
    /// Number of contradictions found
    pub contradictions: u32,
    /// Number of unverifiable specifics
    pub unverifiable_specifics: u32,
    /// Whether evidence was required for this query type
    pub evidence_required: bool,
    /// Whether stage budget was exceeded
    pub budget_exceeded: bool,
    /// Specialist outcome from trace
    pub specialist_outcome: Option<SpecialistOutcome>,
    /// Fallback used (deterministic, timeout, etc.)
    pub fallback_used: Option<FallbackUsed>,
    /// Whether transcript was capped
    pub transcript_capped: bool,
    /// Whether prompt was truncated
    pub prompt_truncated: bool,
}

impl ReviewContext {
    /// Create new context with score
    pub fn new(reliability_score: u8) -> Self {
        Self {
            reliability_score,
            ..Default::default()
        }
    }

    /// Set grounding info
    pub fn with_grounding(mut self, ratio: f32, claims: u32) -> Self {
        self.grounding_ratio = ratio;
        self.total_claims = claims;
        self
    }

    /// Set guard info
    pub fn with_guard(mut self, invention: bool, contradictions: u32, unverifiable: u32) -> Self {
        self.invention_detected = invention;
        self.contradictions = contradictions;
        self.unverifiable_specifics = unverifiable;
        self
    }

    /// Set evidence_required
    pub fn with_evidence_required(mut self, required: bool) -> Self {
        self.evidence_required = required;
        self
    }

    /// Set fallback info
    pub fn with_fallback(mut self, fallback: FallbackUsed) -> Self {
        self.fallback_used = Some(fallback);
        self
    }

    /// Set budget exceeded
    pub fn with_budget_exceeded(mut self, exceeded: bool) -> Self {
        self.budget_exceeded = exceeded;
        self
    }
}

/// Outcome of deterministic gate (before any LLM review)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateOutcome {
    /// Decision from the gate
    pub decision: ReviewDecision,
    /// Reasons for the decision
    pub reasons: Vec<ReviewIssueKind>,
    /// Whether LLM review is required
    pub requires_llm_review: bool,
    /// Confidence in the decision (0.0-1.0)
    pub confidence: f32,
}

impl GateOutcome {
    /// Create an Accept outcome
    pub fn accept() -> Self {
        Self {
            decision: ReviewDecision::Accept,
            reasons: Vec::new(),
            requires_llm_review: false,
            confidence: 1.0,
        }
    }

    /// Create an Accept outcome with fallback tag
    pub fn accept_with_fallback() -> Self {
        Self {
            decision: ReviewDecision::Accept,
            reasons: Vec::new(),
            requires_llm_review: false,
            confidence: 0.85, // Lower confidence due to fallback
        }
    }

    /// Create a Revise outcome
    pub fn revise(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::Revise,
            reasons,
            requires_llm_review: false,
            confidence: 0.9,
        }
    }

    /// Create an Escalate outcome
    pub fn escalate(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::EscalateToSenior,
            reasons,
            requires_llm_review: false,
            confidence: 0.95,
        }
    }

    /// Create an Unclear outcome (requires LLM review)
    pub fn unclear() -> Self {
        Self {
            decision: ReviewDecision::Revise,
            reasons: Vec::new(),
            requires_llm_review: true,
            confidence: 0.5,
        }
    }

    /// Create a ClarifyUser outcome
    pub fn clarify(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::ClarifyUser,
            reasons,
            requires_llm_review: false,
            confidence: 0.9,
        }
    }
}

/// Thresholds for gate decisions (configurable)
#[derive(Debug, Clone)]
pub struct GateThresholds {
    /// Minimum score for accept (default: 80)
    pub accept_score: u8,
    /// Minimum grounding ratio (default: 0.5)
    pub min_grounding: f32,
    /// Score for accept with fallback (default: 70)
    pub fallback_accept_score: u8,
}

impl Default for GateThresholds {
    fn default() -> Self {
        Self {
            accept_score: 80,
            min_grounding: 0.5,
            fallback_accept_score: 70,
        }
    }
}
