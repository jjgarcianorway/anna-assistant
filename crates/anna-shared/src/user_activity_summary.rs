//! User Activity Summary (Phase 73)
//!
//! Tracks and displays user interaction patterns, usage statistics,
//! and activity trends over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time of day bucket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// 6:00 - 12:00
    Morning,
    /// 12:00 - 18:00
    Afternoon,
    /// 18:00 - 22:00
    Evening,
    /// 22:00 - 6:00
    Night,
}

impl TimeOfDay {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Evening => "Evening",
            Self::Night => "Night",
        }
    }

    /// Determine time of day from hour (0-23)
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            6..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            18..=21 => Self::Evening,
            _ => Self::Night,
        }
    }
}

/// Day of week
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl DayOfWeek {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    /// Short display name
    pub fn short(&self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }

    /// From day index (0 = Monday)
    pub fn from_index(index: u8) -> Self {
        match index % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }
}

/// A single activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    /// Timestamp of activity
    pub timestamp: u64,
    /// Type of activity
    pub activity_type: String,
    /// Topic/category (if detected)
    pub topic: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
}

impl ActivityRecord {
    /// Create a new activity record
    pub fn new(activity_type: impl Into<String>, timestamp: u64) -> Self {
        Self {
            timestamp,
            activity_type: activity_type.into(),
            topic: None,
            session_id: None,
            duration_ms: None,
        }
    }
}

/// User activity tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// Total interactions
    pub total_interactions: u64,
    /// Interactions by time of day
    pub by_time_of_day: HashMap<String, u64>,
    /// Interactions by day of week
    pub by_day_of_week: HashMap<String, u64>,
    /// Interactions by topic
    pub by_topic: HashMap<String, u64>,
    /// Interactions by activity type
    pub by_type: HashMap<String, u64>,
    /// Current streak (consecutive days)
    pub current_streak: u32,
    /// Best streak
    pub best_streak: u32,
    /// Last activity timestamp
    pub last_activity: u64,
    /// First activity timestamp
    pub first_activity: u64,
    /// Total sessions
    pub session_count: u64,
    /// Recent activity records (last 50)
    pub recent: Vec<ActivityRecord>,
}

impl UserActivitySummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an activity
    pub fn record(&mut self, record: ActivityRecord, time_of_day: TimeOfDay, day_of_week: DayOfWeek) {
        self.total_interactions += 1;

        // Update by time of day
        *self.by_time_of_day.entry(time_of_day.display().to_string()).or_insert(0) += 1;

        // Update by day of week
        *self.by_day_of_week.entry(day_of_week.display().to_string()).or_insert(0) += 1;

        // Update by topic
        if let Some(ref topic) = record.topic {
            *self.by_topic.entry(topic.clone()).or_insert(0) += 1;
        }

        // Update by type
        *self.by_type.entry(record.activity_type.clone()).or_insert(0) += 1;

        // Update timestamps
        if self.first_activity == 0 || record.timestamp < self.first_activity {
            self.first_activity = record.timestamp;
        }
        self.last_activity = record.timestamp;

        // Add to recent
        self.recent.insert(0, record);
        if self.recent.len() > 50 {
            self.recent.truncate(50);
        }
    }

    /// Most active time of day
    pub fn most_active_time(&self) -> Option<(&String, u64)> {
        self.by_time_of_day.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Most active day of week
    pub fn most_active_day(&self) -> Option<(&String, u64)> {
        self.by_day_of_week.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Top topic
    pub fn top_topic(&self) -> Option<(&String, u64)> {
        self.by_topic.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Top activity type
    pub fn top_activity_type(&self) -> Option<(&String, u64)> {
        self.by_type.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Days active (approximation from first to last activity)
    pub fn days_active(&self) -> u64 {
        if self.first_activity == 0 || self.last_activity == 0 {
            return 0;
        }
        if self.last_activity <= self.first_activity {
            return 1;
        }
        ((self.last_activity - self.first_activity) / 86400) + 1
    }

    /// Average interactions per day
    pub fn avg_interactions_per_day(&self) -> f64 {
        let days = self.days_active();
        if days == 0 {
            return 0.0;
        }
        self.total_interactions as f64 / days as f64
    }
}

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

/// Check if query is asking about user activity
pub fn is_activity_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "my activity",
        "user activity",
        "usage patterns",
        "when do i use",
        "how often",
        "my usage",
        "activity summary",
        "usage stats",
        "when am i active",
        "interaction history",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Detect topic from query
pub fn detect_topic(query: &str) -> Option<String> {
    let q = query.to_lowercase();

    // Package install has highest priority if "install" is present
    if q.contains("install") || q.contains("package") || q.contains("pacman")
        || q.contains("apt") || q.contains("dnf") {
        return Some("package".to_string());
    }

    // Then check specific tools/technologies
    let topics = [
        ("docker", vec!["docker", "container", "compose", "kubernetes"]),
        ("git", vec!["git", "commit", "push", "branch", "merge"]),
        ("editor", vec!["vim", "nano", "emacs", "editor"]),
        ("network", vec!["network", "wifi", "ethernet", "ip", "dns"]),
        ("security", vec!["security", "firewall", "password", "ssh", "key"]),
        ("system", vec!["system", "boot", "kernel", "memory", "cpu"]),
        ("service", vec!["service", "systemd", "restart", "start", "stop"]),
    ];

    for (topic, keywords) in &topics {
        if keywords.iter().any(|kw| q.contains(kw)) {
            return Some(topic.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day_display() {
        assert_eq!(TimeOfDay::Morning.display(), "Morning");
        assert_eq!(TimeOfDay::Night.display(), "Night");
    }

    #[test]
    fn test_time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(2), TimeOfDay::Night);
    }

    #[test]
    fn test_day_of_week_display() {
        assert_eq!(DayOfWeek::Monday.display(), "Monday");
        assert_eq!(DayOfWeek::Monday.short(), "Mon");
    }

    #[test]
    fn test_day_of_week_from_index() {
        assert_eq!(DayOfWeek::from_index(0), DayOfWeek::Monday);
        assert_eq!(DayOfWeek::from_index(6), DayOfWeek::Sunday);
        assert_eq!(DayOfWeek::from_index(7), DayOfWeek::Monday); // Wraps
    }

    #[test]
    fn test_activity_record_new() {
        let record = ActivityRecord::new("query", 1000);
        assert_eq!(record.activity_type, "query");
        assert_eq!(record.timestamp, 1000);
    }

    #[test]
    fn test_user_activity_summary_record() {
        let mut summary = UserActivitySummary::new();
        let record = ActivityRecord::new("query", 1000);
        summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);

        assert_eq!(summary.total_interactions, 1);
        assert_eq!(summary.by_time_of_day.get("Morning"), Some(&1));
        assert_eq!(summary.by_day_of_week.get("Monday"), Some(&1));
    }

    #[test]
    fn test_user_activity_summary_most_active() {
        let mut summary = UserActivitySummary::new();

        // Record more morning activity
        for _ in 0..5 {
            let record = ActivityRecord::new("query", 1000);
            summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);
        }

        for _ in 0..2 {
            let record = ActivityRecord::new("query", 1000);
            summary.record(record, TimeOfDay::Afternoon, DayOfWeek::Tuesday);
        }

        let (time, count) = summary.most_active_time().unwrap();
        assert_eq!(time, "Morning");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_days_active() {
        let mut summary = UserActivitySummary::new();
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + (86400 * 5); // 5 days later

        assert_eq!(summary.days_active(), 6); // Inclusive
    }

    #[test]
    fn test_avg_interactions_per_day() {
        let mut summary = UserActivitySummary::new();
        summary.total_interactions = 100;
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + (86400 * 9); // 10 days

        assert!((summary.avg_interactions_per_day() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_format_activity_summary() {
        let mut summary = UserActivitySummary::new();
        let record = ActivityRecord::new("query", 1000);
        summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);

        let output = format_activity_summary(&summary);
        assert!(output.contains("User Activity"));
        assert!(output.contains("Morning"));
    }

    #[test]
    fn test_format_activity_summary_compact() {
        let mut summary = UserActivitySummary::new();
        summary.total_interactions = 42;
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + 86400;

        let output = format_activity_summary_compact(&summary);
        assert!(output.contains("42i"));
    }

    #[test]
    fn test_activity_insight() {
        let summary = UserActivitySummary::new();
        assert!(activity_insight(&summary).is_none());

        let mut summary2 = UserActivitySummary::new();
        summary2.total_interactions = 10;
        summary2.first_activity = 1000;
        summary2.last_activity = 2000;

        let insight = activity_insight(&summary2);
        assert!(insight.is_some());
    }

    #[test]
    fn test_is_activity_query() {
        assert!(is_activity_query("show my activity"));
        assert!(is_activity_query("what are my usage patterns?"));
        assert!(is_activity_query("how often do I use anna?"));
        assert!(!is_activity_query("how do I install vim?"));
    }

    #[test]
    fn test_detect_topic() {
        assert_eq!(detect_topic("install vim"), Some("package".to_string()));
        assert_eq!(detect_topic("restart docker"), Some("docker".to_string()));
        assert_eq!(detect_topic("git push"), Some("git".to_string()));
        assert_eq!(detect_topic("hello world"), None);
    }
}
