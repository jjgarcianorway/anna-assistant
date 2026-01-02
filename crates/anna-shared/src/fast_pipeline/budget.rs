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

// Re-export all public types from sibling modules
pub use super::budget_tracker::{BudgetTracker, TimeBudgets};
pub use super::budget_types::{
    BudgetResult, BudgetSummary, Phase, PhaseBudget, JUNIOR_BUDGET_MS, PROBE_BUDGET_MS,
    RENDERER_BUDGET_MS, SENIOR_BUDGET_MS, TOTAL_BUDGET_MS, TRANSLATOR_BUDGET_MS,
};

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
