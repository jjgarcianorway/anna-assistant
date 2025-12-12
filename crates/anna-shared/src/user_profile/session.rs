//! Session history tracking for "since last time" summaries (v0.0.238).
//!
//! Tracks what happened in each session to provide meaningful
//! "since last time" summaries when users return.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary of a single session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session ended
    pub ended_at: DateTime<Utc>,
    /// Number of queries made
    pub query_count: u32,
    /// Topics discussed (topic -> count)
    pub topics: HashMap<String, u32>,
    /// Commands learned (new commands user was introduced to)
    pub commands_learned: Vec<String>,
    /// Any recipes executed
    pub recipes_executed: Vec<String>,
    /// Whether system health was checked
    pub checked_health: bool,
    /// Notable events (errors resolved, configs changed, etc.)
    pub notable_events: Vec<String>,
}

impl SessionSummary {
    /// Create a new session starting now
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            started_at: now,
            ended_at: now,
            query_count: 0,
            topics: HashMap::new(),
            commands_learned: Vec::new(),
            recipes_executed: Vec::new(),
            checked_health: false,
            notable_events: Vec::new(),
        }
    }

    /// Record a query with its detected topic
    pub fn record_query(&mut self, topic: Option<&str>) {
        self.query_count += 1;
        self.ended_at = Utc::now();
        if let Some(t) = topic {
            *self.topics.entry(t.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a command the user learned
    pub fn record_command_learned(&mut self, command: &str) {
        if !self.commands_learned.contains(&command.to_string()) {
            self.commands_learned.push(command.to_string());
        }
        self.ended_at = Utc::now();
    }

    /// Record a recipe that was executed
    pub fn record_recipe(&mut self, recipe_id: &str) {
        if !self.recipes_executed.contains(&recipe_id.to_string()) {
            self.recipes_executed.push(recipe_id.to_string());
        }
        self.ended_at = Utc::now();
    }

    /// Record a notable event
    pub fn record_event(&mut self, event: &str) {
        self.notable_events.push(event.to_string());
        self.ended_at = Utc::now();
    }

    /// Mark that health was checked
    pub fn mark_health_check(&mut self) {
        self.checked_health = true;
        self.ended_at = Utc::now();
    }

    /// Get a human-readable summary of what happened
    pub fn summarize(&self) -> Vec<String> {
        let mut summary = Vec::new();

        if self.query_count > 0 {
            let q_word = if self.query_count == 1 {
                "query"
            } else {
                "queries"
            };
            summary.push(format!("Handled {} {}", self.query_count, q_word));
        }

        // Top topic
        if let Some((top_topic, count)) = self.topics.iter().max_by_key(|(_, v)| *v) {
            if *count > 1 {
                summary.push(format!("Discussed {} ({} times)", top_topic, count));
            }
        }

        // Commands learned
        if !self.commands_learned.is_empty() {
            if self.commands_learned.len() == 1 {
                summary.push(format!("Learned about: {}", self.commands_learned[0]));
            } else {
                summary.push(format!(
                    "Learned {} new commands",
                    self.commands_learned.len()
                ));
            }
        }

        // Recipes
        if !self.recipes_executed.is_empty() {
            if self.recipes_executed.len() == 1 {
                summary.push(format!("Ran recipe: {}", self.recipes_executed[0]));
            } else {
                summary.push(format!("Ran {} recipes", self.recipes_executed.len()));
            }
        }

        summary
    }
}

impl Default for SessionSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// History of recent sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionHistory {
    /// Recent sessions (most recent first), limited to last 5
    pub sessions: Vec<SessionSummary>,
}

impl SessionHistory {
    /// Get the last session (before current)
    pub fn last_session(&self) -> Option<&SessionSummary> {
        self.sessions.first()
    }

    /// Add a completed session
    pub fn add_session(&mut self, session: SessionSummary) {
        // Only add if it had activity
        if session.query_count > 0 {
            self.sessions.insert(0, session);
            // Keep only last 5 sessions
            self.sessions.truncate(5);
        }
    }

    /// Generate "since last time" message
    pub fn since_last_time(&self) -> Option<String> {
        let last = self.last_session()?;
        let summary = last.summarize();

        if summary.is_empty() {
            return None;
        }

        // Format time since last session
        let elapsed = Utc::now() - last.ended_at;
        let time_str = if elapsed.num_days() > 0 {
            format!(
                "{} day{} ago",
                elapsed.num_days(),
                if elapsed.num_days() == 1 { "" } else { "s" }
            )
        } else if elapsed.num_hours() > 0 {
            format!(
                "{} hour{} ago",
                elapsed.num_hours(),
                if elapsed.num_hours() == 1 { "" } else { "s" }
            )
        } else {
            "earlier today".to_string()
        };

        Some(format!(
            "Last time ({}): {}",
            time_str,
            summary.join(", ").to_lowercase()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary_new() {
        let session = SessionSummary::new();
        assert_eq!(session.query_count, 0);
        assert!(session.topics.is_empty());
    }

    #[test]
    fn test_session_record_query() {
        let mut session = SessionSummary::new();
        session.record_query(Some("vim"));
        session.record_query(Some("vim"));
        session.record_query(Some("disk"));

        assert_eq!(session.query_count, 3);
        assert_eq!(session.topics.get("vim"), Some(&2));
        assert_eq!(session.topics.get("disk"), Some(&1));
    }

    #[test]
    fn test_session_summarize() {
        let mut session = SessionSummary::new();
        session.record_query(Some("vim"));
        session.record_query(Some("vim"));
        session.record_command_learned("nvim");

        let summary = session.summarize();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_session_history_truncate() {
        let mut history = SessionHistory::default();
        for _ in 0..10 {
            let mut session = SessionSummary::new();
            session.record_query(Some("test"));
            history.add_session(session);
        }
        assert_eq!(history.sessions.len(), 5);
    }
}
