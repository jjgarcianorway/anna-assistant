//! Formatting utilities for activity summaries

use super::summary::UserActivitySummary;

/// Format user activity summary as full display
pub fn format_activity_summary(summary: &UserActivitySummary) -> String {
    let mut lines = Vec::new();

    lines.push("=== User Activity Summary ===".to_string());
    lines.push(String::new());

    // Overview
    lines.push(format!("Total Interactions: {}", summary.total_interactions));
    lines.push(format!("Days Active: {}", summary.days_active()));
    lines.push(format!("Avg/Day: {:.1}", summary.avg_interactions_per_day()));

    if summary.current_streak > 0 {
        lines.push(format!("Current Streak: {} days", summary.current_streak));
    }
    if summary.best_streak > 0 {
        lines.push(format!("Best Streak: {} days", summary.best_streak));
    }

    lines.push(String::new());

    // Patterns
    lines.push("--- Activity Patterns ---".to_string());

    if let Some((time, count)) = summary.most_active_time() {
        lines.push(format!("Most Active Time: {} ({} interactions)", time, count));
    }

    if let Some((day, count)) = summary.most_active_day() {
        lines.push(format!("Most Active Day: {} ({} interactions)", day, count));
    }

    if let Some((topic, count)) = summary.top_topic() {
        lines.push(format!("Top Topic: {} ({} times)", topic, count));
    }

    // By time of day
    if !summary.by_time_of_day.is_empty() {
        lines.push(String::new());
        lines.push("--- By Time of Day ---".to_string());
        for (time, count) in &summary.by_time_of_day {
            let percent = (*count as f64 / summary.total_interactions as f64) * 100.0;
            lines.push(format!("  {}: {} ({:.0}%)", time, *count, percent));
        }
    }

    // By day of week
    if !summary.by_day_of_week.is_empty() {
        lines.push(String::new());
        lines.push("--- By Day of Week ---".to_string());
        for (day, count) in &summary.by_day_of_week {
            let percent = (*count as f64 / summary.total_interactions as f64) * 100.0;
            lines.push(format!("  {}: {} ({:.0}%)", day, *count, percent));
        }
    }

    lines.join("\n")
}

/// Format user activity summary compact
pub fn format_activity_summary_compact(summary: &UserActivitySummary) -> String {
    let mut parts = Vec::new();

    parts.push(format!("{}i", summary.total_interactions));
    parts.push(format!("{}d", summary.days_active()));

    if let Some((time, _)) = summary.most_active_time() {
        parts.push(format!("peak: {}", time));
    }

    if summary.current_streak > 0 {
        parts.push(format!("streak: {}", summary.current_streak));
    }

    parts.join(" | ")
}

/// Format user activity summary one-line
pub fn format_activity_summary_oneline(summary: &UserActivitySummary) -> String {
    format!(
        "Activity: {} interactions over {} days ({:.1}/day)",
        summary.total_interactions,
        summary.days_active(),
        summary.avg_interactions_per_day()
    )
}

/// Generate an activity insight
pub fn activity_insight(summary: &UserActivitySummary) -> Option<String> {
    if summary.total_interactions == 0 {
        return None;
    }

    // Check for patterns
    if let Some((time, count)) = summary.most_active_time() {
        let percent = (count as f64 / summary.total_interactions as f64) * 100.0;
        if percent > 50.0 {
            return Some(format!(
                "You're a {} person! Over {}% of your interactions happen then.",
                time.to_lowercase(),
                percent as u32
            ));
        }
    }

    if let Some((day, count)) = summary.most_active_day() {
        let percent = (count as f64 / summary.total_interactions as f64) * 100.0;
        if percent > 30.0 {
            return Some(format!(
                "{}s are your peak day, accounting for {}% of activity.",
                day,
                percent as u32
            ));
        }
    }

    if summary.current_streak > 7 {
        return Some(format!(
            "Great consistency! You're on a {}-day streak.",
            summary.current_streak
        ));
    }

    Some(format!(
        "You've interacted with Anna {} times across {} days.",
        summary.total_interactions,
        summary.days_active()
    ))
}
