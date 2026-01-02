//! Uptime formatting utilities.

use super::tracker::UptimeTracker;

/// Format duration in seconds to human readable
pub fn format_duration_secs(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let s = secs % 60;
        format!("{}m {}s", mins, s)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Format uptime stats for display
pub fn format_uptime(tracker: &UptimeTracker, now: u64) -> String {
    let mut output = String::new();

    output.push_str("Uptime Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    output.push_str(&format!(
        "Installed: {} days ago\n",
        tracker.days_since_install(now)
    ));
    output.push_str(&format!(
        "Status: {}\n\n",
        if tracker.is_running() { "Running" } else { "Stopped" }
    ));

    if let Some(duration) = tracker.current_session_duration(now) {
        output.push_str(&format!(
            "Current Session: {}\n\n",
            format_duration_secs(duration)
        ));
    }

    output.push_str(&format!(
        "Total Uptime:    {}\n",
        format_duration_secs(tracker.total_uptime_secs)
    ));
    output.push_str(&format!(
        "Sessions:        {}\n",
        tracker.session_count
    ));
    output.push_str(&format!(
        "Avg Session:     {}\n",
        format_duration_secs(tracker.avg_session_duration() as u64)
    ));
    output.push_str(&format!(
        "Uptime Rate:     {:.1}%\n",
        tracker.uptime_percentage(now)
    ));
    output.push_str(&format!(
        "Clean Shutdowns: {:.1}%\n",
        tracker.clean_shutdown_rate()
    ));

    if tracker.longest_session_secs > 0 {
        output.push_str(&format!(
            "\nLongest Session: {}\n",
            format_duration_secs(tracker.longest_session_secs)
        ));
    }

    output
}

/// Format compact uptime info
pub fn format_uptime_compact(tracker: &UptimeTracker, now: u64) -> String {
    let status = if tracker.is_running() { "up" } else { "down" };

    format!(
        "{} for {}, {} sessions, {:.0}% uptime",
        status,
        format_duration_secs(tracker.current_session_duration(now).unwrap_or(0)),
        tracker.session_count,
        tracker.uptime_percentage(now)
    )
}

/// Format uptime as one-liner
pub fn format_uptime_oneline(tracker: &UptimeTracker, now: u64) -> String {
    if tracker.is_running() {
        format!(
            "Up {} | {} total",
            format_duration_secs(tracker.current_session_duration(now).unwrap_or(0)),
            format_duration_secs(tracker.total_uptime_secs)
        )
    } else {
        format!("Down | {} total uptime", format_duration_secs(tracker.total_uptime_secs))
    }
}

/// Generate fun fact about uptime
pub fn uptime_fun_fact(tracker: &UptimeTracker, now: u64) -> Option<String> {
    if tracker.session_count < 2 {
        return None;
    }

    let days = tracker.days_since_install(now);
    let facts = vec![
        format!(
            "Anna has been installed for {} days - that's {} weeks!",
            days,
            days / 7
        ),
        format!(
            "Total uptime is {} - Anna is dedicated!",
            format_duration_secs(tracker.total_uptime_secs)
        ),
        format!(
            "{:.0}% clean shutdowns - {}!",
            tracker.clean_shutdown_rate(),
            if tracker.clean_shutdown_rate() > 90.0 {
                "very reliable"
            } else {
                "room for improvement"
            }
        ),
        format!(
            "Average session lasts {} - {}!",
            format_duration_secs(tracker.avg_session_duration() as u64),
            if tracker.avg_session_duration() > 3600.0 {
                "marathon sessions"
            } else {
                "quick check-ins"
            }
        ),
    ];

    let index = (tracker.session_count as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about uptime
pub fn is_uptime_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "uptime",
        "how long",
        "running for",
        "installed",
        "since install",
        "session",
        "availability",
    ];

    patterns.iter().any(|p| lower.contains(p))
}
