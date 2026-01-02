//! Utility functions for alarm scheduler

use super::types::{AlarmScheduler, DayOfWeek};

/// Parse day of week from string
pub fn parse_day_of_week(s: &str) -> Option<DayOfWeek> {
    let s = s.to_lowercase();
    match s.as_str() {
        "monday" | "mon" => Some(DayOfWeek::Monday),
        "tuesday" | "tue" => Some(DayOfWeek::Tuesday),
        "wednesday" | "wed" => Some(DayOfWeek::Wednesday),
        "thursday" | "thu" => Some(DayOfWeek::Thursday),
        "friday" | "fri" => Some(DayOfWeek::Friday),
        "saturday" | "sat" => Some(DayOfWeek::Saturday),
        "sunday" | "sun" => Some(DayOfWeek::Sunday),
        _ => None,
    }
}

/// Format alarm scheduler for display
pub fn format_alarm_scheduler(scheduler: &AlarmScheduler) -> String {
    let mut lines = vec!["=== Alarm Scheduler ===".to_string()];
    lines.push(String::new());

    if scheduler.alarms.is_empty() {
        lines.push("No alarms scheduled.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total alarms: {}", scheduler.total_count()));
    lines.push(format!("Active: {}", scheduler.active_count()));
    lines.push(format!("Total triggers: {}", scheduler.total_triggers));

    // Active alarms
    let active = scheduler.active();
    if !active.is_empty() {
        lines.push(String::new());
        lines.push("Active alarms:".to_string());
        for a in active.iter().take(5) {
            let day = a.day_of_week.map(|d| d.short()).unwrap_or("*");
            lines.push(format!(
                "  [{}] {} {:02}:{:02} - {}",
                a.frequency.name(),
                day,
                a.hour,
                a.minute,
                a.description
            ));
        }
    }

    lines.join("\n")
}

/// Format alarm scheduler compact
pub fn format_alarm_scheduler_compact(scheduler: &AlarmScheduler) -> String {
    format!(
        "Alarms: {} total | {} active | {} triggered",
        scheduler.total_count(),
        scheduler.active_count(),
        scheduler.total_triggers
    )
}

/// Format alarm scheduler one-line
pub fn format_alarm_scheduler_oneline(scheduler: &AlarmScheduler) -> String {
    format!(
        "{} alarms ({} active)",
        scheduler.total_count(),
        scheduler.active_count()
    )
}
