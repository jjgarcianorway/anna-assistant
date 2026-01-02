//! Query detection and fun facts for resolution times.

use super::formatting::format_duration_ms;
use super::stats::ResolutionTimeTracker;

/// Generate fun fact about resolution times
pub fn resolution_time_fun_fact(tracker: &ResolutionTimeTracker) -> Option<String> {
    if tracker.total_resolutions < 5 {
        return None;
    }

    let avg_secs = tracker.average_ms() / 1000.0;
    let fastest_ms = tracker.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0);
    let slowest_ms = tracker.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0);

    let facts = vec![
        format!(
            "Average resolution takes {:.1} seconds - {} making instant coffee!",
            avg_secs,
            if avg_secs < 30.0 { "faster than" } else { "slower than" }
        ),
        format!(
            "Fastest fix was {} - blink and you'd miss it!",
            format_duration_ms(fastest_ms)
        ),
        format!(
            "Longest resolution took {} - that was a tough one!",
            format_duration_ms(slowest_ms)
        ),
        format!(
            "Success rate is {:.1}% - {}!",
            tracker.success_rate(),
            if tracker.success_rate() > 90.0 {
                "excellent reliability"
            } else if tracker.success_rate() > 70.0 {
                "pretty good"
            } else {
                "room for improvement"
            }
        ),
    ];

    let index = (tracker.total_resolutions as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about resolution times
pub fn is_resolution_time_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "resolution time",
        "how long",
        "fastest resolution",
        "slowest resolution",
        "average time",
        "time to resolve",
        "response time",
    ];

    patterns.iter().any(|p| lower.contains(p))
}
