//! Session History Tracker - Phase 94
//!
//! Tracks user session history with Anna.
//! Useful for "what did I do last time" queries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Session history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionHistoryTracker {
    /// All sessions
    pub sessions: Vec<SessionRecord>,
    /// Count by outcome
    pub by_outcome: HashMap<String, u64>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Total queries across all sessions
    pub total_queries: u64,
    /// Total tickets resolved
    pub total_tickets: u64,
}

impl SessionHistoryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new session
    pub fn start_session(&mut self, id: String, session_type: SessionType, timestamp: u64) {
        let record = SessionRecord {
            id,
            session_type,
            outcome: SessionOutcome::Completed,
            started_at: timestamp,
            ended_at: None,
            duration_secs: None,
            query_count: 0,
            tickets_resolved: 0,
            topics: vec![],
            summary: None,
        };
        *self.by_type.entry(session_type.name().to_string()).or_insert(0) += 1;
        self.sessions.push(record);
    }

    /// End a session
    pub fn end_session(&mut self, id: &str, outcome: SessionOutcome, timestamp: u64) -> bool {
        let found = self.sessions.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.sessions[idx].outcome = outcome;
            self.sessions[idx].ended_at = Some(timestamp);
            self.sessions[idx].duration_secs =
                Some(timestamp.saturating_sub(self.sessions[idx].started_at));
            *self.by_outcome.entry(outcome.name().to_string()).or_insert(0) += 1;
            self.total_queries += self.sessions[idx].query_count;
            self.total_tickets += self.sessions[idx].tickets_resolved;
            true
        } else {
            false
        }
    }

    /// Record a query in session
    pub fn record_query(&mut self, id: &str, topic: Option<&str>) {
        let found = self.sessions.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.sessions[idx].query_count += 1;
            if let Some(t) = topic {
                if !self.sessions[idx].topics.contains(&t.to_string()) {
                    self.sessions[idx].topics.push(t.to_string());
                }
            }
        }
    }

    /// Record a ticket resolved in session
    pub fn record_ticket(&mut self, id: &str) {
        let found = self.sessions.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.sessions[idx].tickets_resolved += 1;
        }
    }

    /// Get session by ID
    pub fn get(&self, id: &str) -> Option<&SessionRecord> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Get most recent session
    pub fn latest(&self) -> Option<&SessionRecord> {
        self.sessions.last()
    }

    /// Get recent sessions
    pub fn recent(&self, limit: usize) -> Vec<&SessionRecord> {
        self.sessions.iter().rev().take(limit).collect()
    }

    /// Get sessions by outcome
    pub fn by_session_outcome(&self, outcome: SessionOutcome) -> Vec<&SessionRecord> {
        self.sessions.iter().filter(|s| s.outcome == outcome).collect()
    }

    /// Get sessions by type
    pub fn by_session_type(&self, session_type: SessionType) -> Vec<&SessionRecord> {
        self.sessions.iter().filter(|s| s.session_type == session_type).collect()
    }

    /// Total session count
    pub fn total_count(&self) -> usize {
        self.sessions.len()
    }

    /// Completed session count
    pub fn completed_count(&self) -> usize {
        self.sessions.iter().filter(|s| s.outcome == SessionOutcome::Completed).count()
    }

    /// Average session duration
    pub fn avg_duration(&self) -> u64 {
        let durations: Vec<u64> =
            self.sessions.iter().filter_map(|s| s.duration_secs).collect();
        if durations.is_empty() {
            0
        } else {
            durations.iter().sum::<u64>() / durations.len() as u64
        }
    }
}

/// Format session history for display
pub fn format_session_history(tracker: &SessionHistoryTracker) -> String {
    let mut lines = vec!["=== Session History ===".to_string()];
    lines.push(String::new());

    if tracker.sessions.is_empty() {
        lines.push("No sessions recorded yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total sessions: {}", tracker.total_count()));
    lines.push(format!("Completed: {}", tracker.completed_count()));
    lines.push(format!("Total queries: {}", tracker.total_queries));
    lines.push(format!("Total tickets: {}", tracker.total_tickets));
    lines.push(format!("Avg duration: {}s", tracker.avg_duration()));

    // Recent sessions
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent sessions:".to_string());
        for session in recent {
            let duration = session.duration_secs.unwrap_or(0);
            lines.push(format!(
                "  [{}] {} - {} queries, {}s",
                session.outcome.symbol(),
                session.session_type.name(),
                session.query_count,
                duration
            ));
        }
    }

    lines.join("\n")
}

/// Format session history compact
pub fn format_session_history_compact(tracker: &SessionHistoryTracker) -> String {
    format!(
        "Sessions: {} total | {} completed | {} queries",
        tracker.total_count(),
        tracker.completed_count(),
        tracker.total_queries
    )
}

/// Format session history one-line
pub fn format_session_history_oneline(tracker: &SessionHistoryTracker) -> String {
    format!(
        "{} sessions ({} completed)",
        tracker.total_count(),
        tracker.completed_count()
    )
}

/// Check if query is about sessions
pub fn is_session_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "session",
        "sessions",
        "last time",
        "previous session",
        "what did i do",
        "history",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about sessions
pub fn session_fun_fact(tracker: &SessionHistoryTracker) -> String {
    if tracker.sessions.is_empty() {
        return "No sessions recorded yet!".to_string();
    }

    let facts = [
        format!("You've had {} sessions with Anna.", tracker.total_count()),
        format!("Anna has answered {} queries.", tracker.total_queries),
        format!("Anna has resolved {} tickets.", tracker.total_tickets),
    ];

    facts[tracker.total_count() % facts.len()].clone()
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

    #[test]
    fn test_start_session() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.get("sess1").is_some());
    }

    #[test]
    fn test_end_session() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.end_session("sess1", SessionOutcome::Completed, 2000);

        let session = tracker.get("sess1").unwrap();
        assert_eq!(session.duration_secs, Some(1000));
        assert_eq!(session.outcome, SessionOutcome::Completed);
    }

    #[test]
    fn test_record_query() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.record_query("sess1", Some("packages"));
        tracker.record_query("sess1", Some("services"));

        let session = tracker.get("sess1").unwrap();
        assert_eq!(session.query_count, 2);
        assert_eq!(session.topics.len(), 2);
    }

    #[test]
    fn test_record_ticket() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.record_ticket("sess1");
        tracker.record_ticket("sess1");

        let session = tracker.get("sess1").unwrap();
        assert_eq!(session.tickets_resolved, 2);
    }

    #[test]
    fn test_recent_sessions() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.start_session("sess2".to_string(), SessionType::OneShot, 2000);
        tracker.start_session("sess3".to_string(), SessionType::Interactive, 3000);

        let recent = tracker.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "sess3");
    }

    #[test]
    fn test_by_outcome() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.end_session("sess1", SessionOutcome::Completed, 2000);
        tracker.start_session("sess2".to_string(), SessionType::Interactive, 3000);
        tracker.end_session("sess2", SessionOutcome::Error, 4000);

        assert_eq!(tracker.by_session_outcome(SessionOutcome::Completed).len(), 1);
        assert_eq!(tracker.by_session_outcome(SessionOutcome::Error).len(), 1);
    }

    #[test]
    fn test_avg_duration() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.end_session("sess1", SessionOutcome::Completed, 2000);
        tracker.start_session("sess2".to_string(), SessionType::Interactive, 3000);
        tracker.end_session("sess2", SessionOutcome::Completed, 6000);

        // (1000 + 3000) / 2 = 2000
        assert_eq!(tracker.avg_duration(), 2000);
    }

    #[test]
    fn test_format_history() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.end_session("sess1", SessionOutcome::Completed, 2000);

        let output = format_session_history(&tracker);
        assert!(output.contains("Session History"));
        assert!(output.contains("Total sessions: 1"));
    }

    #[test]
    fn test_is_session_query() {
        assert!(is_session_query("what did I do last time?"));
        assert!(is_session_query("show session history"));
        assert!(is_session_query("previous session"));
        assert!(!is_session_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);

        let fact = session_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
