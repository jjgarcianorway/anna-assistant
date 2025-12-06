//! Reliability scoring model.
//! v0.0.119: Split into modular files.
//!
//! Pure function scoring with test-locked behavior.
//! Reason codes (not text) for determinism.

// Re-export submodules
mod reasons;
mod types;

pub use reasons::*;
pub use types::*;

use crate::trace::{FallbackUsed, SpecialistOutcome};

// =============================================================================
// Threshold constants
// =============================================================================

/// Invention ceiling - score capped when invention detected
pub const INVENTION_CEILING: u8 = 40;

/// Penalty for ungrounded answer when evidence required
pub const PENALTY_NOT_GROUNDED: i8 = -30;

/// Penalty for budget exceeded
pub const PENALTY_BUDGET_EXCEEDED: i8 = -15;

/// Penalty for probe timeout
pub const PENALTY_PROBE_TIMEOUT: i8 = -10;

/// Penalty for probe truncation
pub const PENALTY_PROMPT_TRUNCATED: i8 = -10;

/// Penalty for transcript capped
pub const PENALTY_TRANSCRIPT_CAPPED: i8 = -5;

/// Penalty for low translator confidence (<70%)
pub const PENALTY_LOW_CONFIDENCE: i8 = -20;

/// Penalty for medium translator confidence (70-85%)
pub const PENALTY_MEDIUM_CONFIDENCE: i8 = -10;

/// Penalty for evidence missing when required
pub const PENALTY_EVIDENCE_MISSING: i8 = -25;

/// Maximum probe coverage penalty (100% missing = -30)
pub const MAX_PROBE_COVERAGE_PENALTY: f32 = 30.0;

/// Threshold for "low" translator confidence
pub const CONFIDENCE_LOW_THRESHOLD: f32 = 0.70;

/// Threshold for "medium" translator confidence
pub const CONFIDENCE_MEDIUM_THRESHOLD: f32 = 0.85;

/// Disk usage warning threshold (percentage)
pub const DISK_WARNING_THRESHOLD: u8 = 85;

/// Disk usage critical threshold (percentage)
pub const DISK_CRITICAL_THRESHOLD: u8 = 95;

/// Memory usage high threshold (percentage)
pub const MEMORY_HIGH_THRESHOLD: f32 = 0.90;

/// Penalty for using deterministic fallback
pub const PENALTY_FALLBACK_USED: i8 = -5;

/// Hard cap when evidence_required=true but no probes succeeded
pub const NO_EVIDENCE_RELIABILITY_CAP: u8 = 40;

// =============================================================================
// Core computation
// =============================================================================

/// Pure function: compute reliability from inputs.
/// Test-locked behavior - changes here require golden test updates.
pub fn compute_reliability(input: &ReliabilityInput) -> ReliabilityOutput {
    let mut score: i16 = 100;
    let mut reasons = Vec::new();
    let mut breakdown = Vec::new();

    // Compute probe health
    let probe_health = compute_probe_health(input);
    let probe_coverage_ratio = if input.planned_probes == 0 {
        1.0
    } else {
        input.succeeded_probes as f32 / input.planned_probes as f32
    };

    // === HARD CEILING: invention detection ===
    if !input.no_invention {
        let ceiling = INVENTION_CEILING as i16;
        if score > ceiling {
            let delta = ceiling - score;
            breakdown.push(ScoreComponent {
                name: "invention_ceiling",
                delta: delta as i8,
                reason: Some(ReliabilityReason::InventionDetected),
            });
            score = ceiling;
            reasons.push(ReliabilityReason::InventionDetected);
        }
    }

    // === Evidence grounding ===
    if !input.answer_grounded && input.evidence_required {
        let delta = PENALTY_NOT_GROUNDED;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "not_grounded",
            delta,
            reason: Some(ReliabilityReason::NotGrounded),
        });
        reasons.push(ReliabilityReason::NotGrounded);
    }

    // === Budget exceeded ===
    let budget_penalty_applied = if input.budget_exceeded {
        let delta = PENALTY_BUDGET_EXCEEDED;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "budget_exceeded",
            delta,
            reason: Some(ReliabilityReason::BudgetExceeded),
        });
        reasons.push(ReliabilityReason::BudgetExceeded);
        true
    } else {
        false
    };

    // === Probe contribution ===
    if input.planned_probes == 0 {
        if input.evidence_required {
            let delta = PENALTY_EVIDENCE_MISSING;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "evidence_missing",
                delta,
                reason: Some(ReliabilityReason::EvidenceMissing),
            });
            reasons.push(ReliabilityReason::EvidenceMissing);
        }
    } else {
        let coverage_penalty = ((1.0 - probe_coverage_ratio) * MAX_PROBE_COVERAGE_PENALTY).round() as i8;
        if coverage_penalty > 0 {
            score -= coverage_penalty as i16;
            breakdown.push(ScoreComponent {
                name: "probe_coverage",
                delta: -coverage_penalty,
                reason: Some(ReliabilityReason::ProbeFailed),
            });
            reasons.push(ReliabilityReason::ProbeFailed);
        }

        if input.timed_out_probes > 0 && !budget_penalty_applied {
            let delta = PENALTY_PROBE_TIMEOUT;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "probe_timeout",
                delta,
                reason: Some(ReliabilityReason::ProbeTimeout),
            });
            reasons.push(ReliabilityReason::ProbeTimeout);
        }
    }

    // === Translator confidence ===
    if input.translator_used {
        if input.translator_confidence < CONFIDENCE_LOW_THRESHOLD {
            let delta = PENALTY_LOW_CONFIDENCE;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "low_confidence",
                delta,
                reason: Some(ReliabilityReason::LowConfidence),
            });
            reasons.push(ReliabilityReason::LowConfidence);
        } else if input.translator_confidence < CONFIDENCE_MEDIUM_THRESHOLD {
            let delta = PENALTY_MEDIUM_CONFIDENCE;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "medium_confidence",
                delta,
                reason: Some(ReliabilityReason::LowConfidence),
            });
            reasons.push(ReliabilityReason::LowConfidence);
        }
    }

    // === Resource caps ===
    if input.prompt_truncated {
        let delta = PENALTY_PROMPT_TRUNCATED;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "prompt_truncated",
            delta,
            reason: Some(ReliabilityReason::PromptTruncated),
        });
        reasons.push(ReliabilityReason::PromptTruncated);
    }

    if input.transcript_capped {
        let delta = PENALTY_TRANSCRIPT_CAPPED;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "transcript_capped",
            delta,
            reason: Some(ReliabilityReason::TranscriptCapped),
        });
        reasons.push(ReliabilityReason::TranscriptCapped);
    }

    // === Fallback penalty ===
    let fallback_penalty_applies = match (&input.specialist_outcome, &input.fallback_used) {
        (Some(outcome), Some(FallbackUsed::Deterministic { .. })) => {
            !matches!(outcome, SpecialistOutcome::Ok | SpecialistOutcome::Skipped)
        }
        _ => false,
    };

    if fallback_penalty_applies {
        let delta = PENALTY_FALLBACK_USED;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "fallback_used",
            delta,
            reason: Some(ReliabilityReason::FallbackUsed),
        });
        reasons.push(ReliabilityReason::FallbackUsed);
    }

    // Clamp to valid range
    let score = score.clamp(0, 100) as u8;

    // Deduplicate reasons
    let mut seen = std::collections::HashSet::new();
    reasons.retain(|r| seen.insert(*r));

    ReliabilityOutput {
        score,
        reasons,
        breakdown,
        probe_health,
        probe_coverage_ratio,
    }
}

/// Derive probe health from input
fn compute_probe_health(input: &ReliabilityInput) -> ProbeHealth {
    if input.planned_probes == 0 {
        ProbeHealth::NotNeeded
    } else if input.succeeded_probes == input.planned_probes {
        ProbeHealth::AllOk
    } else if input.succeeded_probes == 0 {
        ProbeHealth::None
    } else {
        ProbeHealth::Partial
    }
}

/// Heuristic: does this query type require evidence?
pub fn query_requires_evidence(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    let evidence_keywords = [
        "what process", "which process", "how much memory", "how much ram",
        "how much disk", "how much cpu", "disk space", "disk usage",
        "memory usage", "cpu usage", "top process", "using the most",
        "consuming", "running", "listening", "what port", "network",
        "interface", "ip address", "current", "right now", "at the moment",
    ];

    evidence_keywords.iter().any(|kw| query_lower.contains(kw))
}
