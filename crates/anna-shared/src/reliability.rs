//! Reliability scoring model.
//!
//! Pure function scoring with test-locked behavior.
//! Reason codes (not text) for determinism.
//! TRUST: Structured explanations when score < 80.

use crate::resource_limits::ResourceDiagnostic;
use serde::{Deserialize, Serialize};

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
            Self::PromptTruncated => "context was truncated",
            Self::TranscriptCapped => "response was capped",
            Self::LowConfidence => "query interpretation uncertain",
            Self::NotGrounded => "answer not fully grounded in data",
        }
    }

    /// Priority for user display (lower = higher priority)
    /// BudgetExceeded is higher priority than ProbeTimeout (subsumes it)
    pub fn priority(&self) -> u8 {
        match self {
            Self::InventionDetected => 0,
            Self::EvidenceMissing => 1,
            Self::BudgetExceeded => 2,  // Higher priority than ProbeTimeout
            Self::ProbeTimeout => 3,
            Self::ProbeFailed => 4,
            Self::PromptTruncated => 5,
            Self::TranscriptCapped => 6,
            Self::LowConfidence => 7,
            Self::NotGrounded => 8,
        }
    }

    /// Templated detail message for this reason code.
    /// TRUST: Deterministic, no freeform speculation.
    pub fn detail_template(&self, context: &ReasonContext) -> String {
        match self {
            Self::InventionDetected => {
                "score capped at 40 due to detected assumptions".to_string()
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
            Self::PromptTruncated => {
                "specialist prompt exceeded size limit".to_string()
            }
            Self::TranscriptCapped => {
                "transcript exceeded event limit".to_string()
            }
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
    // Grounding context
    pub total_claims: u32,
    pub grounding_ratio: f32,
    // Budget context (METER phase)
    pub exceeded_stage: String,
    pub stage_budget_ms: u64,
    pub stage_elapsed_ms: u64,
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
        }
    }
}

// === TRUST: Structured explanation types ===

/// A single reason item in the explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonItem {
    /// The reason code
    pub code: ReliabilityReason,
    /// Penalty applied (negative), or None for ceilings
    pub penalty: Option<i32>,
    /// Templated detail message
    pub details: String,
}

/// Structured reliability explanation (TRUST phase)
/// Only populated when score < 80
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityExplanation {
    /// The reliability score being explained
    pub score: u8,
    /// 1-2 sentence summary (stable phrasing)
    pub summary: String,
    /// Reason items ordered by impact (highest priority first)
    pub reasons: Vec<ReasonItem>,
    /// Resource diagnostics from COST phase
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

        // Build reason items from breakdown, sorted by priority
        let mut reasons: Vec<ReasonItem> = output
            .breakdown
            .iter()
            .filter_map(|c| {
                c.reason.map(|code| ReasonItem {
                    code,
                    penalty: if c.name == "invention_ceiling" {
                        None // Ceiling, not penalty
                    } else {
                        Some(c.delta as i32)
                    },
                    details: code.detail_template(&context),
                })
            })
            .collect();

        // Sort by priority (stable ordering)
        reasons.sort_by_key(|r| r.code.priority());

        // Deduplicate by code (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        reasons.retain(|r| seen.insert(r.code));

        // Build summary
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

    // Check for invention ceiling
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

    // Answer quality signals (from grounding)
    pub answer_grounded: bool, // Derived: (grounding_ratio >= 0.5) && (total_claims > 0)
    pub no_invention: bool,

    // Grounding signals (ANCHOR phase)
    pub grounding_ratio: f32,  // verified_claims / total_claims
    pub total_claims: u32,     // Number of auditable claims extracted

    // Evidence signals
    pub evidence_required: bool,

    // Resource signals
    pub prompt_truncated: bool,
    pub transcript_capped: bool,

    // Budget signals (METER phase)
    pub budget_exceeded: bool,
    pub exceeded_stage: Option<String>,
    pub stage_budget_ms: u64,
    pub stage_elapsed_ms: u64,

    // Deterministic path
    pub used_deterministic: bool,
    pub parsed_data_count: usize,
}

impl ReliabilityInput {
    /// Derive answer_grounded from grounding report.
    /// Rule: answer_grounded = (grounding_ratio >= 0.5) && (total_claims > 0)
    /// This prevents gaming by making no-claims answers = NOT grounded.
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
    /// Final score 0-100
    pub score: u8,
    /// All reason codes that contributed to degradation
    pub reasons: Vec<ReliabilityReason>,
    /// Breakdown for debug mode
    pub breakdown: Vec<ScoreComponent>,
    /// Derived probe health
    pub probe_health: ProbeHealth,
    /// Probe coverage ratio [0.0, 1.0]
    pub probe_coverage_ratio: f32,
}

impl ReliabilityOutput {
    /// Get the highest-priority reason (for user display when score < 80)
    pub fn primary_reason(&self) -> Option<&ReliabilityReason> {
        self.reasons.iter().min_by_key(|r| r.priority())
    }

    /// Get user-facing explanation string (if score < 80)
    pub fn explanation(&self, score_threshold: u8) -> Option<String> {
        if self.score >= score_threshold {
            return None;
        }
        self.primary_reason().map(|r| r.explanation().to_string())
    }
}

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
    // If no_invention is false, clamp to max 40
    if !input.no_invention {
        let ceiling = 40i16;
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
        let delta = -30;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "not_grounded",
            delta,
            reason: Some(ReliabilityReason::NotGrounded),
        });
        reasons.push(ReliabilityReason::NotGrounded);
    }

    // === Budget exceeded (METER phase) ===
    // BudgetExceeded subsumes ProbeTimeout - if budget exceeded, skip timeout penalty
    let budget_penalty_applied = if input.budget_exceeded {
        // Budget exceeded penalty: -15 (more severe than probe timeout's -10)
        // Rationale: stage-level budget exceeded affects entire stage, not just one probe
        let delta = -15;
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
        // No probes planned
        if input.evidence_required {
            let delta = -25;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "evidence_missing",
                delta,
                reason: Some(ReliabilityReason::EvidenceMissing),
            });
            reasons.push(ReliabilityReason::EvidenceMissing);
        }
    } else {
        // Probes were planned - calculate coverage penalty
        let coverage_penalty = ((1.0 - probe_coverage_ratio) * 30.0).round() as i8;
        if coverage_penalty > 0 {
            score -= coverage_penalty as i16;
            breakdown.push(ScoreComponent {
                name: "probe_coverage",
                delta: -coverage_penalty,
                reason: Some(ReliabilityReason::ProbeFailed),
            });
            reasons.push(ReliabilityReason::ProbeFailed);
        }

        // Extra penalty for timeouts (worse UX than failures)
        // NO DOUBLE PENALTY: Skip if budget_exceeded was already applied (subsumption)
        if input.timed_out_probes > 0 && !budget_penalty_applied {
            let delta = -10;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "probe_timeout",
                delta,
                reason: Some(ReliabilityReason::ProbeTimeout),
            });
            reasons.push(ReliabilityReason::ProbeTimeout);
        }
    }

    // === Translator confidence (only if translator used) ===
    if input.translator_used {
        if input.translator_confidence < 0.7 {
            let delta = -20;
            score += delta as i16;
            breakdown.push(ScoreComponent {
                name: "low_confidence",
                delta,
                reason: Some(ReliabilityReason::LowConfidence),
            });
            reasons.push(ReliabilityReason::LowConfidence);
        } else if input.translator_confidence < 0.85 {
            let delta = -10;
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
        let delta = -10;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "prompt_truncated",
            delta,
            reason: Some(ReliabilityReason::PromptTruncated),
        });
        reasons.push(ReliabilityReason::PromptTruncated);
    }

    if input.transcript_capped {
        let delta = -5;
        score += delta as i16;
        breakdown.push(ScoreComponent {
            name: "transcript_capped",
            delta,
            reason: Some(ReliabilityReason::TranscriptCapped),
        });
        reasons.push(ReliabilityReason::TranscriptCapped);
    }

    // Clamp to valid range
    let score = score.clamp(0, 100) as u8;

    // Deduplicate reasons (keep order)
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
/// Used when translator doesn't provide probes but query clearly needs data.
pub fn query_requires_evidence(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Keywords that indicate data-dependent questions
    let evidence_keywords = [
        "what process",
        "which process",
        "how much memory",
        "how much ram",
        "how much disk",
        "how much cpu",
        "disk space",
        "disk usage",
        "memory usage",
        "cpu usage",
        "top process",
        "using the most",
        "consuming",
        "running",
        "listening",
        "what port",
        "network",
        "interface",
        "ip address",
        "current",
        "right now",
        "at the moment",
    ];

    evidence_keywords
        .iter()
        .any(|kw| query_lower.contains(kw))
}

// Golden tests are in tests/reliability_tests.rs
