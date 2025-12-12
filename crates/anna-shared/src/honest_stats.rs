//! Honest Stats Tracking (v0.0.415).
//!
//! Stats that reflect reality, not fantasy.
//! The "100% success" lie must die.
//!
//! Rules:
//! - "resolved" = status=ok + confidence>=0.8 + non-empty summary + valid response
//! - "partial" = status=partial
//! - "failed" = status=failed OR parse error OR timeout

use crate::strict_contract::{StrictSpecialistResponse, StrictStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Honest ticket outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// Fully resolved with high confidence
    Resolved,
    /// Partially answered
    Partial,
    /// Failed to answer
    Failed,
}

impl TicketOutcome {
    /// Derive from specialist response
    pub fn from_response(response: &StrictSpecialistResponse) -> Self {
        if response.is_resolved() {
            Self::Resolved
        } else if response.status == StrictStatus::Partial {
            Self::Partial
        } else {
            Self::Failed
        }
    }

    /// Derive from parse failure
    pub fn from_parse_error() -> Self {
        Self::Failed
    }

    /// Derive from timeout
    pub fn from_timeout() -> Self {
        Self::Failed
    }
}

/// Honest stats store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestStats {
    /// Total tickets processed
    pub total_tickets: u64,

    /// Tickets fully resolved
    pub resolved: u64,

    /// Tickets partially answered
    pub partial: u64,

    /// Tickets failed
    pub failed: u64,

    /// Average response time (ms)
    pub avg_response_ms: f64,

    /// Parse errors
    pub parse_errors: u64,

    /// Timeouts
    pub timeouts: u64,

    /// Per-domain breakdown
    pub by_domain: HashMap<String, DomainStats>,

    /// Per-intent breakdown
    pub by_intent: HashMap<String, IntentStats>,

    /// Recent failures for debugging
    #[serde(default)]
    pub recent_failures: Vec<FailureRecord>,
}

/// Per-domain statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainStats {
    pub total: u64,
    pub resolved: u64,
    pub partial: u64,
    pub failed: u64,
}

/// Per-intent statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentStats {
    pub total: u64,
    pub resolved: u64,
    pub failed: u64,
    pub avg_confidence: f32,
}

/// Record of a failure for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub ticket_id: String,
    pub domain: String,
    pub intent: String,
    pub reason: String,
    pub timestamp: u64,
}

impl HonestStats {
    /// Load from disk
    pub fn load() -> Self {
        let path = stats_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = stats_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Record a ticket outcome
    pub fn record(
        &mut self,
        ticket_id: &str,
        domain: &str,
        intent: &str,
        outcome: TicketOutcome,
        response_ms: u64,
        confidence: f32,
    ) {
        self.total_tickets += 1;

        match outcome {
            TicketOutcome::Resolved => self.resolved += 1,
            TicketOutcome::Partial => self.partial += 1,
            TicketOutcome::Failed => self.failed += 1,
        }

        // Update average response time
        let n = self.total_tickets as f64;
        self.avg_response_ms = (self.avg_response_ms * (n - 1.0) + response_ms as f64) / n;

        // Update domain stats
        let domain_stats = self.by_domain.entry(domain.to_string()).or_default();
        domain_stats.total += 1;
        match outcome {
            TicketOutcome::Resolved => domain_stats.resolved += 1,
            TicketOutcome::Partial => domain_stats.partial += 1,
            TicketOutcome::Failed => domain_stats.failed += 1,
        }

        // Update intent stats
        let intent_stats = self.by_intent.entry(intent.to_string()).or_default();
        intent_stats.total += 1;
        match outcome {
            TicketOutcome::Resolved => intent_stats.resolved += 1,
            TicketOutcome::Failed => intent_stats.failed += 1,
            _ => {}
        }
        let n = intent_stats.total as f32;
        intent_stats.avg_confidence = (intent_stats.avg_confidence * (n - 1.0) + confidence) / n;

        // Record failure for debugging
        if outcome == TicketOutcome::Failed {
            self.recent_failures.push(FailureRecord {
                ticket_id: ticket_id.to_string(),
                domain: domain.to_string(),
                intent: intent.to_string(),
                reason: "Unknown".to_string(),
                timestamp: current_secs(),
            });
            // Keep only last 50 failures
            if self.recent_failures.len() > 50 {
                self.recent_failures.remove(0);
            }
        }
    }

    /// Record a parse error
    pub fn record_parse_error(&mut self, ticket_id: &str, domain: &str, intent: &str, error: &str) {
        self.parse_errors += 1;
        self.record(ticket_id, domain, intent, TicketOutcome::Failed, 0, 0.0);

        // Update failure record with parse error reason
        if let Some(last) = self.recent_failures.last_mut() {
            if last.ticket_id == ticket_id {
                last.reason = format!("Parse error: {}", truncate(error, 100));
            }
        }
    }

    /// Record a timeout
    pub fn record_timeout(&mut self, ticket_id: &str, domain: &str, intent: &str, elapsed_ms: u64) {
        self.timeouts += 1;
        self.record(
            ticket_id,
            domain,
            intent,
            TicketOutcome::Failed,
            elapsed_ms,
            0.0,
        );

        // Update failure record with timeout reason
        if let Some(last) = self.recent_failures.last_mut() {
            if last.ticket_id == ticket_id {
                last.reason = format!("Timeout after {}ms", elapsed_ms);
            }
        }
    }

    /// Get success rate (honest)
    pub fn success_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            0.0
        } else {
            (self.resolved as f64 / self.total_tickets as f64) * 100.0
        }
    }

    /// Get failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            0.0
        } else {
            (self.failed as f64 / self.total_tickets as f64) * 100.0
        }
    }

    /// Format for display
    pub fn format_summary(&self) -> String {
        format!(
            "Tickets: {} total | {} resolved ({:.1}%) | {} partial | {} failed ({:.1}%)\nAvg response: {:.0}ms | Parse errors: {} | Timeouts: {}",
            self.total_tickets,
            self.resolved,
            self.success_rate(),
            self.partial,
            self.failed,
            self.failure_rate(),
            self.avg_response_ms,
            self.parse_errors,
            self.timeouts
        )
    }

    /// Reset stats (for testing)
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn stats_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("anna")
        .join("honest_stats.json")
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_from_response() {
        let good = StrictSpecialistResponse::ok("DSK-001", "query", "Answer", 0.95)
            .with_evidence("probe", "data");
        assert_eq!(TicketOutcome::from_response(&good), TicketOutcome::Resolved);

        let partial = StrictSpecialistResponse::partial("DSK-001", "query", "Partial");
        assert_eq!(
            TicketOutcome::from_response(&partial),
            TicketOutcome::Partial
        );

        let failed = StrictSpecialistResponse::failed("DSK-001", "query", "Error");
        assert_eq!(TicketOutcome::from_response(&failed), TicketOutcome::Failed);
    }

    #[test]
    fn test_honest_stats_tracking() {
        let mut stats = HonestStats::default();

        // Record some outcomes
        stats.record(
            "DSK-001",
            "system",
            "query_metric",
            TicketOutcome::Resolved,
            100,
            0.95,
        );
        stats.record(
            "DSK-002",
            "system",
            "query_metric",
            TicketOutcome::Resolved,
            150,
            0.90,
        );
        stats.record(
            "DSK-003",
            "system",
            "query_metric",
            TicketOutcome::Failed,
            200,
            0.0,
        );

        assert_eq!(stats.total_tickets, 3);
        assert_eq!(stats.resolved, 2);
        assert_eq!(stats.failed, 1);
        assert!((stats.success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_parse_error_tracking() {
        let mut stats = HonestStats::default();

        stats.record_parse_error("DSK-001", "system", "query", "No JSON found");

        assert_eq!(stats.parse_errors, 1);
        assert_eq!(stats.failed, 1);
        assert!(!stats.recent_failures.is_empty());
    }

    #[test]
    fn test_timeout_tracking() {
        let mut stats = HonestStats::default();

        stats.record_timeout("DSK-001", "system", "query", 5000);

        assert_eq!(stats.timeouts, 1);
        assert_eq!(stats.failed, 1);
    }

    #[test]
    fn test_format_summary() {
        let mut stats = HonestStats::default();
        stats.record(
            "DSK-001",
            "system",
            "query",
            TicketOutcome::Resolved,
            100,
            0.95,
        );

        let summary = stats.format_summary();
        assert!(summary.contains("1 total"));
        assert!(summary.contains("1 resolved"));
        assert!(summary.contains("100.0%"));
    }
}
