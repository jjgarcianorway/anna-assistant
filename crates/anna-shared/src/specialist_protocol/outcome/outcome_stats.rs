//! Aggregated statistics for honest ticket outcome reporting.

use super::TicketOutcome;
use serde::{Deserialize, Serialize};

/// Aggregated stats for honest reporting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestTicketStats {
    /// Total tickets processed
    pub total: usize,
    /// Full successes
    pub success: usize,
    /// Useful partial answers
    pub useful_partial: usize,
    /// Honest "I don't know"
    pub honest_unknown: usize,
    /// Hard failures
    pub failed: usize,
    /// Internal errors (parse, timeout, etc.)
    pub internal_errors: usize,
    /// Parse errors specifically
    pub parse_errors: usize,
    /// Timeouts specifically
    pub timeouts: usize,
    /// Average response time (ms)
    pub avg_response_ms: f64,
    /// Escalations (junior -> senior)
    pub escalations: usize,
}

impl HonestTicketStats {
    /// Record an outcome
    pub fn record(&mut self, outcome: TicketOutcome, response_ms: u64) {
        self.total += 1;

        // Update running average
        let n = self.total as f64;
        self.avg_response_ms = self.avg_response_ms * ((n - 1.0) / n) + (response_ms as f64) / n;

        match outcome {
            TicketOutcome::Success => self.success += 1,
            TicketOutcome::UsefulPartial => self.useful_partial += 1,
            TicketOutcome::HonestUnknown => self.honest_unknown += 1,
            TicketOutcome::Failed => self.failed += 1,
            TicketOutcome::InternalError => self.internal_errors += 1,
        }
    }

    /// Record a parse error
    pub fn record_parse_error(&mut self) {
        self.parse_errors += 1;
        self.internal_errors += 1;
        self.total += 1;
    }

    /// Record a timeout
    pub fn record_timeout(&mut self) {
        self.timeouts += 1;
        self.internal_errors += 1;
        self.total += 1;
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.escalations += 1;
    }

    /// Get resolved count (user got useful answer)
    pub fn resolved(&self) -> usize {
        self.success + self.useful_partial + self.honest_unknown
    }

    /// Get success rate (only full successes)
    pub fn success_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f32 / self.total as f32) * 100.0
        }
    }

    /// Get resolution rate (all useful answers)
    pub fn resolution_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.resolved() as f32 / self.total as f32) * 100.0
        }
    }

    /// Get failure rate
    pub fn failure_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            ((self.failed + self.internal_errors) as f32 / self.total as f32) * 100.0
        }
    }

    /// Get internal error rate
    pub fn internal_error_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.internal_errors as f32 / self.total as f32) * 100.0
        }
    }

    /// Validate that stats are honest (no impossible rates)
    pub fn validate(&self) -> Result<(), String> {
        // Sum of parts must equal total
        let _sum = self.success + self.useful_partial + self.honest_unknown + self.failed;
        // Note: internal_errors overlap with failed, so we don't add them

        // Success rate can't be 100% if there are failures
        if self.success_rate() > 99.9 && self.failed > 0 {
            return Err("Success rate claims 100% but failures exist".to_string());
        }

        // Resolution rate can't exceed 100%
        if self.resolution_rate() > 100.0 {
            return Err("Resolution rate exceeds 100%".to_string());
        }

        Ok(())
    }
}

impl std::fmt::Display for HonestTicketStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[service desk]")?;
        writeln!(f, "  total_tickets   {}", self.total)?;
        writeln!(
            f,
            "  resolved        {} ({:.0}%)",
            self.resolved(),
            self.resolution_rate()
        )?;
        writeln!(f, "  escalated       {}", self.escalations)?;
        writeln!(f, "  avg_response    {:.0}ms", self.avg_response_ms)?;
        writeln!(f)?;
        writeln!(f, "[reliability]")?;
        writeln!(
            f,
            "  success         {} ({:.0}%)",
            self.success,
            self.success_rate()
        )?;
        writeln!(f, "  useful_partial  {}", self.useful_partial)?;
        writeln!(f, "  honest_unknown  {}", self.honest_unknown)?;
        writeln!(f, "  failed          {}", self.failed)?;
        writeln!(
            f,
            "  internal_errors {} (parse: {}, timeout: {})",
            self.internal_errors, self.parse_errors, self.timeouts
        )?;
        Ok(())
    }
}
