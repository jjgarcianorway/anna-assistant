//! Time budgets for entire pipeline.

use std::time::Instant;

use super::constants::*;
use super::phase::{Phase, PhaseBudget};
use super::summary::BudgetSummary;

/// Result of a budget check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetResult {
    /// Within budget, proceed.
    Proceed,
    /// Exceeded budget, should cancel.
    Exceeded,
    /// Total budget exhausted.
    TotalExhausted,
}

/// Time budgets for entire pipeline.
#[derive(Debug, Clone)]
pub struct TimeBudgets {
    /// Total budget for the query.
    pub total_budget_ms: u64,
    /// Per-phase budgets.
    pub phases: Vec<PhaseBudget>,
    /// When pipeline started.
    pub started_at: Option<Instant>,
    /// Total elapsed time.
    pub elapsed_ms: u64,
}

impl TimeBudgets {
    /// Create standard budgets.
    pub fn standard() -> Self {
        Self {
            total_budget_ms: TOTAL_BUDGET_MS,
            phases: vec![
                PhaseBudget::new(Phase::TranslatorIntent),
                PhaseBudget::new(Phase::ProbeCollection),
                PhaseBudget::new(Phase::JuniorSpecialist),
                PhaseBudget::new(Phase::SeniorSpecialist),
                PhaseBudget::new(Phase::Renderer),
            ],
            started_at: None,
            elapsed_ms: 0,
        }
    }

    /// Create with custom total budget.
    pub fn with_total(total_ms: u64) -> Self {
        let mut budgets = Self::standard();
        budgets.total_budget_ms = total_ms;
        budgets
    }

    /// Start the timer.
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }

    /// Update elapsed time.
    pub fn update_elapsed(&mut self) {
        if let Some(start) = self.started_at {
            self.elapsed_ms = start.elapsed().as_millis() as u64;
        }
    }

    /// Get phase budget.
    pub fn get_phase(&self, phase: Phase) -> Option<&PhaseBudget> {
        self.phases.iter().find(|p| p.phase == phase)
    }

    /// Get mutable phase budget.
    pub fn get_phase_mut(&mut self, phase: Phase) -> Option<&mut PhaseBudget> {
        self.phases.iter_mut().find(|p| p.phase == phase)
    }

    /// Check if total budget is exhausted.
    pub fn total_exhausted(&self) -> bool {
        self.elapsed_ms >= self.total_budget_ms
    }

    /// Get remaining total budget.
    pub fn remaining_total(&self) -> u64 {
        if self.elapsed_ms >= self.total_budget_ms {
            0
        } else {
            self.total_budget_ms - self.elapsed_ms
        }
    }

    /// Check budget for a phase.
    pub fn check_phase(&mut self, phase: Phase) -> BudgetResult {
        self.update_elapsed();

        if self.total_exhausted() {
            return BudgetResult::TotalExhausted;
        }

        let phase_budget = phase.budget_ms();
        let remaining = self.remaining_total();

        if remaining < phase_budget / 2 {
            // Not enough time for this phase
            BudgetResult::TotalExhausted
        } else {
            BudgetResult::Proceed
        }
    }

    /// Complete a phase.
    pub fn complete_phase(&mut self, phase: Phase, actual_ms: u64) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.complete(actual_ms);
        }
        self.update_elapsed();
    }

    /// Timeout a phase.
    pub fn timeout_phase(&mut self, phase: Phase, actual_ms: u64) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.timeout(actual_ms);
        }
        self.update_elapsed();
    }

    /// Skip a phase.
    pub fn skip_phase(&mut self, phase: Phase) {
        if let Some(p) = self.get_phase_mut(phase) {
            p.skip();
        }
    }

    /// Get summary of phase timings.
    pub fn summary(&self) -> BudgetSummary {
        let mut phases = Vec::new();
        let mut timed_out_count = 0;
        let mut skipped_count = 0;

        for p in &self.phases {
            if p.timed_out {
                timed_out_count += 1;
            }
            if p.skipped {
                skipped_count += 1;
            }
            if let Some(actual) = p.actual_ms {
                phases.push((p.phase, actual, p.timed_out));
            }
        }

        BudgetSummary {
            total_ms: self.elapsed_ms,
            phases,
            timed_out_phases: timed_out_count,
            skipped_phases: skipped_count,
            within_budget: self.elapsed_ms <= self.total_budget_ms,
        }
    }
}

impl Default for TimeBudgets {
    fn default() -> Self {
        Self::standard()
    }
}
