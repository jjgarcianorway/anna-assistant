//! Session history formatting functions

use crate::session_history::tracker::SessionHistoryTracker;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_history::types::{SessionOutcome, SessionType};

    #[test]
    fn test_format_history() {
        let mut tracker = SessionHistoryTracker::new();
        tracker.start_session("sess1".to_string(), SessionType::Interactive, 1000);
        tracker.end_session("sess1", SessionOutcome::Completed, 2000);

        let output = format_session_history(&tracker);
        assert!(output.contains("Session History"));
        assert!(output.contains("Total sessions: 1"));
    }
}
