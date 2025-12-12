//! Time budgets and retry strategy (v0.0.433).
//!
//! Enforces strict time limits for each processing stage.

use super::contract::{SpecialistResult, TicketOutcome};
use super::{
    GLOBAL_HARD_CAP_MS, JUNIOR_HARD_CAP_MS, JUNIOR_SOFT_BUDGET_MS, MAX_PARSE_RETRIES,
    SENIOR_HARD_CAP_MS, SENIOR_SOFT_BUDGET_MS, TRANSLATOR_HARD_CAP_MS,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Processing stage for timeout tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeoutStage {
    /// Translator classification.
    Translator,
    /// Junior specialist routing.
    JuniorRouting,
    /// Junior specialist LLM call.
    JuniorLlm,
    /// Senior specialist routing.
    SeniorRouting,
    /// Senior specialist LLM call.
    SeniorLlm,
    /// JSON parsing.
    Parse,
    /// Probe execution.
    Probes,
    /// Knowledge lookup.
    Knowledge,
    /// Overall ticket processing.
    Global,
}

impl TimeoutStage {
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Translator => "translator_classification",
            Self::JuniorRouting => "junior_routing",
            Self::JuniorLlm => "junior_llm_parse",
            Self::SeniorRouting => "senior_routing",
            Self::SeniorLlm => "senior_llm_parse",
            Self::Parse => "json_parse",
            Self::Probes => "probe_execution",
            Self::Knowledge => "knowledge_lookup",
            Self::Global => "global",
        }
    }
}

/// Time budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBudget {
    /// Soft budget (warning threshold).
    pub soft_ms: u64,
    /// Hard cap (absolute limit).
    pub hard_ms: u64,
}

impl TimeBudget {
    /// Create a new budget.
    pub fn new(soft_ms: u64, hard_ms: u64) -> Self {
        Self { soft_ms, hard_ms }
    }

    /// Check if time has exceeded soft budget.
    pub fn is_over_soft(&self, elapsed_ms: u64) -> bool {
        elapsed_ms > self.soft_ms
    }

    /// Check if time has exceeded hard cap.
    pub fn is_over_hard(&self, elapsed_ms: u64) -> bool {
        elapsed_ms > self.hard_ms
    }

    /// Remaining time before hard cap.
    pub fn remaining_ms(&self, elapsed_ms: u64) -> u64 {
        self.hard_ms.saturating_sub(elapsed_ms)
    }
}

/// Full timeout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Translator budget.
    pub translator: TimeBudget,
    /// Junior specialist budget.
    pub junior: TimeBudget,
    /// Senior specialist budget.
    pub senior: TimeBudget,
    /// Global ticket budget.
    pub global: TimeBudget,
    /// Maximum retries for parse errors.
    pub max_parse_retries: usize,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            translator: TimeBudget::new(TRANSLATOR_HARD_CAP_MS, TRANSLATOR_HARD_CAP_MS),
            junior: TimeBudget::new(JUNIOR_SOFT_BUDGET_MS, JUNIOR_HARD_CAP_MS),
            senior: TimeBudget::new(SENIOR_SOFT_BUDGET_MS, SENIOR_HARD_CAP_MS),
            global: TimeBudget::new(GLOBAL_HARD_CAP_MS, GLOBAL_HARD_CAP_MS),
            max_parse_retries: MAX_PARSE_RETRIES,
        }
    }
}

/// Timing for a single stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    /// Which stage.
    pub stage: TimeoutStage,
    /// Start time (unix ms).
    pub started_ms: u64,
    /// Duration in ms.
    pub duration_ms: u64,
    /// Whether it exceeded soft budget.
    pub over_soft: bool,
    /// Whether it hit hard cap (timed out).
    pub timed_out: bool,
}

/// Retry strategy for failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// No retry allowed.
    None,
    /// Retry once with stricter prompt.
    OnceWithStricterPrompt,
    /// Retry with exponential backoff.
    ExponentialBackoff { max_attempts: usize },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::OnceWithStricterPrompt
    }
}

/// Timeout enforcer for ticket processing.
pub struct TimeoutEnforcer {
    config: TimeoutConfig,
    global_start: Instant,
    stage_timings: Vec<StageTiming>,
    current_stage: Option<(TimeoutStage, Instant)>,
    retries_attempted: usize,
}

impl TimeoutEnforcer {
    /// Create a new enforcer with default config.
    pub fn new() -> Self {
        Self::with_config(TimeoutConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: TimeoutConfig) -> Self {
        Self {
            config,
            global_start: Instant::now(),
            stage_timings: Vec::new(),
            current_stage: None,
            retries_attempted: 0,
        }
    }

    /// Start timing a stage.
    pub fn start_stage(&mut self, stage: TimeoutStage) {
        // End current stage if any
        self.end_current_stage();
        self.current_stage = Some((stage, Instant::now()));
    }

    /// End the current stage.
    pub fn end_current_stage(&mut self) {
        if let Some((stage, start)) = self.current_stage.take() {
            let duration_ms = start.elapsed().as_millis() as u64;
            let budget = self.budget_for_stage(stage);

            self.stage_timings.push(StageTiming {
                stage,
                started_ms: start.elapsed().as_millis() as u64,
                duration_ms,
                over_soft: budget.is_over_soft(duration_ms),
                timed_out: budget.is_over_hard(duration_ms),
            });
        }
    }

    /// Get budget for a stage.
    fn budget_for_stage(&self, stage: TimeoutStage) -> &TimeBudget {
        match stage {
            TimeoutStage::Translator => &self.config.translator,
            TimeoutStage::JuniorRouting | TimeoutStage::JuniorLlm => &self.config.junior,
            TimeoutStage::SeniorRouting | TimeoutStage::SeniorLlm => &self.config.senior,
            TimeoutStage::Global => &self.config.global,
            _ => &self.config.junior, // Default for other stages
        }
    }

    /// Check if global timeout exceeded.
    pub fn is_global_timeout(&self) -> bool {
        self.global_elapsed_ms() > self.config.global.hard_ms
    }

    /// Global elapsed time in ms.
    pub fn global_elapsed_ms(&self) -> u64 {
        self.global_start.elapsed().as_millis() as u64
    }

    /// Remaining global time.
    pub fn remaining_global_ms(&self) -> u64 {
        self.config.global.hard_ms.saturating_sub(self.global_elapsed_ms())
    }

    /// Check if current stage is over soft budget.
    pub fn is_current_over_soft(&self) -> bool {
        if let Some((stage, start)) = &self.current_stage {
            let elapsed = start.elapsed().as_millis() as u64;
            self.budget_for_stage(*stage).is_over_soft(elapsed)
        } else {
            false
        }
    }

    /// Check if current stage is over hard cap.
    pub fn is_current_over_hard(&self) -> bool {
        if let Some((stage, start)) = &self.current_stage {
            let elapsed = start.elapsed().as_millis() as u64;
            self.budget_for_stage(*stage).is_over_hard(elapsed)
        } else {
            false
        }
    }

    /// Wrap a timeout as a SpecialistResult.
    pub fn wrap_timeout(&self, stage: TimeoutStage) -> SpecialistResult {
        SpecialistResult::timeout(stage.name())
    }

    /// Check if retry is allowed for parse error.
    pub fn can_retry_parse(&self) -> bool {
        self.retries_attempted < self.config.max_parse_retries && !self.is_global_timeout()
    }

    /// Record a retry attempt.
    pub fn record_retry(&mut self) {
        self.retries_attempted += 1;
    }

    /// Get retry count.
    pub fn retry_count(&self) -> usize {
        self.retries_attempted
    }

    /// Get all stage timings.
    pub fn timings(&self) -> &[StageTiming] {
        &self.stage_timings
    }

    /// Get timing summary.
    pub fn timing_summary(&self) -> TimingSummary {
        self.end_current_stage_snapshot();

        let mut summary = TimingSummary::default();
        for timing in &self.stage_timings {
            match timing.stage {
                TimeoutStage::Translator => summary.translator_ms += timing.duration_ms,
                TimeoutStage::JuniorRouting | TimeoutStage::JuniorLlm => {
                    summary.junior_llm_ms += timing.duration_ms
                }
                TimeoutStage::SeniorRouting | TimeoutStage::SeniorLlm => {
                    summary.senior_llm_ms += timing.duration_ms
                }
                TimeoutStage::Parse => summary.parse_ms += timing.duration_ms,
                TimeoutStage::Probes => summary.probes_ms += timing.duration_ms,
                TimeoutStage::Knowledge => summary.knowledge_ms += timing.duration_ms,
                TimeoutStage::Global => {}
            }
            if timing.timed_out {
                summary.timeouts.push(timing.stage);
            }
        }
        summary.total_ms = self.global_elapsed_ms();
        summary
    }

    /// Snapshot current stage without ending it.
    fn end_current_stage_snapshot(&self) {
        // This is a read-only snapshot - actual stage continues
    }

    /// Create a Duration for remaining time (for async timeouts).
    pub fn remaining_duration(&self) -> Duration {
        Duration::from_millis(self.remaining_global_ms())
    }
}

impl Default for TimeoutEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of timing breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingSummary {
    /// Translator time.
    pub translator_ms: u64,
    /// Junior LLM time.
    pub junior_llm_ms: u64,
    /// Senior LLM time.
    pub senior_llm_ms: u64,
    /// Parse time.
    pub parse_ms: u64,
    /// Probes time.
    pub probes_ms: u64,
    /// Knowledge lookup time.
    pub knowledge_ms: u64,
    /// Total time.
    pub total_ms: u64,
    /// Stages that timed out.
    pub timeouts: Vec<TimeoutStage>,
}

impl TimingSummary {
    /// Format for debug display.
    pub fn format_debug(&self) -> String {
        let mut parts = Vec::new();
        if self.translator_ms > 0 {
            parts.push(format!("translator: {}ms", self.translator_ms));
        }
        if self.junior_llm_ms > 0 {
            parts.push(format!("junior_llm: {}ms", self.junior_llm_ms));
        }
        if self.senior_llm_ms > 0 {
            parts.push(format!("senior_llm: {}ms", self.senior_llm_ms));
        }
        if self.parse_ms > 0 {
            parts.push(format!("parse: {}ms", self.parse_ms));
        }
        if self.probes_ms > 0 {
            parts.push(format!("probes: {}ms", self.probes_ms));
        }
        parts.push(format!("total: {}ms", self.total_ms));

        if !self.timeouts.is_empty() {
            let timeout_names: Vec<_> = self.timeouts.iter().map(|t| t.name()).collect();
            parts.push(format!("TIMEOUTS: {}", timeout_names.join(", ")));
        }

        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_budget() {
        let budget = TimeBudget::new(100, 200);

        assert!(!budget.is_over_soft(50));
        assert!(budget.is_over_soft(150));
        assert!(!budget.is_over_hard(150));
        assert!(budget.is_over_hard(250));
        assert_eq!(budget.remaining_ms(100), 100);
    }

    #[test]
    fn test_timeout_enforcer() {
        let mut enforcer = TimeoutEnforcer::new();

        enforcer.start_stage(TimeoutStage::Translator);
        assert!(!enforcer.is_global_timeout());

        // After a very short time, should not be over
        assert!(!enforcer.is_current_over_hard());
    }

    #[test]
    fn test_retry_tracking() {
        let mut enforcer = TimeoutEnforcer::new();

        assert!(enforcer.can_retry_parse());
        enforcer.record_retry();
        assert!(!enforcer.can_retry_parse()); // Only one retry allowed
    }
}
