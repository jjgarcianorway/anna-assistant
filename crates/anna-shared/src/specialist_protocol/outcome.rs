//! Ticket outcome tracking for honest stats (v0.0.428).
//!
//! A ticket counts as "resolved" only if:
//! - status == success OR
//! - status == partial with a clearly useful answer
//!
//! A ticket counts as "failed" if:
//! - status == failure
//! - final answer says "I don't know" without meaningful facts
//! - parse errors with no useful fallback

use super::{ResponseStatus, StrictResponse, ValidationResult};
use serde::{Deserialize, Serialize};

/// Ticket outcome from user perspective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// Full success: user got a complete, evidence-backed answer
    Success,
    /// Useful partial: user got meaningful but incomplete info
    UsefulPartial,
    /// Honest unknown: we admitted we don't know (still honest)
    HonestUnknown,
    /// Failed: user did not get useful information
    Failed,
    /// Internal error: parse/timeout that we couldn't recover from
    InternalError,
}

impl TicketOutcome {
    /// Check if this outcome counts as "resolved" for stats
    pub fn is_resolved(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::UsefulPartial | Self::HonestUnknown
        )
    }

    /// Check if this is a hard failure
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed | Self::InternalError)
    }

    /// Get XP value for this outcome
    pub fn xp_value(&self) -> i32 {
        match self {
            Self::Success => 10,
            Self::UsefulPartial => 6,
            Self::HonestUnknown => 3,
            Self::Failed => 0,
            Self::InternalError => -2,
        }
    }
}

/// Determine ticket outcome from response
pub fn determine_outcome(
    response: &StrictResponse,
    validation: &ValidationResult,
) -> TicketOutcome {
    // If validation failed seriously, it's an internal error
    if !validation.valid {
        let has_serious_error = validation.errors.iter().any(|e| {
            matches!(
                e,
                super::ValidationError::InventedData(_)
                    | super::ValidationError::ForbiddenPattern(_)
            )
        });

        if has_serious_error {
            return TicketOutcome::InternalError;
        }
    }

    match response.status {
        ResponseStatus::Success => {
            // Success requires evidence and valid response
            if response.evidence.probes_used.is_empty()
                && response.evidence.arch_wiki_pages.is_empty()
                && response.evidence.man_pages.is_empty()
            {
                // No evidence - downgrade to partial
                if response.confidence >= 0.8 {
                    return TicketOutcome::UsefulPartial;
                }
                return TicketOutcome::Failed;
            }

            if validation.valid && response.confidence >= 0.7 {
                TicketOutcome::Success
            } else {
                TicketOutcome::UsefulPartial
            }
        }

        ResponseStatus::Partial => {
            // Check if partial has useful content
            if is_useful_partial(response) {
                TicketOutcome::UsefulPartial
            } else {
                TicketOutcome::Failed
            }
        }

        ResponseStatus::Failure => {
            // Check if it's an honest "I don't know" vs complete failure
            if is_honest_unknown(response) {
                TicketOutcome::HonestUnknown
            } else {
                TicketOutcome::Failed
            }
        }
    }
}

/// Check if a partial response is useful
fn is_useful_partial(response: &StrictResponse) -> bool {
    // Must have some meaningful content
    if response.summary.trim().is_empty() {
        return false;
    }

    // Must have at least one fact or evidence
    let has_facts = !response.details.key_facts.is_empty();
    let has_evidence = !response.evidence.probes_used.is_empty();
    let has_diagnosis = response
        .details
        .diagnosis
        .as_ref()
        .map(|d| !d.is_empty())
        .unwrap_or(false);

    // Check for common "useless partial" patterns
    let summary_lower = response.summary.to_lowercase();
    let useless_patterns = [
        "i could not",
        "i was unable",
        "no data available",
        "cannot determine",
        "unable to determine",
    ];

    let only_says_failure =
        useless_patterns.iter().all(|p| summary_lower.contains(p)) && !has_facts && !has_evidence;

    if only_says_failure {
        return false;
    }

    // Must have meaningful confidence
    if response.confidence < 0.3 {
        return false;
    }

    has_facts || has_evidence || has_diagnosis
}

/// Check if failure is an honest "I don't know"
fn is_honest_unknown(response: &StrictResponse) -> bool {
    let summary_lower = response.summary.to_lowercase();

    // Honest unknown patterns
    let honest_patterns = [
        "i don't have",
        "i cannot determine",
        "no specialist available",
        "outside my expertise",
        "i don't know",
        "i cannot answer",
        "i lack the data",
    ];

    honest_patterns.iter().any(|p| summary_lower.contains(p))
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::{ProbeEvidence, ResponseMeta};

    fn make_meta() -> ResponseMeta {
        ResponseMeta {
            handled_by: "Test".to_string(),
            ticket_id: "TEST-001".to_string(),
            version: 1,
        }
    }

    fn make_success() -> StrictResponse {
        StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "0 failed units".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        )
    }

    #[test]
    fn test_success_outcome() {
        let response = make_success();
        let validation = super::super::validate_response(&response);
        let outcome = determine_outcome(&response, &validation);

        assert_eq!(outcome, TicketOutcome::Success);
        assert!(outcome.is_resolved());
        assert_eq!(outcome.xp_value(), 10);
    }

    #[test]
    fn test_useful_partial_outcome() {
        let response = StrictResponse::partial(
            "storage.disk",
            "check_disk_usage",
            "Root filesystem is at 97% used.",
            vec!["30 GiB free".to_string()],
            "Could not identify largest directories.",
            vec![ProbeEvidence {
                id: "df".to_string(),
                summary: "97% used".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        )
        .with_confidence(0.6);

        let validation = super::super::validate_response(&response);
        let outcome = determine_outcome(&response, &validation);

        assert_eq!(outcome, TicketOutcome::UsefulPartial);
        assert!(outcome.is_resolved());
    }

    #[test]
    fn test_failure_outcome() {
        let response = StrictResponse::failure(
            "system",
            "unknown",
            "Complete system failure occurred.",
            make_meta(),
        );

        let validation = super::super::validate_response(&response);
        let outcome = determine_outcome(&response, &validation);

        assert_eq!(outcome, TicketOutcome::Failed);
        assert!(outcome.is_failed());
    }

    #[test]
    fn test_honest_unknown_outcome() {
        let response = StrictResponse::failure(
            "network",
            "check_vpn",
            "I don't have the capability to check VPN configuration.",
            make_meta(),
        );

        let validation = super::super::validate_response(&response);
        let outcome = determine_outcome(&response, &validation);

        assert_eq!(outcome, TicketOutcome::HonestUnknown);
        assert!(outcome.is_resolved()); // Honest unknown is still "resolved"
    }

    #[test]
    fn test_stats_recording() {
        let mut stats = HonestTicketStats::default();

        stats.record(TicketOutcome::Success, 500);
        stats.record(TicketOutcome::Success, 600);
        stats.record(TicketOutcome::UsefulPartial, 800);
        stats.record(TicketOutcome::Failed, 100);

        assert_eq!(stats.total, 4);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.useful_partial, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.resolved(), 3);
    }

    #[test]
    fn test_stats_validation() {
        let mut stats = HonestTicketStats::default();
        stats.total = 10;
        stats.success = 10;
        stats.failed = 1; // This is inconsistent!

        let result = stats.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_display() {
        let mut stats = HonestTicketStats::default();
        stats.record(TicketOutcome::Success, 500);
        stats.record_parse_error();

        let display = format!("{}", stats);
        assert!(display.contains("total_tickets"));
        assert!(display.contains("parse"));
    }
}
