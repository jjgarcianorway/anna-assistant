//! Event record types (v0.0.190).

use serde::{Deserialize, Serialize};

/// Single event record for the log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// Unique request ID
    pub request_id: String,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Query class (e.g., "memory_usage", "configure_editor")
    pub query_class: String,
    /// Outcome: "verified", "failed", "timeout", "clarification"
    pub outcome: String,
    /// Reliability score (0-100)
    pub reliability: u8,
    /// Team that handled the request
    pub team: String,
    /// Whether escalation occurred
    pub escalated: bool,
    /// Escalation tier reached (0=none, 1=junior, 2=senior, 3=manager)
    pub escalation_tier: u8,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Number of interactions (clarifications, retries)
    pub interactions: u32,
    /// Recipe ID if one was used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_used: Option<String>,
    /// Recipe ID if one was learned
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_learned: Option<String>,
}

impl EventRecord {
    pub fn new(request_id: &str, query_class: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            query_class: query_class.to_string(),
            outcome: "pending".to_string(),
            reliability: 0,
            team: "unknown".to_string(),
            escalated: false,
            escalation_tier: 0,
            duration_ms: 0,
            interactions: 0,
            recipe_used: None,
            recipe_learned: None,
        }
    }

    /// Mark as verified
    pub fn verified(mut self, reliability: u8) -> Self {
        self.outcome = "verified".to_string();
        self.reliability = reliability;
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.outcome = "failed".to_string();
        self
    }

    /// Mark as timeout
    pub fn timeout(mut self) -> Self {
        self.outcome = "timeout".to_string();
        self
    }

    /// Set team
    pub fn with_team(mut self, team: &str) -> Self {
        self.team = team.to_string();
        self
    }

    /// Set escalation info
    pub fn with_escalation(mut self, tier: u8) -> Self {
        self.escalated = tier > 0;
        self.escalation_tier = tier;
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}
