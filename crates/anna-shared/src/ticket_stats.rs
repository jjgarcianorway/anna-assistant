//! Truthful ticket statistics (v0.0.407).
//!
//! v0.0.411: Stats computed from TicketOutcome for honesty
//!
//! Extracted from ticket_log.rs for modularity.
//! Provides honest stats computation:
//! - Only count real successes (outcome = Success)
//! - Track partial answers and cannot_answer separately
//! - Track errors by type
//! - Calculate averages only from resolved tickets

use crate::ticket_log::TicketLog;
use crate::ticket_state::{TicketOutcome, TicketState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// v0.0.411: Outcome-based ticket statistics (truthful)
/// v0.0.464: Enhanced with repeated questions, topic tracking per Phase 30
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketStats {
    /// Total ticket count (all tickets created)
    pub total: usize,
    /// Success (outcome = Success)
    pub success: usize,
    /// Partial answer (outcome = Partial)
    pub partial: usize,
    /// Cannot answer safely (outcome = CannotAnswerSafely)
    pub cannot_answer: usize,
    /// Failed with errors (outcome = Error*)
    pub failed: usize,
    /// LLM failures specifically (parse + timeout + validation)
    pub llm_failed: usize,
    /// Tickets that reached answered state
    pub answered: usize,
    /// Escalated tickets count
    pub escalated: usize,
    /// Tickets by handler type
    pub by_handler: HashMap<String, usize>,
    /// Tickets by domain
    pub by_domain: HashMap<String, usize>,
    /// Tickets by error kind
    pub by_error: HashMap<String, usize>,
    /// v0.0.411: Tickets by outcome
    pub by_outcome: HashMap<String, usize>,
    /// Average duration (ms) - only for resolved tickets
    pub avg_duration_ms: u64,
    /// Average reliability score - only for resolved tickets
    pub avg_reliability: u8,
    /// Success rate (0-100) - full successes only
    pub success_rate: f32,
    /// v0.0.411: Resolution rate (0-100) - resolved (including partial/cannot_answer)
    pub resolution_rate: f32,
    /// v0.0.411: Error rate (0-100)
    pub error_rate: f32,
    /// v0.0.464: Repeated questions (query hash -> count)
    #[serde(default)]
    pub repeated_queries: HashMap<String, usize>,
    /// v0.0.464: Most asked topic (domain with most tickets)
    #[serde(default)]
    pub top_topic: Option<String>,
    /// v0.0.464: Tickets by intent
    #[serde(default)]
    pub by_intent: HashMap<String, usize>,
}

/// Calculate statistics from ticket logs (v0.0.411: Outcome-based)
///
/// This function computes stats based on TicketOutcome:
/// - `success`: outcome = Success
/// - `partial`: outcome = Partial
/// - `cannot_answer`: outcome = CannotAnswerSafely
/// - `failed`: outcome = Error* variants
/// - Rates computed from these real outcomes
pub fn calculate_stats(tickets: &[TicketLog]) -> TicketStats {
    let mut stats = TicketStats::default();

    if tickets.is_empty() {
        return stats;
    }

    let mut resolved_duration = 0u64;
    let mut resolved_count = 0usize;
    let mut reliability_sum = 0u64;
    let mut reliability_count = 0usize;

    for ticket in tickets {
        stats.total += 1;

        // v0.0.411: Derive outcome from ticket
        let outcome = derive_outcome(ticket);
        *stats.by_outcome.entry(outcome.to_string()).or_default() += 1;

        // Count by outcome type
        match outcome {
            TicketOutcome::Success => {
                stats.success += 1;
                stats.answered += 1;
            }
            TicketOutcome::Partial => {
                stats.partial += 1;
                stats.answered += 1;
            }
            TicketOutcome::CannotAnswerSafely => {
                stats.cannot_answer += 1;
                stats.answered += 1;
            }
            TicketOutcome::ErrorParse
            | TicketOutcome::ErrorTimeout
            | TicketOutcome::ErrorTool
            | TicketOutcome::ErrorInternal => {
                stats.failed += 1;
            }
        }

        // Track LLM-specific failures
        if ticket.is_llm_failure() {
            stats.llm_failed += 1;
        }

        // For resolved tickets (not errors), track duration and reliability
        if outcome.is_resolved() {
            resolved_duration += ticket.duration_ms;
            resolved_count += 1;

            if ticket.reliability_score > 0 {
                reliability_sum += ticket.reliability_score as u64;
                reliability_count += 1;
            }
        }

        if ticket.escalated {
            stats.escalated += 1;
        }

        // Track by handler
        *stats
            .by_handler
            .entry(categorize_handler(&ticket.handled_by))
            .or_default() += 1;
        *stats.by_domain.entry(ticket.domain.clone()).or_default() += 1;

        // v0.0.464: Track by intent
        *stats.by_intent.entry(ticket.intent.clone()).or_default() += 1;

        // v0.0.464: Track repeated queries (normalize to lowercase for matching)
        let query_key = normalize_query(&ticket.query);
        *stats.repeated_queries.entry(query_key).or_default() += 1;

        // Track by error kind
        if let Some(ref kind) = ticket.error_kind {
            *stats.by_error.entry(kind.to_string()).or_default() += 1;
        }
    }

    // Calculate averages only from resolved tickets
    if resolved_count > 0 {
        stats.avg_duration_ms = resolved_duration / resolved_count as u64;
    }
    if reliability_count > 0 {
        stats.avg_reliability = (reliability_sum / reliability_count as u64) as u8;
    }

    // v0.0.411: Calculate rates from outcomes
    if stats.total > 0 {
        let total = stats.total as f32;
        stats.success_rate = (stats.success as f32 / total) * 100.0;
        stats.resolution_rate =
            ((stats.success + stats.partial + stats.cannot_answer) as f32 / total) * 100.0;
        stats.error_rate = (stats.failed as f32 / total) * 100.0;
    }

    // v0.0.464: Find top topic (domain with most tickets)
    stats.top_topic = stats
        .by_domain
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(domain, _)| domain.clone());

    // v0.0.464: Filter repeated queries to only show actually repeated ones (count > 1)
    stats.repeated_queries.retain(|_, count| *count > 1);

    stats
}

/// Derive outcome from ticket log (for legacy tickets without explicit outcome)
fn derive_outcome(ticket: &TicketLog) -> TicketOutcome {
    // If ticket has explicit error kind, derive from that
    if let Some(ref kind) = ticket.error_kind {
        return kind.to_outcome();
    }

    // If ticket has explicit state
    if let Some(state) = &ticket.state {
        match state {
            TicketState::Failed => {
                return TicketOutcome::ErrorInternal;
            }
            TicketState::Success => {
                // Derive from result and reliability
                if ticket.result == crate::ticket_log::TicketResult::Success {
                    if ticket.reliability_score >= 80 {
                        return TicketOutcome::Success;
                    } else if ticket.reliability_score >= 50 {
                        return TicketOutcome::Partial;
                    }
                }
                return TicketOutcome::CannotAnswerSafely;
            }
            _ => {}
        }
    }

    // Legacy: derive from result field
    match ticket.result {
        crate::ticket_log::TicketResult::Success => {
            if ticket.reliability_score >= 80 {
                TicketOutcome::Success
            } else if ticket.reliability_score >= 50 {
                TicketOutcome::Partial
            } else {
                TicketOutcome::CannotAnswerSafely
            }
        }
        crate::ticket_log::TicketResult::Partial => TicketOutcome::Partial,
        crate::ticket_log::TicketResult::Failed => TicketOutcome::ErrorInternal,
        crate::ticket_log::TicketResult::NeedsClarification => TicketOutcome::CannotAnswerSafely,
        crate::ticket_log::TicketResult::Cancelled => TicketOutcome::CannotAnswerSafely,
    }
}

/// Categorize handler into "recipe", "llm", or "deterministic"
fn categorize_handler(handler: &str) -> String {
    if handler.starts_with("recipe:") {
        "recipe".to_string()
    } else if handler.starts_with("llm:") || handler.contains("specialist") {
        "llm".to_string()
    } else if handler.contains("deterministic") || handler.contains("direct") {
        "deterministic".to_string()
    } else {
        "other".to_string()
    }
}

/// v0.0.464: Normalize query for duplicate detection
/// Lowercase, trim, remove extra whitespace
fn normalize_query(query: &str) -> String {
    query
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl std::fmt::Display for TicketStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[service desk]")?;
        writeln!(f, "  total_tickets         {}", self.total)?;
        writeln!(f, "  success               {}", self.success)?;
        writeln!(f, "  partial               {}", self.partial)?;
        writeln!(f, "  cannot_answer_safely  {}", self.cannot_answer)?;
        writeln!(f, "  failed                {}", self.failed)?;
        writeln!(f, "  success_rate          {:.1}%", self.success_rate)?;
        writeln!(f, "  resolution_rate       {:.1}%", self.resolution_rate)?;
        writeln!(f, "  error_rate            {:.1}%", self.error_rate)?;
        writeln!(f, "  avg_response          {}ms", self.avg_duration_ms)?;
        writeln!(f)?;
        writeln!(f, "[errors]")?;

        // Count error types
        let parse_errors = self.by_error.get("llm_parse_error").unwrap_or(&0)
            + self.by_error.get("validation_failed").unwrap_or(&0);
        let timeouts = self.by_error.get("llm_timeout").unwrap_or(&0);
        let tool_failures = self.by_error.get("probe_failure").unwrap_or(&0);
        let internal = self.by_error.get("internal_error").unwrap_or(&0);

        writeln!(f, "  parse_errors          {}", parse_errors)?;
        writeln!(f, "  timeouts              {}", timeouts)?;
        writeln!(f, "  tool_failures         {}", tool_failures)?;
        writeln!(f, "  internal_errors       {}", internal)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::SpecialistDomain;
    use crate::ticket_log::TicketResult;
    use crate::ticket_state::ErrorKind;

    #[test]
    fn test_calculate_stats_basic() {
        let tickets = vec![
            TicketLog::new("T1", SpecialistDomain::System, "diagnose", "q1")
                .with_handler("recipe:check_mem")
                .with_metrics(100, 90) // High reliability = Success
                .with_answer("ok", TicketResult::Success),
            TicketLog::new("T2", SpecialistDomain::Network, "diagnose", "q2")
                .with_handler("llm:junior")
                .with_metrics(500, 80) // Good reliability = Success
                .with_answer("ok", TicketResult::Success),
            TicketLog::new("T3", SpecialistDomain::Storage, "diagnose", "q3")
                .with_handler("llm:senior")
                .with_metrics(1000, 60) // Low reliability = Partial
                .with_answer("partial", TicketResult::Partial),
        ];

        let stats = calculate_stats(&tickets);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.partial, 1);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.by_handler.get("recipe"), Some(&1));
        assert_eq!(stats.by_handler.get("llm"), Some(&2));
    }

    #[test]
    fn test_calculate_stats_empty() {
        let stats = calculate_stats(&[]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.error_rate, 0.0);
    }

    #[test]
    fn test_calculate_stats_llm_failures() {
        let mut ticket = TicketLog::new("T1", SpecialistDomain::System, "diagnose", "q1");
        ticket.error_kind = Some(ErrorKind::LlmTimeout);
        ticket.state = Some(TicketState::Failed);

        let stats = calculate_stats(&[ticket]);
        assert_eq!(stats.llm_failed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.error_rate, 100.0);
    }

    #[test]
    fn test_calculate_stats_validation_failures() {
        // v0.0.409: Validation failures should count as LLM failures
        let mut ticket = TicketLog::new("T1", SpecialistDomain::System, "diagnose", "q1");
        ticket.error_kind = Some(ErrorKind::ValidationFailed);
        ticket.state = Some(TicketState::Failed);

        let stats = calculate_stats(&[ticket]);
        assert_eq!(stats.llm_failed, 1); // Validation failure is an LLM failure
        assert_eq!(stats.failed, 1);
        assert!(stats.by_outcome.contains_key("error_parse"));
    }

    #[test]
    fn test_calculate_stats_outcomes() {
        let tickets = vec![
            TicketLog::new("T1", SpecialistDomain::System, "diagnose", "q1")
                .with_metrics(100, 90)
                .with_answer("ok", TicketResult::Success),
            TicketLog::new("T2", SpecialistDomain::Network, "diagnose", "q2")
                .with_metrics(200, 60)
                .with_answer("partial", TicketResult::Partial),
            TicketLog::new("T3", SpecialistDomain::Storage, "diagnose", "q3")
                .with_error(ErrorKind::LlmTimeout),
        ];

        let stats = calculate_stats(&tickets);
        assert_eq!(stats.success, 1);
        assert_eq!(stats.partial, 1);
        assert_eq!(stats.failed, 1);
        // Resolution rate = (success + partial + cannot_answer) / total
        // = (1 + 1 + 0) / 3 = 66.66%
        assert!(stats.resolution_rate > 66.0 && stats.resolution_rate < 67.0);
    }

    #[test]
    fn test_categorize_handler() {
        assert_eq!(categorize_handler("recipe:check_disk"), "recipe");
        assert_eq!(categorize_handler("llm:junior"), "llm");
        assert_eq!(categorize_handler("deterministic:facts"), "deterministic");
        assert_eq!(categorize_handler("unknown"), "other");
    }

    #[test]
    fn test_stats_display() {
        let stats = TicketStats {
            total: 100,
            success: 70,
            partial: 15,
            cannot_answer: 5,
            failed: 10,
            success_rate: 70.0,
            resolution_rate: 90.0,
            error_rate: 10.0,
            ..Default::default()
        };

        let display = format!("{}", stats);
        assert!(display.contains("total_tickets         100"));
        assert!(display.contains("success               70"));
        assert!(display.contains("error_rate            10.0%"));
    }
}
