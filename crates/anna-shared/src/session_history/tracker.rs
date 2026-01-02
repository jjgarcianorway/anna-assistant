//! Session history tracker implementation

use crate::session_history::types::{SessionOutcome, SessionRecord, SessionType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
