//! Reliability reason codes and explanations.
//! v0.0.119: Extracted from reliability.rs for modularity.

use serde::{Deserialize, Serialize};

use super::{INVENTION_CEILING, ReliabilityInput, ReliabilityOutput};
use crate::resource_limits::ResourceDiagnostic;

/// Reason codes for reliability degradation.
/// Stored as codes, mapped to text at the edge.
/// Priority order matters - first is highest priority for user display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReliabilityReason {
    /// Answer contains hedging/invention language (hard ceiling)
    InventionDetected,
    /// Evidence was needed but not available
    EvidenceMissing,
    /// Stage budget exceeded (METER phase) - subsumes ProbeTimeout
    BudgetExceeded,
    /// One or more probes timed out
    ProbeTimeout,
    /// One or more probes failed (non-zero exit)
    ProbeFailed,
    /// Deterministic fallback was used (specialist did not complete)
    FallbackUsed,
    /// Specialist prompt was truncated
    PromptTruncated,
    /// Transcript was capped at size limit
    TranscriptCapped,
    /// Translator confidence was low
    LowConfidence,
    /// Answer not grounded in probe data
    NotGrounded,
}

impl ReliabilityReason {
    /// User-facing explanation (single line, lowercase start)
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::InventionDetected => "answer may contain assumptions",
            Self::EvidenceMissing => "limited evidence available",
            Self::BudgetExceeded => "stage budget exceeded",
            Self::ProbeTimeout => "probe timed out",
            Self::ProbeFailed => "probe failed",
            Self::FallbackUsed => "used deterministic fallback",
            Self::PromptTruncated => "context was truncated",
            Self::TranscriptCapped => "response was capped",
            Self::LowConfidence => "query interpretation uncertain",
            Self::NotGrounded => "answer not fully grounded in data",
        }
    }

    /// Priority for user display (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::InventionDetected => 0,
            Self::EvidenceMissing => 1,
            Self::BudgetExceeded => 2,
            Self::ProbeTimeout => 3,
            Self::ProbeFailed => 4,
            Self::FallbackUsed => 5,
            Self::PromptTruncated => 6,
            Self::TranscriptCapped => 7,
            Self::LowConfidence => 8,
            Self::NotGrounded => 9,
        }
    }

    /// Templated detail message for this reason code.
    pub fn detail_template(&self, context: &ReasonContext) -> String {
        match self {
            Self::InventionDetected => {
                format!("score capped at {} due to detected assumptions", INVENTION_CEILING)
            }
            Self::EvidenceMissing => {
                format!(
                    "query requires evidence but {} probes were planned",
                    context.planned_probes
                )
            }
            Self::BudgetExceeded => {
                format!(
                    "{} stage exceeded budget ({}ms > {}ms)",
                    context.exceeded_stage,
                    context.stage_elapsed_ms,
                    context.stage_budget_ms
                )
            }
            Self::ProbeTimeout => {
                format!(
                    "{} of {} probes timed out",
                    context.timed_out_probes, context.planned_probes
                )
            }
            Self::ProbeFailed => {
                format!(
                    "{} of {} probes succeeded (coverage {:.0}%)",
                    context.succeeded_probes,
                    context.planned_probes,
                    context.probe_coverage_ratio * 100.0
                )
            }
            Self::FallbackUsed => {
                if !context.evidence_kinds.is_empty() {
                    format!(
                        "specialist did not complete; used {} fallback with {} evidence",
                        context.fallback_route_class,
                        context.evidence_kinds.join(", ")
                    )
                } else {
                    format!(
                        "specialist did not complete; used {} fallback",
                        context.fallback_route_class
                    )
                }
            }
            Self::PromptTruncated => "specialist prompt exceeded size limit".to_string(),
            Self::TranscriptCapped => "transcript exceeded event limit".to_string(),
            Self::LowConfidence => {
                format!(
                    "translator confidence {:.0}% below threshold",
                    context.translator_confidence * 100.0
                )
            }
            Self::NotGrounded => {
                if context.total_claims == 0 {
                    "answer contains no verifiable claims".to_string()
                } else {
                    format!(
                        "grounding ratio {:.0}% ({} claims verified)",
                        context.grounding_ratio * 100.0,
                        (context.grounding_ratio * context.total_claims as f32).round() as u32
                    )
                }
            }
        }
    }
}

/// Context for generating reason details (numeric facts only)
#[derive(Debug, Clone, Default)]
pub struct ReasonContext {
    pub planned_probes: usize,
    pub succeeded_probes: usize,
    pub timed_out_probes: usize,
    pub translator_confidence: f32,
    pub probe_coverage_ratio: f32,
    pub total_claims: u32,
    pub grounding_ratio: f32,
    pub exceeded_stage: String,
    pub stage_budget_ms: u64,
    pub stage_elapsed_ms: u64,
    pub used_deterministic_fallback: bool,
    pub fallback_route_class: String,
    pub evidence_kinds: Vec<String>,
}

impl ReasonContext {
    /// Build context from ReliabilityInput
    pub fn from_input(input: &ReliabilityInput, coverage: f32) -> Self {
        Self {
            planned_probes: input.planned_probes,
            succeeded_probes: input.succeeded_probes,
            timed_out_probes: input.timed_out_probes,
            translator_confidence: input.translator_confidence,
            probe_coverage_ratio: coverage,
            total_claims: input.total_claims,
            grounding_ratio: input.grounding_ratio,
            exceeded_stage: input.exceeded_stage.clone().unwrap_or_default(),
            stage_budget_ms: input.stage_budget_ms,
            stage_elapsed_ms: input.stage_elapsed_ms,
            used_deterministic_fallback: input.used_deterministic_fallback,
            fallback_route_class: input.fallback_route_class.clone(),
            evidence_kinds: input.evidence_kinds.clone(),
        }
    }
}

/// A single reason item in the explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonItem {
    pub code: ReliabilityReason,
    pub penalty: Option<i32>,
    pub details: String,
}

/// Structured reliability explanation
/// Only populated when score < EXPLANATION_THRESHOLD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityExplanation {
    pub score: u8,
    pub summary: String,
    pub reasons: Vec<ReasonItem>,
    pub diagnostics: Vec<ResourceDiagnostic>,
}

/// Threshold below which explanations are generated
pub const EXPLANATION_THRESHOLD: u8 = 80;

impl ReliabilityExplanation {
    /// Build explanation from output and input context.
    /// Returns None if score >= EXPLANATION_THRESHOLD.
    pub fn build(
        output: &ReliabilityOutput,
        input: &ReliabilityInput,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<Self> {
        if output.score >= EXPLANATION_THRESHOLD {
            return None;
        }

        let context = ReasonContext::from_input(input, output.probe_coverage_ratio);

        let mut reasons: Vec<ReasonItem> = output
            .breakdown
            .iter()
            .filter_map(|c| {
                c.reason.map(|code| ReasonItem {
                    code,
                    penalty: if c.name == "invention_ceiling" {
                        None
                    } else {
                        Some(c.delta as i32)
                    },
                    details: code.detail_template(&context),
                })
            })
            .collect();

        reasons.sort_by_key(|r| r.code.priority());

        let mut seen = std::collections::HashSet::new();
        reasons.retain(|r| seen.insert(r.code));

        let summary = build_summary(output, &reasons);

        Some(Self {
            score: output.score,
            summary,
            reasons,
            diagnostics,
        })
    }
}

/// Build a 1-2 sentence summary from the explanation data
fn build_summary(output: &ReliabilityOutput, reasons: &[ReasonItem]) -> String {
    if reasons.is_empty() {
        return format!("Reliability score {} (no specific issues identified).", output.score);
    }

    let has_invention = reasons.iter().any(|r| r.code == ReliabilityReason::InventionDetected);

    if has_invention {
        let other_count = reasons.len() - 1;
        if other_count > 0 {
            format!(
                "Reliability score {} (capped at 40 due to detected assumptions; {} other issue{} also present).",
                output.score,
                other_count,
                if other_count == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Reliability score {} (capped at 40 due to detected assumptions).",
                output.score
            )
        }
    } else {
        let primary = &reasons[0];
        if reasons.len() == 1 {
            format!(
                "Reliability score {}: {}.",
                output.score,
                primary.code.explanation()
            )
        } else {
            format!(
                "Reliability score {}: {} (+{} other issue{}).",
                output.score,
                primary.code.explanation(),
                reasons.len() - 1,
                if reasons.len() == 2 { "" } else { "s" }
            )
        }
    }
}
