//! Session types and enums

use serde::{Deserialize, Serialize};

/// Session outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SessionOutcome {
    #[default]
    Completed,
    Abandoned,
    Error,
    Timeout,
    UserExit,
}

impl SessionOutcome {
    pub fn name(&self) -> &'static str {
        match self {
            SessionOutcome::Completed => "Completed",
            SessionOutcome::Abandoned => "Abandoned",
            SessionOutcome::Error => "Error",
            SessionOutcome::Timeout => "Timeout",
            SessionOutcome::UserExit => "User Exit",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            SessionOutcome::Completed => "✓",
            SessionOutcome::Abandoned => "○",
            SessionOutcome::Error => "✗",
            SessionOutcome::Timeout => "⏱",
            SessionOutcome::UserExit => "→",
        }
    }
}

/// Session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SessionType {
    #[default]
    Interactive,
    OneShot,
    Background,
    Scheduled,
}

impl SessionType {
    pub fn name(&self) -> &'static str {
        match self {
            SessionType::Interactive => "Interactive",
            SessionType::OneShot => "One-shot",
            SessionType::Background => "Background",
            SessionType::Scheduled => "Scheduled",
        }
    }
}

/// A session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique session ID
    pub id: String,
    /// Session type
    pub session_type: SessionType,
    /// Session outcome
    pub outcome: SessionOutcome,
    /// Start time (Unix timestamp)
    pub started_at: u64,
    /// End time (Unix timestamp)
    pub ended_at: Option<u64>,
    /// Duration in seconds
    pub duration_secs: Option<u64>,
    /// Number of queries in session
    pub query_count: u64,
    /// Number of tickets resolved
    pub tickets_resolved: u64,
    /// Main topics discussed
    pub topics: Vec<String>,
    /// Summary of session (auto-generated)
    pub summary: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_outcome() {
        assert_eq!(SessionOutcome::Completed.name(), "Completed");
        assert_eq!(SessionOutcome::Completed.symbol(), "✓");
    }

    #[test]
    fn test_session_type() {
        assert_eq!(SessionType::Interactive.name(), "Interactive");
        assert_eq!(SessionType::OneShot.name(), "One-shot");
    }
}
