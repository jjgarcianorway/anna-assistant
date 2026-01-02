//! Stats engine implementation (v0.0.433).
//!
//! Engine for tracking ticket outcomes and calculating XP.

use super::super::contract::TicketOutcome;
use super::types::{TicketStats, TruthfulStats};

/// Stats engine for tracking outcomes.
pub struct StatsEngine {
    stats: TruthfulStats,
    /// Base XP per successful ticket.
    base_xp: u64,
}

impl StatsEngine {
    /// Create a new stats engine.
    pub fn new() -> Self {
        Self {
            stats: TruthfulStats::default(),
            base_xp: 10,
        }
    }

    /// Create with initial stats.
    pub fn with_stats(stats: TruthfulStats) -> Self {
        Self { stats, base_xp: 10 }
    }

    /// Record a ticket completion.
    pub fn record_ticket(&mut self, ticket_stats: TicketStats) {
        let xp = self.calculate_xp(&ticket_stats);
        self.stats.record(&ticket_stats, xp);
    }

    /// Calculate XP for a ticket.
    fn calculate_xp(&self, stats: &TicketStats) -> u64 {
        if !stats.outcome.is_success() && !matches!(stats.outcome, TicketOutcome::Partial) {
            return 0;
        }

        let mut xp = self.base_xp;

        // Bonus for probes used (complexity)
        xp += (stats.probes_count as u64) * 2;

        // Bonus for fast resolution
        if stats.processing_time_ms < 2000 {
            xp += 5;
        }

        // Penalty for needing retry
        if stats.retry_used {
            xp = xp.saturating_sub(3);
        }

        xp
    }

    /// Get current stats.
    pub fn stats(&self) -> &TruthfulStats {
        &self.stats
    }

    /// Get mutable stats.
    pub fn stats_mut(&mut self) -> &mut TruthfulStats {
        &mut self.stats
    }

    /// Format stats for display.
    pub fn format_display(&self) -> String {
        let s = &self.stats;
        let mut lines = Vec::new();

        lines.push("=== Service Desk Statistics ===".to_string());
        lines.push(String::new());
        lines.push(s.format_summary());
        lines.push(String::new());
        lines.push(format!(
            "Resolved: {} (success: {}, partial: {})",
            s.total_resolved(),
            s.successes,
            s.partial_resolutions
        ));
        lines.push(format!(
            "Failures: {} (timeouts: {}, parse errors: {}, internal: {})",
            s.failures, s.timeouts, s.parse_errors, s.internal_errors
        ));
        lines.push(format!(
            "Awaiting clarification: {}",
            s.awaiting_clarification
        ));
        lines.push(format!("Total XP: {}", s.total_xp));

        // Department breakdown
        if !s.by_department.is_empty() {
            lines.push(String::new());
            lines.push("By Department:".to_string());
            for (name, dept) in &s.by_department {
                lines.push(format!(
                    "  {}: {:.0}% success ({} resolved, {} failed)",
                    name,
                    dept.success_rate() * 100.0,
                    dept.successes + dept.partial_resolutions,
                    dept.failures
                ));
            }
        }

        // Recent failures
        if !s.recent_failures.is_empty() {
            lines.push(String::new());
            lines.push(format!(
                "Recent failures (last 24h): {}",
                s.recent_failures.len()
            ));
        }

        lines.join("\n")
    }
}

impl Default for StatsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_rate_calculation() {
        let mut stats = TruthfulStats::default();

        // 8 successes, 2 failures = 80% success rate
        for _ in 0..8 {
            stats.record(&TicketStats::new("t", TicketOutcome::Success), 10);
        }
        for _ in 0..2 {
            stats.record(&TicketStats::new("t", TicketOutcome::Timeout), 0);
        }

        assert!((stats.success_rate() - 0.8).abs() < 0.01);
        assert_eq!(stats.failures, 2);
        assert_eq!(stats.successes, 8);
    }

    #[test]
    fn test_no_xp_for_failures() {
        let mut engine = StatsEngine::new();

        engine.record_ticket(TicketStats::new("t1", TicketOutcome::Timeout));
        engine.record_ticket(TicketStats::new("t2", TicketOutcome::ParseError));

        assert_eq!(engine.stats().total_xp, 0);
        assert_eq!(engine.stats().failures, 2);
    }

    #[test]
    fn test_xp_for_success() {
        let mut engine = StatsEngine::new();

        engine.record_ticket(TicketStats::new("t1", TicketOutcome::Success));

        assert!(engine.stats().total_xp > 0);
        assert_eq!(engine.stats().successes, 1);
    }

    #[test]
    fn test_clarification_not_counted() {
        let mut stats = TruthfulStats::default();

        stats.record(
            &TicketStats::new("t", TicketOutcome::ClarificationRequired),
            0,
        );

        // Should not count as success or failure
        assert_eq!(stats.successes, 0);
        assert_eq!(stats.failures, 0);
        assert_eq!(stats.awaiting_clarification, 1);
    }

    #[test]
    fn test_format_summary() {
        let mut stats = TruthfulStats::default();

        for _ in 0..7 {
            stats.record(&TicketStats::new("t", TicketOutcome::Success), 10);
        }
        for _ in 0..3 {
            stats.record(&TicketStats::new("t", TicketOutcome::Timeout), 0);
        }

        let summary = stats.format_summary();
        assert!(summary.contains("70%"));
        assert!(summary.contains("3 tickets failed"));
    }
}
