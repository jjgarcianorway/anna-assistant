//! Session history utility functions

use crate::session_history::tracker::SessionHistoryTracker;

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
    use crate::session_history::types::SessionType;

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
