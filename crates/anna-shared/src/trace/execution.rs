//! ExecutionTrace struct and constructors (v0.0.184).

use serde::{Deserialize, Serialize};

use super::evidence::EvidenceKind;
use super::outcomes::{FallbackUsed, ReviewerOutcome, SpecialistOutcome};
use super::probe_stats::ProbeStats;

/// Full execution trace for a request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Outcome of the specialist stage
    pub specialist_outcome: SpecialistOutcome,
    /// What fallback was used (if any)
    pub fallback_used: FallbackUsed,
    /// Probe execution statistics
    pub probe_stats: ProbeStats,
    /// Evidence kinds parsed from probe data
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Whether the final answer came from deterministic path
    pub answer_is_deterministic: bool,
    /// Outcome of review stage (v0.0.26)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_outcome: Option<ReviewerOutcome>,
}

impl ExecutionTrace {
    /// Create a trace for successful specialist response
    pub fn specialist_ok(probe_stats: ProbeStats) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Ok,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds: vec![],
            answer_is_deterministic: false,
            reviewer_outcome: None,
        }
    }

    /// Create a trace for skipped specialist (deterministic route)
    pub fn deterministic_route(
        _route_class: &str,
        probe_stats: ProbeStats,
        evidence_kinds: Vec<EvidenceKind>,
    ) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Skipped,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
            reviewer_outcome: None,
        }
    }

    /// Create a trace for specialist timeout with fallback
    pub fn specialist_timeout_with_fallback(
        route_class: &str,
        probe_stats: ProbeStats,
        evidence_kinds: Vec<EvidenceKind>,
    ) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Timeout,
            fallback_used: FallbackUsed::Deterministic {
                route_class: route_class.to_string(),
            },
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
            reviewer_outcome: None,
        }
    }

    /// Create a trace for specialist error with fallback
    pub fn specialist_error_with_fallback(
        route_class: &str,
        probe_stats: ProbeStats,
        evidence_kinds: Vec<EvidenceKind>,
    ) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Error,
            fallback_used: FallbackUsed::Deterministic {
                route_class: route_class.to_string(),
            },
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
            reviewer_outcome: None,
        }
    }

    /// Create a trace for specialist timeout without successful fallback
    pub fn specialist_timeout_no_fallback(probe_stats: ProbeStats) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Timeout,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds: vec![],
            answer_is_deterministic: false,
            reviewer_outcome: None,
        }
    }

    /// Create a trace for global request timeout (v0.0.34)
    pub fn global_timeout(timeout_secs: u64) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Timeout,
            fallback_used: FallbackUsed::Timeout {
                route_class: "global".to_string(),
                timeout_ms: timeout_secs * 1000,
            },
            probe_stats: ProbeStats::default(),
            evidence_kinds: vec![],
            answer_is_deterministic: true, // The timeout response is deterministic
            reviewer_outcome: None,
        }
    }

    /// Set reviewer outcome (v0.0.26)
    pub fn with_reviewer_outcome(mut self, outcome: ReviewerOutcome) -> Self {
        self.reviewer_outcome = Some(outcome);
        self
    }
}

impl std::fmt::Display for ExecutionTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path: ")?;
        if self.answer_is_deterministic {
            match &self.fallback_used {
                FallbackUsed::None => write!(f, "deterministic route")?,
                FallbackUsed::Deterministic { route_class } => {
                    write!(f, "deterministic fallback ({})", route_class)?
                }
                FallbackUsed::Timeout {
                    route_class,
                    timeout_ms,
                } => write!(f, "timeout fallback ({}ms, {})", timeout_ms, route_class)?,
            }
        } else {
            write!(f, "specialist")?;
        }

        write!(f, ", specialist: {}", self.specialist_outcome)?;

        if !self.evidence_kinds.is_empty() {
            let kinds: Vec<_> = self.evidence_kinds.iter().map(|k| k.to_string()).collect();
            write!(f, ", evidence: [{}]", kinds.join(", "))?;
        }

        Ok(())
    }
}
