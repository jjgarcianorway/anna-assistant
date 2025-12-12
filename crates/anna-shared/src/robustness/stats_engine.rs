//! Truthful stats engine (v0.0.433).
//!
//! Stats that reflect actual outcomes, not any response.

use super::contract::TicketOutcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stats for a single ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketStats {
    /// Ticket ID.
    pub ticket_id: String,
    /// Final outcome.
    pub outcome: TicketOutcome,
    /// Processing time in ms.
    pub processing_time_ms: u64,
    /// Timestamp (unix ms).
    pub timestamp_ms: u64,
    /// Handler name.
    pub handler: Option<String>,
    /// Department.
    pub department: Option<String>,
    /// Number of probes ran.
    pub probes_count: usize,
    /// Whether retry was used.
    pub retry_used: bool,
}

impl TicketStats {
    /// Create new stats entry.
    pub fn new(ticket_id: &str, outcome: TicketOutcome) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            outcome,
            processing_time_ms: 0,
            timestamp_ms: now_ms(),
            handler: None,
            department: None,
            probes_count: 0,
            retry_used: false,
        }
    }
}

/// Record of a failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    /// Ticket ID.
    pub ticket_id: String,
    /// Type of failure.
    pub outcome: TicketOutcome,
    /// Error info if available.
    pub error_info: Option<String>,
    /// Timestamp.
    pub timestamp_ms: u64,
    /// Stage where failure occurred.
    pub failed_stage: Option<String>,
}

/// Stats per staff member.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaffStats {
    /// Staff name.
    pub name: String,
    /// Successful tickets.
    pub tickets_success: u64,
    /// Failed tickets.
    pub tickets_failed: u64,
    /// Partial resolutions.
    pub tickets_partial: u64,
    /// Total processing time.
    pub total_time_ms: u64,
    /// XP earned.
    pub xp_earned: u64,
}

impl StaffStats {
    /// Create new staff stats.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Success rate (0.0 to 1.0).
    pub fn success_rate(&self) -> f32 {
        let total = self.tickets_success + self.tickets_failed;
        if total == 0 {
            0.0
        } else {
            self.tickets_success as f32 / total as f32
        }
    }

    /// Total resolved (success + partial).
    pub fn total_resolved(&self) -> u64 {
        self.tickets_success + self.tickets_partial
    }

    /// Record a ticket outcome.
    pub fn record(&mut self, outcome: TicketOutcome, time_ms: u64, xp: u64) {
        self.total_time_ms += time_ms;

        match outcome {
            TicketOutcome::Success => {
                self.tickets_success += 1;
                self.xp_earned += xp;
            }
            TicketOutcome::Partial => {
                self.tickets_partial += 1;
                self.xp_earned += xp / 2; // Half XP for partial
            }
            TicketOutcome::Unsupported
            | TicketOutcome::InternalError
            | TicketOutcome::Timeout
            | TicketOutcome::ParseError => {
                self.tickets_failed += 1;
                // No XP for failures
            }
            TicketOutcome::ClarificationRequired => {
                // Not counted yet - final outcome pending
            }
        }
    }
}

/// Stats per department.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepartmentStats {
    /// Department name.
    pub name: String,
    /// Total successes.
    pub successes: u64,
    /// Total failures.
    pub failures: u64,
    /// Partial resolutions.
    pub partial_resolutions: u64,
    /// Timeouts specifically.
    pub timeouts: u64,
    /// Parse errors specifically.
    pub parse_errors: u64,
    /// Staff stats within this department.
    pub staff: HashMap<String, StaffStats>,
}

impl DepartmentStats {
    /// Create new department stats.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Overall success rate.
    pub fn success_rate(&self) -> f32 {
        let total = self.successes + self.failures;
        if total == 0 {
            0.0
        } else {
            self.successes as f32 / total as f32
        }
    }

    /// Record a ticket outcome.
    pub fn record(&mut self, outcome: TicketOutcome, handler: Option<&str>, time_ms: u64, xp: u64) {
        match outcome {
            TicketOutcome::Success => self.successes += 1,
            TicketOutcome::Partial => self.partial_resolutions += 1,
            TicketOutcome::Timeout => {
                self.failures += 1;
                self.timeouts += 1;
            }
            TicketOutcome::ParseError => {
                self.failures += 1;
                self.parse_errors += 1;
            }
            TicketOutcome::Unsupported | TicketOutcome::InternalError => {
                self.failures += 1;
            }
            TicketOutcome::ClarificationRequired => {
                // Not counted yet
            }
        }

        // Update staff stats
        if let Some(name) = handler {
            let staff = self
                .staff
                .entry(name.to_string())
                .or_insert_with(|| StaffStats::new(name));
            staff.record(outcome, time_ms, xp);
        }
    }
}

/// Aggregated truthful stats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TruthfulStats {
    /// Total successful resolutions.
    pub successes: u64,
    /// Total failures.
    pub failures: u64,
    /// Partial resolutions.
    pub partial_resolutions: u64,
    /// Tickets awaiting clarification.
    pub awaiting_clarification: u64,
    /// Total tickets processed.
    pub total_processed: u64,
    /// Timeouts count.
    pub timeouts: u64,
    /// Parse errors count.
    pub parse_errors: u64,
    /// Internal errors count.
    pub internal_errors: u64,
    /// Unsupported requests count.
    pub unsupported: u64,
    /// Total XP awarded.
    pub total_xp: u64,
    /// Stats by department.
    pub by_department: HashMap<String, DepartmentStats>,
    /// Recent failures (last 24h).
    pub recent_failures: Vec<FailureRecord>,
    /// Last update timestamp.
    pub last_update_ms: u64,
}

impl TruthfulStats {
    /// Overall success rate.
    pub fn success_rate(&self) -> f32 {
        let decided = self.successes + self.failures;
        if decided == 0 {
            0.0
        } else {
            self.successes as f32 / decided as f32
        }
    }

    /// Total resolved (success + partial).
    pub fn total_resolved(&self) -> u64 {
        self.successes + self.partial_resolutions
    }

    /// Format summary string.
    pub fn format_summary(&self) -> String {
        let rate = self.success_rate() * 100.0;
        if self.failures > 0 {
            format!(
                "Overall success rate: {:.0}%, {} tickets failed",
                rate, self.failures
            )
        } else {
            format!("Overall success rate: {:.0}%", rate)
        }
    }

    /// Record a ticket outcome.
    pub fn record(&mut self, stats: &TicketStats, xp: u64) {
        self.total_processed += 1;
        self.last_update_ms = now_ms();

        match stats.outcome {
            TicketOutcome::Success => {
                self.successes += 1;
                self.total_xp += xp;
            }
            TicketOutcome::Partial => {
                self.partial_resolutions += 1;
                self.total_xp += xp / 2;
            }
            TicketOutcome::ClarificationRequired => {
                self.awaiting_clarification += 1;
            }
            TicketOutcome::Unsupported => {
                self.failures += 1;
                self.unsupported += 1;
                self.add_failure_record(stats);
            }
            TicketOutcome::InternalError => {
                self.failures += 1;
                self.internal_errors += 1;
                self.add_failure_record(stats);
            }
            TicketOutcome::Timeout => {
                self.failures += 1;
                self.timeouts += 1;
                self.add_failure_record(stats);
            }
            TicketOutcome::ParseError => {
                self.failures += 1;
                self.parse_errors += 1;
                self.add_failure_record(stats);
            }
        }

        // Update department stats
        if let Some(dept) = &stats.department {
            let dept_stats = self
                .by_department
                .entry(dept.clone())
                .or_insert_with(|| DepartmentStats::new(dept));
            dept_stats.record(
                stats.outcome,
                stats.handler.as_deref(),
                stats.processing_time_ms,
                xp,
            );
        }
    }

    /// Add a failure record.
    fn add_failure_record(&mut self, stats: &TicketStats) {
        // Keep only last 100 failures
        if self.recent_failures.len() >= 100 {
            self.recent_failures.remove(0);
        }

        self.recent_failures.push(FailureRecord {
            ticket_id: stats.ticket_id.clone(),
            outcome: stats.outcome,
            error_info: None,
            timestamp_ms: stats.timestamp_ms,
            failed_stage: None,
        });
    }

    /// Prune old failure records (older than 24h).
    pub fn prune_old_failures(&mut self) {
        let cutoff = now_ms().saturating_sub(24 * 60 * 60 * 1000);
        self.recent_failures.retain(|f| f.timestamp_ms > cutoff);
    }
}

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
        lines.push(format!("Resolved: {} (success: {}, partial: {})",
            s.total_resolved(), s.successes, s.partial_resolutions));
        lines.push(format!("Failures: {} (timeouts: {}, parse errors: {}, internal: {})",
            s.failures, s.timeouts, s.parse_errors, s.internal_errors));
        lines.push(format!("Awaiting clarification: {}", s.awaiting_clarification));
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
            lines.push(format!("Recent failures (last 24h): {}", s.recent_failures.len()));
        }

        lines.join("\n")
    }
}

impl Default for StatsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Current time in milliseconds.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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

        stats.record(&TicketStats::new("t", TicketOutcome::ClarificationRequired), 0);

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
