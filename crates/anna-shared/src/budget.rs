//! METER: Stage-level latency budgets with explicit degradation.
//!
//! Provides configurable budgets per stage and budget enforcement.
//! Pure decision functions for testability.
//!
//! v0.0.36: Added ProbeBudget for controlling probe resource usage.
//! v0.0.41: Added LLM token budgets and timeout fallback constants.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// === LLM Token Budget Constants (v0.0.41) ===

/// Max tokens for LLM draft responses (keep tight for speed)
pub const LLM_MAX_DRAFT_TOKENS: u32 = 800;
/// Max tokens for LLM specialist responses
pub const LLM_MAX_SPECIALIST_TOKENS: u32 = 1200;
/// Max context tokens for local LLM (8k context models)
pub const LLM_MAX_CONTEXT_TOKENS: u32 = 6000;
/// Translator timeout in seconds (triggers fallback)
pub const TRANSLATOR_TIMEOUT_SECS: u64 = 30;
/// Specialist timeout in seconds (triggers graceful degradation)
pub const SPECIALIST_TIMEOUT_SECS: u64 = 45;

/// Stage names for budget tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Translator,
    Probes,
    Specialist,
    Supervisor,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Translator => write!(f, "translator"),
            Self::Probes => write!(f, "probes"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
        }
    }
}

// === Probe Budget (v0.0.36) ===

/// Budget for probe resource usage.
/// Limits the number of probes and total output size to prevent runaway costs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProbeBudget {
    /// Maximum number of probes to run
    pub max_probes: usize,
    /// Maximum total probe output in bytes
    pub max_output_bytes: usize,
    /// Per-probe output cap in bytes
    pub per_probe_cap_bytes: usize,
}

impl Default for ProbeBudget {
    fn default() -> Self {
        Self {
            max_probes: 4,              // Match fast path probe count
            max_output_bytes: 64_000,   // 64KB total
            per_probe_cap_bytes: 16_000, // 16KB per probe
        }
    }
}

impl ProbeBudget {
    /// Create a minimal probe budget for fast path queries
    pub fn fast_path() -> Self {
        Self {
            max_probes: 4,
            max_output_bytes: 32_000,
            per_probe_cap_bytes: 8_000,
        }
    }

    /// Create a standard probe budget for specialist queries
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create an extended probe budget for complex queries
    pub fn extended() -> Self {
        Self {
            max_probes: 6,
            max_output_bytes: 128_000,
            per_probe_cap_bytes: 32_000,
        }
    }

    /// Check if adding output would exceed budget
    pub fn would_exceed(&self, current_bytes: usize, new_bytes: usize) -> bool {
        current_bytes + new_bytes > self.max_output_bytes
    }

    /// Cap output to per-probe limit
    pub fn cap_output(&self, output: &str) -> String {
        if output.len() <= self.per_probe_cap_bytes {
            output.to_string()
        } else {
            let truncated = &output[..self.per_probe_cap_bytes];
            format!("{}... [truncated, {} bytes exceeded cap]",
                truncated, output.len() - self.per_probe_cap_bytes)
        }
    }
}

/// Result of probe budget check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeBudgetCheck {
    /// Within budget
    Ok,
    /// Probe count exceeded
    ProbeCountExceeded { limit: usize, attempted: usize },
    /// Output size exceeded
    OutputSizeExceeded { limit: usize, current: usize },
}

/// Configurable budgets per stage (in milliseconds).
///
/// Defaults are conservative to allow LLM response variation:
/// - translator: 5000ms (5s) - fast model, simple task
/// - probes: 12000ms (12s) - multiple probes can run in parallel
/// - specialist: 15000ms (15s) - larger model, complex reasoning
/// - supervisor: 8000ms (8s) - fast model, validation
/// - total: 25000ms (25s) - entire request
/// - margin: 1000ms (1s) - buffer for orchestration overhead
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StageBudget {
    /// Translator stage budget in ms
    pub translator_ms: u64,
    /// Probes stage budget in ms
    pub probes_ms: u64,
    /// Specialist stage budget in ms
    pub specialist_ms: u64,
    /// Supervisor stage budget in ms
    pub supervisor_ms: u64,
    /// Total request budget in ms
    pub total_ms: u64,
    /// Margin for orchestration overhead in ms
    pub margin_ms: u64,
}

impl Default for StageBudget {
    fn default() -> Self {
        Self {
            translator_ms: 5_000,
            probes_ms: 12_000,
            specialist_ms: 15_000,
            supervisor_ms: 8_000,
            total_ms: 25_000,
            margin_ms: 1_000,
        }
    }
}

impl StageBudget {
    /// Get budget for a specific stage.
    pub fn get(&self, stage: Stage) -> u64 {
        match stage {
            Stage::Translator => self.translator_ms,
            Stage::Probes => self.probes_ms,
            Stage::Specialist => self.specialist_ms,
            Stage::Supervisor => self.supervisor_ms,
        }
    }

    /// Effective total budget (total - margin).
    pub fn effective_total(&self) -> u64 {
        self.total_ms.saturating_sub(self.margin_ms)
    }
}

/// Budget check result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCheck {
    /// Within budget
    Ok,
    /// Stage budget exceeded
    StageExceeded {
        stage: Stage,
        budget_ms: u64,
        elapsed_ms: u64,
    },
    /// Total request budget exceeded
    TotalExceeded {
        budget_ms: u64,
        elapsed_ms: u64,
    },
}

impl BudgetCheck {
    /// Check if budget was exceeded.
    pub fn is_exceeded(&self) -> bool {
        !matches!(self, Self::Ok)
    }

    /// Get the stage if this is a stage-level exceed.
    pub fn exceeded_stage(&self) -> Option<Stage> {
        match self {
            Self::StageExceeded { stage, .. } => Some(*stage),
            _ => None,
        }
    }
}

/// Pure function: check if stage elapsed time exceeds budget.
/// Does NOT check total budget - use check_total for that.
pub fn check_stage_budget(
    stage: Stage,
    elapsed_ms: u64,
    budget: &StageBudget,
) -> BudgetCheck {
    let stage_budget = budget.get(stage);
    if elapsed_ms > stage_budget {
        BudgetCheck::StageExceeded {
            stage,
            budget_ms: stage_budget,
            elapsed_ms,
        }
    } else {
        BudgetCheck::Ok
    }
}

/// Pure function: check if total request time exceeds budget.
pub fn check_total_budget(
    elapsed_ms: u64,
    budget: &StageBudget,
) -> BudgetCheck {
    let effective = budget.effective_total();
    if elapsed_ms > effective {
        BudgetCheck::TotalExceeded {
            budget_ms: effective,
            elapsed_ms,
        }
    } else {
        BudgetCheck::Ok
    }
}

/// Budget enforcer for tracking stage timing.
/// Wraps Instant for real-time tracking with testable pure functions.
#[derive(Debug, Clone)]
pub struct BudgetEnforcer {
    /// When the request started
    request_start: Instant,
    /// Budget configuration
    budget: StageBudget,
    /// Current stage start time (if any)
    stage_start: Option<(Stage, Instant)>,
}

impl BudgetEnforcer {
    /// Create new enforcer with given budget.
    pub fn new(budget: StageBudget) -> Self {
        Self {
            request_start: Instant::now(),
            budget,
            stage_start: None,
        }
    }

    /// Create enforcer with default budget.
    pub fn with_defaults() -> Self {
        Self::new(StageBudget::default())
    }

    /// Start timing a stage.
    pub fn start_stage(&mut self, stage: Stage) {
        self.stage_start = Some((stage, Instant::now()));
    }

    /// End timing current stage and check budget.
    /// Returns BudgetCheck result.
    pub fn end_stage(&mut self) -> BudgetCheck {
        if let Some((stage, start)) = self.stage_start.take() {
            let elapsed = start.elapsed().as_millis() as u64;
            check_stage_budget(stage, elapsed, &self.budget)
        } else {
            BudgetCheck::Ok
        }
    }

    /// Check current stage budget without ending it.
    pub fn check_stage(&self) -> BudgetCheck {
        if let Some((stage, start)) = &self.stage_start {
            let elapsed = start.elapsed().as_millis() as u64;
            check_stage_budget(*stage, elapsed, &self.budget)
        } else {
            BudgetCheck::Ok
        }
    }

    /// Check total request budget.
    pub fn check_total(&self) -> BudgetCheck {
        let elapsed = self.request_start.elapsed().as_millis() as u64;
        check_total_budget(elapsed, &self.budget)
    }

    /// Get total elapsed time since request start.
    pub fn total_elapsed(&self) -> Duration {
        self.request_start.elapsed()
    }

    /// Get total elapsed time in milliseconds.
    pub fn total_elapsed_ms(&self) -> u64 {
        self.request_start.elapsed().as_millis() as u64
    }

    /// Get current stage elapsed time in milliseconds.
    pub fn stage_elapsed_ms(&self) -> Option<u64> {
        self.stage_start.as_ref().map(|(_, start)| {
            start.elapsed().as_millis() as u64
        })
    }

    /// Get the budget configuration.
    pub fn budget(&self) -> &StageBudget {
        &self.budget
    }

    /// Check remaining budget for a stage.
    /// Returns 0 if budget already exceeded.
    pub fn remaining_stage_budget(&self, stage: Stage) -> u64 {
        let stage_budget = self.budget.get(stage);
        if let Some((current_stage, start)) = &self.stage_start {
            if *current_stage == stage {
                let elapsed = start.elapsed().as_millis() as u64;
                return stage_budget.saturating_sub(elapsed);
            }
        }
        stage_budget
    }

    /// Check remaining total budget.
    /// Returns 0 if budget already exceeded.
    pub fn remaining_total_budget(&self) -> u64 {
        let effective = self.budget.effective_total();
        let elapsed = self.total_elapsed_ms();
        effective.saturating_sub(elapsed)
    }
}

/// Stage timing result for logging/diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    pub stage: Stage,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub exceeded: bool,
}

impl StageTiming {
    pub fn new(stage: Stage, elapsed_ms: u64, budget: &StageBudget) -> Self {
        let budget_ms = budget.get(stage);
        Self {
            stage,
            elapsed_ms,
            budget_ms,
            exceeded: elapsed_ms > budget_ms,
        }
    }
}

// === LLM Budget (v0.0.41) ===

/// Configurable LLM token budgets for local inference (v0.0.41)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LlmBudget {
    /// Max tokens for draft/translation responses
    pub max_draft_tokens: u32,
    /// Max tokens for specialist responses
    pub max_specialist_tokens: u32,
    /// Max context tokens (prompt + response)
    pub max_context_tokens: u32,
    /// Translator timeout in seconds
    pub translator_timeout_secs: u64,
    /// Specialist timeout in seconds
    pub specialist_timeout_secs: u64,
}

impl Default for LlmBudget {
    fn default() -> Self {
        Self {
            max_draft_tokens: LLM_MAX_DRAFT_TOKENS,
            max_specialist_tokens: LLM_MAX_SPECIALIST_TOKENS,
            max_context_tokens: LLM_MAX_CONTEXT_TOKENS,
            translator_timeout_secs: TRANSLATOR_TIMEOUT_SECS,
            specialist_timeout_secs: SPECIALIST_TIMEOUT_SECS,
        }
    }
}

impl LlmBudget {
    /// Create tight budget for fast path (minimal tokens)
    pub fn fast_path() -> Self {
        Self {
            max_draft_tokens: 400,
            max_specialist_tokens: 600,
            max_context_tokens: 4000,
            translator_timeout_secs: 15,
            specialist_timeout_secs: 20,
        }
    }

    /// Create standard budget for normal queries
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create extended budget for complex queries
    pub fn extended() -> Self {
        Self {
            max_draft_tokens: 1200,
            max_specialist_tokens: 2000,
            max_context_tokens: 8000,
            translator_timeout_secs: 60,
            specialist_timeout_secs: 90,
        }
    }

    /// Check if elapsed time exceeds translator timeout
    pub fn is_translator_timeout(&self, elapsed_secs: u64) -> bool {
        elapsed_secs > self.translator_timeout_secs
    }

    /// Check if elapsed time exceeds specialist timeout
    pub fn is_specialist_timeout(&self, elapsed_secs: u64) -> bool {
        elapsed_secs > self.specialist_timeout_secs
    }
}

/// Result of LLM fallback decision (v0.0.41)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmFallback {
    /// Continue normally
    Continue,
    /// Translator timed out, use fallback interpretation
    TranslatorTimeout { elapsed_secs: u64 },
    /// Specialist timed out, return raw probe data
    SpecialistTimeout { elapsed_secs: u64 },
}

impl LlmFallback {
    /// Check if any timeout occurred
    pub fn is_timeout(&self) -> bool {
        !matches!(self, Self::Continue)
    }

    /// Get fallback message for user display
    pub fn fallback_message(&self) -> Option<String> {
        match self {
            Self::Continue => None,
            Self::TranslatorTimeout { elapsed_secs } => Some(format!(
                "Request interpretation took too long ({}s). Using simplified routing.",
                elapsed_secs
            )),
            Self::SpecialistTimeout { elapsed_secs } => Some(format!(
                "Analysis took too long ({}s). Here's the raw system data:",
                elapsed_secs
            )),
        }
    }
}

/// Check if should fall back based on elapsed time (v0.0.41)
pub fn check_llm_fallback(
    stage: Stage,
    elapsed_secs: u64,
    budget: &LlmBudget,
) -> LlmFallback {
    match stage {
        Stage::Translator => {
            if budget.is_translator_timeout(elapsed_secs) {
                LlmFallback::TranslatorTimeout { elapsed_secs }
            } else {
                LlmFallback::Continue
            }
        }
        Stage::Specialist => {
            if budget.is_specialist_timeout(elapsed_secs) {
                LlmFallback::SpecialistTimeout { elapsed_secs }
            } else {
                LlmFallback::Continue
            }
        }
        _ => LlmFallback::Continue,
    }
}

// Tests moved to tests/budget_tests.rs
