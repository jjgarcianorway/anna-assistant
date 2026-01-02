//! Utility functions for idle time detection

use super::tracker::IdleTimeTracker;

/// Check if query is about idle time
pub fn is_idle_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "idle time",
        "when idle",
        "machine idle",
        "system idle",
        "background work",
        "background task",
        "idle period",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about idle time
pub fn idle_fun_fact(tracker: &IdleTimeTracker) -> String {
    if tracker.periods.is_empty() {
        return "No idle periods recorded yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has tracked {} idle periods.",
            tracker.period_count()
        ),
        format!(
            "Total idle time: {} minutes.",
            tracker.total_idle_secs / 60
        ),
        format!(
            "{} background tasks completed during idle.",
            tracker.tasks_completed
        ),
        format!(
            "Average idle period: {:.1} minutes.",
            tracker.avg_idle_duration() / 60.0
        ),
        {
            if let Some(longest) = tracker.longest_idle() {
                format!("Longest idle period: {} minutes.", longest / 60)
            } else {
                "No completed idle periods yet.".to_string()
            }
        },
    ];

    facts[tracker.period_count() % facts.len()].clone()
}
