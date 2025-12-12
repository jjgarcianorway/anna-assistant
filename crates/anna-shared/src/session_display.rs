//! Session Summary Display.
//!
//! Provides formatted display of session information:
//! - Current session stats
//! - Session history summary
//! - "Since last time" information
//!
//! Per VISION.md: Session summaries in greetings

use crate::user_profile::{SessionHistory, SessionSummary, UserProfile};
use chrono::{DateTime, Utc};

/// Current session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Session duration in seconds
    pub duration_secs: i64,
    /// Number of queries
    pub query_count: u32,
    /// Unique topics discussed
    pub topics_count: usize,
    /// Commands learned this session
    pub commands_learned: usize,
    /// Recipes executed
    pub recipes_run: usize,
    /// Notable events
    pub events_count: usize,
}

impl SessionStats {
    /// Build from session summary
    pub fn from_session(session: &SessionSummary) -> Self {
        let duration = session.ended_at - session.started_at;
        Self {
            duration_secs: duration.num_seconds(),
            query_count: session.query_count,
            topics_count: session.topics.len(),
            commands_learned: session.commands_learned.len(),
            recipes_run: session.recipes_executed.len(),
            events_count: session.notable_events.len(),
        }
    }

    /// Check if session had any activity
    pub fn has_activity(&self) -> bool {
        self.query_count > 0
    }
}

/// Format session duration
pub fn format_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

/// Format time ago
pub fn format_time_ago(dt: DateTime<Utc>) -> String {
    let elapsed = Utc::now() - dt;
    let days = elapsed.num_days();
    let hours = elapsed.num_hours();
    let mins = elapsed.num_minutes();

    if days > 0 {
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if mins > 0 {
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else {
        "just now".to_string()
    }
}

/// Format current session summary
pub fn format_current_session(session: &SessionSummary) -> String {
    let stats = SessionStats::from_session(session);
    let mut lines = vec![];

    lines.push("Current Session".to_string());
    lines.push(format!("  duration      {}", format_duration(stats.duration_secs)));
    lines.push(format!("  queries       {}", stats.query_count));

    if stats.topics_count > 0 {
        let topics: Vec<String> = session.topics.keys().take(3).cloned().collect();
        lines.push(format!("  topics        {} ({})", stats.topics_count, topics.join(", ")));
    }

    if stats.commands_learned > 0 {
        lines.push(format!("  learned       {} commands", stats.commands_learned));
    }

    if stats.recipes_run > 0 {
        lines.push(format!("  recipes       {} run", stats.recipes_run));
    }

    if !session.notable_events.is_empty() {
        lines.push(format!("  events        {}", session.notable_events.len()));
    }

    lines.join("\n")
}

/// Format session history summary
pub fn format_session_history(history: &SessionHistory) -> String {
    if history.sessions.is_empty() {
        return "No previous sessions".to_string();
    }

    let mut lines = vec![];
    lines.push(format!("Session History ({} sessions)", history.sessions.len()));

    for (i, session) in history.sessions.iter().take(5).enumerate() {
        let time_ago = format_time_ago(session.ended_at);
        let duration = format_duration((session.ended_at - session.started_at).num_seconds());
        let topics: Vec<String> = session.topics.keys().take(2).cloned().collect();
        let topics_str = if topics.is_empty() {
            "".to_string()
        } else {
            format!(" - {}", topics.join(", "))
        };

        lines.push(format!(
            "  {}. {} ({}, {} queries){}",
            i + 1,
            time_ago,
            duration,
            session.query_count,
            topics_str
        ));
    }

    lines.join("\n")
}

/// Get "since last time" summary line for greetings
pub fn get_since_last_time(profile: &UserProfile) -> Option<String> {
    profile.session_history.since_last_time()
}

/// Format a brief session summary for display
pub fn format_brief_summary(session: &SessionSummary) -> String {
    let summary = session.summarize();
    if summary.is_empty() {
        "No activity".to_string()
    } else {
        summary.join(", ")
    }
}

/// Check if query is asking about session info
pub fn is_session_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "session summary", "session stats", "session info",
        "what did we do", "this session", "session history",
        "show session", "current session"
    ])
}

/// Get the type of session query
pub fn get_session_query_type(query: &str) -> SessionQueryType {
    let lower = query.to_lowercase();

    if matches_any(&lower, &["session history", "past sessions", "previous sessions"]) {
        SessionQueryType::History
    } else if matches_any(&lower, &["current session", "this session", "session stats"]) {
        SessionQueryType::Current
    } else {
        SessionQueryType::Summary
    }
}

/// Type of session query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionQueryType {
    /// Current session info
    Current,
    /// Session history
    History,
    /// General summary
    Summary,
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session() -> SessionSummary {
        let mut session = SessionSummary::new();
        session.query_count = 5;
        session.topics.insert("vim".to_string(), 3);
        session.topics.insert("docker".to_string(), 2);
        session.commands_learned.push("nvim".to_string());
        session.recipes_executed.push("restart-docker".to_string());
        session
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3700), "1h 1m");
    }

    #[test]
    fn test_session_stats() {
        let session = create_test_session();
        let stats = SessionStats::from_session(&session);

        assert_eq!(stats.query_count, 5);
        assert_eq!(stats.topics_count, 2);
        assert_eq!(stats.commands_learned, 1);
        assert_eq!(stats.recipes_run, 1);
        assert!(stats.has_activity());
    }

    #[test]
    fn test_format_current_session() {
        let session = create_test_session();
        let output = format_current_session(&session);

        assert!(output.contains("Current Session"));
        assert!(output.contains("queries"));
        assert!(output.contains("5"));
    }

    #[test]
    fn test_format_session_history() {
        let mut history = SessionHistory::default();
        let mut session = create_test_session();
        session.record_query(Some("test")); // Update ended_at
        history.add_session(session);

        let output = format_session_history(&history);
        assert!(output.contains("Session History"));
        assert!(output.contains("1 sessions"));
    }

    #[test]
    fn test_format_brief_summary() {
        let session = create_test_session();
        let summary = format_brief_summary(&session);
        assert!(summary.contains("queries") || summary.contains("Handled"));
    }

    #[test]
    fn test_is_session_query() {
        assert!(is_session_query("show me session stats"));
        assert!(is_session_query("what did we do this session"));
        assert!(!is_session_query("how much disk space"));
    }

    #[test]
    fn test_get_session_query_type() {
        assert_eq!(
            get_session_query_type("show session history"),
            SessionQueryType::History
        );
        assert_eq!(
            get_session_query_type("current session stats"),
            SessionQueryType::Current
        );
        assert_eq!(
            get_session_query_type("session summary"),
            SessionQueryType::Summary
        );
    }

    #[test]
    fn test_empty_history() {
        let history = SessionHistory::default();
        let output = format_session_history(&history);
        assert!(output.contains("No previous sessions"));
    }
}
