//! Honest metrics display (v0.0.426).
//!
//! User-facing stats that never lie about success rates.
//! No more "100% success" when there are failures.

use crate::ticket_lifecycle::{
    compute_specialist_metrics, format_specialist_roster, ReliabilityMetrics, SpecialistMetrics,
    TicketRecord,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete honest stats output for annactl
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestStats {
    /// Service desk metrics
    pub service_desk: ServiceDeskStats,
    /// Reliability breakdown
    pub reliability: ReliabilityBreakdown,
    /// Per-specialist metrics
    pub specialists: HashMap<String, SpecialistMetrics>,
}

/// Service desk level statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceDeskStats {
    pub total_tickets: usize,
    pub resolved_success: usize,
    pub resolved_partial: usize,
    pub honest_unknown: usize,
    pub failed: usize,
    pub escalated: usize,
    pub avg_response_sec: f64,
}

/// Reliability breakdown for debugging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityBreakdown {
    pub success_rate: f32,
    pub reliability_rate: f32,
    pub parse_errors: usize,
    pub timeout_errors: usize,
    pub internal_errors: usize,
}

impl HonestStats {
    /// Compute from ticket records
    pub fn compute(tickets: &[TicketRecord]) -> Self {
        let metrics = ReliabilityMetrics::compute(tickets);
        let specialist_metrics = compute_specialist_metrics(tickets);

        Self {
            service_desk: ServiceDeskStats {
                total_tickets: metrics.total_tickets,
                resolved_success: metrics.resolved_success,
                resolved_partial: metrics.resolved_partial,
                honest_unknown: metrics.honest_unknown,
                failed: metrics.failed,
                escalated: metrics.escalated,
                avg_response_sec: metrics.avg_response_ms as f64 / 1000.0,
            },
            reliability: ReliabilityBreakdown {
                success_rate: metrics.success_rate,
                reliability_rate: metrics.reliability_rate,
                parse_errors: metrics.parse_errors,
                timeout_errors: metrics.timeout_errors,
                internal_errors: metrics.internal_errors,
            },
            specialists: specialist_metrics,
        }
    }

    /// Format for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Service desk section
        output.push_str("[service desk]\n");
        output.push_str(&format!(
            "  total_tickets         {}\n",
            self.service_desk.total_tickets
        ));
        output.push_str(&format!(
            "  resolved_success      {}\n",
            self.service_desk.resolved_success
        ));
        output.push_str(&format!(
            "  resolved_partial      {}\n",
            self.service_desk.resolved_partial
        ));
        output.push_str(&format!(
            "  honest_unknown        {}\n",
            self.service_desk.honest_unknown
        ));
        output.push_str(&format!(
            "  failed                {}\n",
            self.service_desk.failed
        ));
        output.push_str(&format!(
            "  escalated             {}\n",
            self.service_desk.escalated
        ));
        output.push_str(&format!(
            "  avg_response          {:.1}s\n",
            self.service_desk.avg_response_sec
        ));
        output.push('\n');

        // Reliability section
        output.push_str("[reliability]\n");
        output.push_str(&format!(
            "  success_rate          {:.0}%\n",
            self.reliability.success_rate
        ));
        output.push_str(&format!(
            "  reliability_rate      {:.0}%   ; success / (success + failed + internal errors)\n",
            self.reliability.reliability_rate
        ));
        output.push_str(&format!(
            "  parse_errors          {}\n",
            self.reliability.parse_errors
        ));
        output.push_str(&format!(
            "  timeout_errors        {}\n",
            self.reliability.timeout_errors
        ));
        output.push_str(&format!(
            "  internal_errors       {}\n",
            self.reliability.internal_errors
        ));
        output.push('\n');

        // Staff roster section
        output.push_str(&format_specialist_roster(&self.specialists));

        output
    }

    /// Validate stats are honest (for testing)
    pub fn validate(&self) -> Vec<String> {
        let mut issues = vec![];

        // Can't claim 100% success if there are any failures
        if self.service_desk.failed > 0 && self.reliability.success_rate >= 100.0 {
            issues.push(format!(
                "Invalid: {} failures but success_rate is {:.0}%",
                self.service_desk.failed, self.reliability.success_rate
            ));
        }

        // Can't claim 100% reliability if there are parse/internal errors
        let total_errors = self.reliability.parse_errors + self.reliability.internal_errors;
        if total_errors > 0 && self.reliability.reliability_rate >= 100.0 {
            issues.push(format!(
                "Invalid: {} errors but reliability_rate is {:.0}%",
                total_errors, self.reliability.reliability_rate
            ));
        }

        // Total must add up
        let sum = self.service_desk.resolved_success
            + self.service_desk.resolved_partial
            + self.service_desk.honest_unknown
            + self.service_desk.failed;

        // Allow for cancelled/pending tickets
        if sum > self.service_desk.total_tickets {
            issues.push(format!(
                "Invalid: sum of outcomes ({}) > total_tickets ({})",
                sum, self.service_desk.total_tickets
            ));
        }

        issues
    }
}

/// User-friendly error message for internal failures
pub fn format_internal_error(ticket_id: &str, debug_mode: bool) -> InternalErrorDisplay {
    InternalErrorDisplay {
        ticket_id: ticket_id.to_string(),
        user_message: "Anna hit an internal error while processing this ticket. This is a bug in Anna, not your fault.".to_string(),
        debug_mode,
        debug_info: None,
    }
}

/// Internal error display for user
#[derive(Debug, Clone)]
pub struct InternalErrorDisplay {
    pub ticket_id: String,
    pub user_message: String,
    pub debug_mode: bool,
    pub debug_info: Option<DebugInfo>,
}

/// Debug information (only shown in debug mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub ticket_id: String,
    pub error_kind: String,
    pub error_message: String,
    pub attempts: u8,
}

impl InternalErrorDisplay {
    /// Add debug info
    pub fn with_debug(mut self, kind: &str, message: &str, attempts: u8) -> Self {
        self.debug_info = Some(DebugInfo {
            ticket_id: self.ticket_id.clone(),
            error_kind: kind.to_string(),
            error_message: message.to_string(),
            attempts,
        });
        self
    }

    /// Format for display
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.user_message);
        output.push('\n');

        if self.debug_mode {
            if let Some(ref debug) = self.debug_info {
                output.push_str("\n[debug]\n");
                output.push_str(&format!("  ticket_id      {}\n", debug.ticket_id));
                output.push_str(&format!("  error_kind     {}\n", debug.error_kind));
                output.push_str(&format!("  error_message  {}\n", debug.error_message));
                output.push_str(&format!("  attempts       {}\n", debug.attempts));
            }
        }

        output
    }
}

/// Sanity check: ensure stats don't claim impossible success rates
pub fn sanity_check_stats(stats: &HonestStats) -> bool {
    let issues = stats.validate();
    issues.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v3::SpecialistResponse;
    use crate::ticket_lifecycle::{InternalError, TicketRecord};

    #[test]
    fn test_honest_stats_compute() {
        let tickets = create_test_tickets();
        let stats = HonestStats::compute(&tickets);

        assert_eq!(stats.service_desk.total_tickets, 5);
        assert_eq!(stats.service_desk.resolved_success, 2);
        assert_eq!(stats.service_desk.failed, 1);
        assert!(stats.reliability.success_rate < 100.0);
    }

    #[test]
    fn test_stats_validation() {
        // Create stats that would be invalid
        let mut stats = HonestStats::default();
        stats.service_desk.total_tickets = 10;
        stats.service_desk.failed = 5;
        stats.reliability.success_rate = 100.0; // Invalid!

        let issues = stats.validate();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_internal_error_display() {
        let display = format_internal_error("DSK-001", false);
        let output = display.format();
        assert!(output.contains("internal error"));
        assert!(!output.contains("[debug]"));

        let display_debug =
            format_internal_error("DSK-001", true).with_debug("parse_error", "Invalid JSON", 2);
        let output_debug = display_debug.format();
        assert!(output_debug.contains("[debug]"));
        assert!(output_debug.contains("parse_error"));
    }

    #[test]
    fn test_format_output() {
        let tickets = create_test_tickets();
        let stats = HonestStats::compute(&tickets);
        let output = stats.format();

        assert!(output.contains("[service desk]"));
        assert!(output.contains("[reliability]"));
        assert!(output.contains("total_tickets"));
        assert!(output.contains("success_rate"));
    }

    fn create_test_tickets() -> Vec<TicketRecord> {
        vec![
            create_success("T1"),
            create_success("T2"),
            create_partial("T3"),
            create_failed("T4"),
            create_honest_unknown("T5"),
        ]
    }

    fn create_success(id: &str) -> TicketRecord {
        let mut t = TicketRecord::new(id, "Test");
        t.start_processing("desktop.junior").unwrap();
        t.mark_answered(&SpecialistResponse::success(id, "Success"))
            .unwrap();
        t.mark_user_satisfied("Answer").unwrap();
        t
    }

    fn create_partial(id: &str) -> TicketRecord {
        let mut t = TicketRecord::new(id, "Test");
        t.start_processing("desktop.junior").unwrap();
        t.mark_answered(&SpecialistResponse::partial(id, "Partial"))
            .unwrap();
        t.mark_user_satisfied("Partial").unwrap();
        t
    }

    fn create_failed(id: &str) -> TicketRecord {
        let mut t = TicketRecord::new(id, "Test");
        t.start_processing("desktop.junior").unwrap();
        t.mark_failed(InternalError::ParseError {
            attempts: 2,
            last_error: "Bad JSON".to_string(),
        })
        .unwrap();
        t
    }

    fn create_honest_unknown(id: &str) -> TicketRecord {
        let mut t = TicketRecord::new(id, "Test");
        t.start_processing("desktop.junior").unwrap();
        t.mark_answered(&SpecialistResponse::no_data(id, "No data"))
            .unwrap();
        t.mark_user_satisfied("Unknown").unwrap();
        t
    }
}
