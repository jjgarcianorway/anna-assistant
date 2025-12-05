//! Reliability scoring model.
//!
//! Pure function scoring with test-locked behavior.
//! Reason codes (not text) for determinism.
//! TRUST: Structured explanations when score < 80.
//! RESCUE Hardening: Explicit thresholds for warnings.

use crate::resource_limits::ResourceDiagnostic;
use crate::trace::{FallbackUsed, SpecialistOutcome};
use serde::{Deserialize, Serialize};

// =============================================================================
// RESCUE Hardening: Explicit threshold constants
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

/// Penalty for using deterministic fallback (specialist did not complete)
/// Small penalty - answer is still valid, just not LLM-enhanced
pub const PENALTY_FALLBACK_USED: i8 = -5;

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
    /// BudgetExceeded is higher priority than ProbeTimeout (subsumes it)
    pub fn priority(&self) -> u8 {
        match self {
            Self::InventionDetected => 0,
            Self::EvidenceMissing => 1,
            Self::BudgetExceeded => 2,  // Higher priority than ProbeTimeout
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
    /// TRUST: Deterministic, no freeform speculation.
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
    // Fallback context (TRUST+ phase)
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

    // Fallback context (TRUST+ phase)
    pub used_deterministic_fallback: bool,
    pub fallback_route_class: String,
    pub evidence_kinds: Vec<String>,

    // Trace context (v0.0.24) - source of truth for fallback guardrail
    /// Specialist outcome from ExecutionTrace
    pub specialist_outcome: Option<SpecialistOutcome>,
    /// Fallback used from ExecutionTrace
    pub fallback_used: Option<FallbackUsed>,
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

    // === Builder methods for testing (v0.0.45) ===

    /// Set whether evidence is required
    pub fn with_evidence_required(mut self, required: bool) -> Self {
        self.evidence_required = required;
        self
    }

    /// Set planned probes count
    pub fn with_planned_probes(mut self, count: usize) -> Self {
        self.planned_probes = count;
        self
    }

    /// Set succeeded probes count
    pub fn with_succeeded_probes(mut self, count: usize) -> Self {
        self.succeeded_probes = count;
        self
    }

    /// Set total claims count
    pub fn with_total_claims(mut self, count: u32) -> Self {
        self.total_claims = count;
        self
    }

    /// Set verified claims count (derives grounding_ratio)
    pub fn with_verified_claims(mut self, count: u32) -> Self {
        if self.total_claims > 0 {
            self.grounding_ratio = count as f32 / self.total_claims as f32;
        }
        self
    }

    /// Set answer_grounded flag
    pub fn with_answer_grounded(mut self, grounded: bool) -> Self {
        self.answer_grounded = grounded;
        self
    }

    /// Set no_invention flag (true = no invention detected)
    pub fn with_no_invention(mut self, no_invention: bool) -> Self {
        self.no_invention = no_invention;
        self
    }

    /// Set translator confidence (0.0 to 1.0)
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
    // If no_invention is false, clamp to INVENTION_CEILING
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

    // === Budget exceeded (METER phase) ===
    // BudgetExceeded subsumes ProbeTimeout - if budget exceeded, skip timeout penalty
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
        // No probes planned
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
        // Probes were planned - calculate coverage penalty
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

        // Extra penalty for timeouts (worse UX than failures)
        // NO DOUBLE PENALTY: Skip if budget_exceeded was already applied (subsumption)
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

    // === Translator confidence (only if translator used) ===
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

    // === Fallback penalty (v0.0.24) ===
    // GUARDRAIL: Only penalize when specialist did NOT complete and fallback was used.
    // Normal deterministic routing (specialist Skipped) is NOT penalized.
    let fallback_penalty_applies = match (&input.specialist_outcome, &input.fallback_used) {
        // Specialist didn't complete (Timeout/Error/BudgetExceeded) AND fallback was used
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
