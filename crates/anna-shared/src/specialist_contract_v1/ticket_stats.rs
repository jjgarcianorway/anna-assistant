//! Ticket statistics tracking with integrity guarantees.

use super::TicketState;
use std::collections::HashMap;

/// Ticket stats with integrity guarantees.
#[derive(Debug, Clone, Default)]
pub struct TicketStats {
    /// Counts by state.
    counts: HashMap<TicketState, usize>,
    /// Total tickets processed.
    total: usize,
}

impl TicketStats {
    /// Create empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket in a state.
    pub fn record(&mut self, state: TicketState) {
        *self.counts.entry(state).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get count for a state.
    pub fn count(&self, state: TicketState) -> usize {
        self.counts.get(&state).copied().unwrap_or(0)
    }

    /// Get resolved count (RESOLVED state only).
    pub fn resolved(&self) -> usize {
        self.count(TicketState::Resolved)
    }

    /// Get failed count (FAILED_PROBE + FAILED_SPECIALIST).
    pub fn failed(&self) -> usize {
        self.count(TicketState::FailedProbe) + self.count(TicketState::FailedSpecialist)
    }

    /// Get total tickets.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Calculate success rate (resolved / total).
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.resolved() as f64 / self.total as f64
        }
    }

    /// Get summary for logging.
    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            total: self.total,
            resolved: self.resolved(),
            failed_probe: self.count(TicketState::FailedProbe),
            failed_specialist: self.count(TicketState::FailedSpecialist),
            need_clarification: self.count(TicketState::NeedClarification),
            escalated: self.count(TicketState::Escalated),
            success_rate: self.success_rate(),
        }
    }
}

/// Stats summary for display.
#[derive(Debug, Clone)]
pub struct StatsSummary {
    /// Total tickets.
    pub total: usize,
    /// Resolved count.
    pub resolved: usize,
    /// Failed probe count.
    pub failed_probe: usize,
    /// Failed specialist count.
    pub failed_specialist: usize,
    /// Need clarification count.
    pub need_clarification: usize,
    /// Escalated count.
    pub escalated: usize,
    /// Success rate (0.0-1.0).
    pub success_rate: f64,
}

impl StatsSummary {
    /// Format for logging.
    pub fn log_message(&self) -> String {
        format!(
            "[stats] total={} resolved={} failed_probe={} failed_specialist={} \
             need_clarification={} escalated={} success_rate={:.1}%",
            self.total,
            self.resolved,
            self.failed_probe,
            self.failed_specialist,
            self.need_clarification,
            self.escalated,
            self.success_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_tracking() {
        let mut stats = TicketStats::new();

        stats.record(TicketState::Resolved);
        stats.record(TicketState::Resolved);
        stats.record(TicketState::FailedProbe);
        stats.record(TicketState::FailedSpecialist);

        assert_eq!(stats.total(), 4);
        assert_eq!(stats.resolved(), 2);
        assert_eq!(stats.failed(), 2);
        assert!((stats.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stats_summary() {
        let mut stats = TicketStats::new();
        stats.record(TicketState::Resolved);
        stats.record(TicketState::FailedProbe);

        let summary = stats.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.resolved, 1);
        assert_eq!(summary.failed_probe, 1);
    }
}
