//! METER: Stage-level latency budgets with explicit degradation.
//!
//! Provides configurable budgets per stage and budget enforcement.
//! Pure decision functions for testability.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_budget_defaults() {
        let budget = StageBudget::default();
        assert_eq!(budget.translator_ms, 5_000);
        assert_eq!(budget.probes_ms, 12_000);
        assert_eq!(budget.specialist_ms, 15_000);
        assert_eq!(budget.supervisor_ms, 8_000);
        assert_eq!(budget.total_ms, 25_000);
        assert_eq!(budget.margin_ms, 1_000);
        assert_eq!(budget.effective_total(), 24_000);
    }

    #[test]
    fn test_check_stage_budget_ok() {
        let budget = StageBudget::default();
        let result = check_stage_budget(Stage::Translator, 4_000, &budget);
        assert_eq!(result, BudgetCheck::Ok);
    }

    #[test]
    fn test_check_stage_budget_exceeded() {
        let budget = StageBudget::default();
        let result = check_stage_budget(Stage::Translator, 6_000, &budget);
        assert!(matches!(
            result,
            BudgetCheck::StageExceeded {
                stage: Stage::Translator,
                budget_ms: 5_000,
                elapsed_ms: 6_000,
            }
        ));
    }

    #[test]
    fn test_check_total_budget_ok() {
        let budget = StageBudget::default();
        let result = check_total_budget(20_000, &budget);
        assert_eq!(result, BudgetCheck::Ok);
    }

    #[test]
    fn test_check_total_budget_exceeded() {
        let budget = StageBudget::default();
        // Effective total is 24_000 (25_000 - 1_000 margin)
        let result = check_total_budget(25_000, &budget);
        assert!(matches!(
            result,
            BudgetCheck::TotalExceeded {
                budget_ms: 24_000,
                elapsed_ms: 25_000,
            }
        ));
    }

    #[test]
    fn test_stage_timing() {
        let budget = StageBudget::default();
        let timing = StageTiming::new(Stage::Probes, 15_000, &budget);
        assert!(timing.exceeded);
        assert_eq!(timing.budget_ms, 12_000);
    }

    #[test]
    fn test_budget_check_is_exceeded() {
        assert!(!BudgetCheck::Ok.is_exceeded());
        assert!(BudgetCheck::StageExceeded {
            stage: Stage::Translator,
            budget_ms: 5_000,
            elapsed_ms: 6_000,
        }.is_exceeded());
        assert!(BudgetCheck::TotalExceeded {
            budget_ms: 24_000,
            elapsed_ms: 25_000,
        }.is_exceeded());
    }
}
