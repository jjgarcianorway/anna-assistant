//! Budget summary and display formatting.

use super::phase::Phase;

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
