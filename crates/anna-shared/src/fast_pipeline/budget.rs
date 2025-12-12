//! Hard Time Budgets (Part A) - v0.0.438.
//!
//! Define explicit budgets per phase:
//! - translator_intent: 700ms
//! - probe_collection: 2500ms (can be parallel)
//! - junior_specialist: 1500ms
//! - senior_specialist: 3500ms (only on escalation)
//! - renderer: 200ms
//!
//! Overall max for one-shot queries: 6.5s
//!
//! If any phase exceeds budget: cancel it, mark timeout, continue with fallback.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Budget for translator intent extraction.
pub const TRANSLATOR_BUDGET_MS: u64 = 700;

/// Budget for probe collection (parallel probes).
pub const PROBE_BUDGET_MS: u64 = 2500;

/// Budget for junior specialist.
pub const JUNIOR_BUDGET_MS: u64 = 1500;

/// Budget for senior specialist (escalation only).
pub const SENIOR_BUDGET_MS: u64 = 3500;

/// Budget for renderer.
pub const RENDERER_BUDGET_MS: u64 = 200;

/// Total budget for one-shot queries.
pub const TOTAL_BUDGET_MS: u64 = 6500;

/// Pipeline phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Translator intent extraction.
    TranslatorIntent,
    /// Probe collection.
    ProbeCollection,
    /// Junior specialist analysis.
    JuniorSpecialist,
    /// Senior specialist analysis (escalation).
    SeniorSpecialist,
    /// Renderer (final output).
    Renderer,
}

impl Phase {
    /// Get budget for this phase in milliseconds.
    pub fn budget_ms(&self) -> u64 {
        match self {
            Self::TranslatorIntent => TRANSLATOR_BUDGET_MS,
            Self::JuniorSpecialist => JUNIOR_BUDGET_MS,
            Self::SeniorSpecialist => SENIOR_BUDGET_MS,
            Self::ProbeCollection => PROBE_BUDGET_MS,
            Self::Renderer => RENDERER_BUDGET_MS,
        }
    }

    /// Get budget as Duration.
    pub fn budget(&self) -> Duration {
        Duration::from_millis(self.budget_ms())
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TranslatorIntent => "intent extraction",
            Self::ProbeCollection => "probe collection",
            Self::JuniorSpecialist => "junior specialist",
            Self::SeniorSpecialist => "senior specialist",
            Self::Renderer => "rendering",
        }
    }
}

/// Budget for a single phase.
#[derive(Debug, Clone)]
pub struct PhaseBudget {
    /// Phase name.
    pub phase: Phase,
    /// Budget in milliseconds.
    pub budget_ms: u64,
    /// Actual time spent.
    pub actual_ms: Option<u64>,
    /// Whether this phase timed out.
    pub timed_out: bool,
    /// Whether this phase was skipped.
    pub skipped: bool,
}

impl PhaseBudget {
    /// Create a new phase budget.
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            budget_ms: phase.budget_ms(),
            actual_ms: None,
            timed_out: false,
            skipped: false,
        }
    }

    /// Mark as completed with actual time.
    pub fn complete(&mut self, actual_ms: u64) {
        self.actual_ms = Some(actual_ms);
        self.timed_out = actual_ms > self.budget_ms;
    }

    /// Mark as timed out.
    pub fn timeout(&mut self, actual_ms: u64) {
        self.actual_ms = Some(actual_ms);
        self.timed_out = true;
    }

    /// Mark as skipped.
    pub fn skip(&mut self) {
        self.skipped = true;
    }

    /// Check if within budget.
    pub fn within_budget(&self) -> bool {
        !self.timed_out
    }

    /// Get remaining budget.
    pub fn remaining_ms(&self, elapsed_ms: u64) -> u64 {
        if elapsed_ms >= self.budget_ms {
            0
        } else {
            self.budget_ms - elapsed_ms
        }
    }
}

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

/// Summary of budget usage.
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    /// Total time spent.
    pub total_ms: u64,
    /// Per-phase timings: (phase, actual_ms, timed_out).
    pub phases: Vec<(Phase, u64, bool)>,
    /// Count of timed out phases.
    pub timed_out_phases: usize,
    /// Count of skipped phases.
    pub skipped_phases: usize,
    /// Whether total was within budget.
    pub within_budget: bool,
}

impl BudgetSummary {
    /// Format as display string.
    pub fn display(&self) -> String {
        let status = if self.within_budget { "OK" } else { "OVER" };
        let phase_str: Vec<String> = self
            .phases
            .iter()
            .map(|(p, ms, timeout)| {
                let marker = if *timeout { "!" } else { "" };
                format!("{}{}:{}ms", p.label(), marker, ms)
            })
            .collect();

        format!(
            "[{}] {}ms total | {} | timeouts:{} skipped:{}",
            status,
            self.total_ms,
            phase_str.join(" | "),
            self.timed_out_phases,
            self.skipped_phases
        )
    }
}

/// Tracks budget for a running operation.
pub struct BudgetTracker {
    /// Budgets configuration.
    pub budgets: TimeBudgets,
    /// Current phase.
    pub current_phase: Option<Phase>,
    /// Phase start time.
    pub phase_started_at: Option<Instant>,
}

impl BudgetTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        let mut budgets = TimeBudgets::standard();
        budgets.start();

        Self {
            budgets,
            current_phase: None,
            phase_started_at: None,
        }
    }

    /// Start a phase.
    pub fn start_phase(&mut self, phase: Phase) -> BudgetResult {
        let result = self.budgets.check_phase(phase);

        if result == BudgetResult::Proceed {
            self.current_phase = Some(phase);
            self.phase_started_at = Some(Instant::now());
        }

        result
    }

    /// Complete current phase.
    pub fn complete_phase(&mut self) {
        if let (Some(phase), Some(start)) = (self.current_phase, self.phase_started_at) {
            let actual = start.elapsed().as_millis() as u64;
            self.budgets.complete_phase(phase, actual);
        }
        self.current_phase = None;
        self.phase_started_at = None;
    }

    /// Check if current phase has exceeded budget.
    pub fn phase_exceeded(&self) -> bool {
        if let (Some(phase), Some(start)) = (self.current_phase, self.phase_started_at) {
            let elapsed = start.elapsed().as_millis() as u64;
            elapsed > phase.budget_ms()
        } else {
            false
        }
    }

    /// Get remaining time for current phase.
    pub fn phase_remaining_ms(&self) -> u64 {
        if let (Some(phase), Some(start)) = (self.current_phase, self.phase_started_at) {
            let elapsed = start.elapsed().as_millis() as u64;
            let budget = phase.budget_ms();
            if elapsed >= budget {
                0
            } else {
                budget - elapsed
            }
        } else {
            0
        }
    }

    /// Cancel current phase as timeout.
    pub fn timeout_phase(&mut self) {
        if let (Some(phase), Some(start)) = (self.current_phase, self.phase_started_at) {
            let actual = start.elapsed().as_millis() as u64;
            self.budgets.timeout_phase(phase, actual);
        }
        self.current_phase = None;
        self.phase_started_at = None;
    }

    /// Skip a phase.
    pub fn skip_phase(&mut self, phase: Phase) {
        self.budgets.skip_phase(phase);
    }

    /// Get summary.
    pub fn summary(&self) -> BudgetSummary {
        self.budgets.summary()
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_budgets() {
        assert_eq!(Phase::TranslatorIntent.budget_ms(), 700);
        assert_eq!(Phase::JuniorSpecialist.budget_ms(), 1500);
        assert_eq!(Phase::SeniorSpecialist.budget_ms(), 3500);
    }

    #[test]
    fn test_time_budgets_standard() {
        let budgets = TimeBudgets::standard();
        assert_eq!(budgets.total_budget_ms, 6500);
        assert_eq!(budgets.phases.len(), 5);
    }

    #[test]
    fn test_budget_tracker() {
        let mut tracker = BudgetTracker::new();

        // Start a phase
        let result = tracker.start_phase(Phase::TranslatorIntent);
        assert_eq!(result, BudgetResult::Proceed);

        // Complete it
        tracker.complete_phase();

        let summary = tracker.summary();
        assert!(summary.phases.len() >= 1);
    }

    #[test]
    fn test_phase_budget_timeout() {
        let mut phase = PhaseBudget::new(Phase::JuniorSpecialist);
        phase.timeout(2000); // Exceeded 1500ms budget

        assert!(phase.timed_out);
        assert!(!phase.within_budget());
    }

    #[test]
    fn test_budget_summary_display() {
        let mut budgets = TimeBudgets::standard();
        budgets.complete_phase(Phase::TranslatorIntent, 500);
        budgets.timeout_phase(Phase::JuniorSpecialist, 2000);
        budgets.elapsed_ms = 2500;

        let summary = budgets.summary();
        let display = summary.display();

        assert!(display.contains("OK"));
        assert!(display.contains("timeouts:1"));
    }
}
