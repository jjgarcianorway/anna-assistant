//! Alarm Scheduler - Phase 90
//!
//! Schedules recurring notifications via natural language.
//! VISION.md: "User can set specific alarms via natural language"
//! Example: "Please notify me every Monday at 9 about storage progression"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alarm frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlarmFrequency {
    #[default]
    Once,
    Daily,
    Weekly,
    Monthly,
    Hourly,
    Custom,
}

impl AlarmFrequency {
    pub fn name(&self) -> &'static str {
        match self {
            AlarmFrequency::Once => "Once",
            AlarmFrequency::Daily => "Daily",
            AlarmFrequency::Weekly => "Weekly",
            AlarmFrequency::Monthly => "Monthly",
            AlarmFrequency::Hourly => "Hourly",
            AlarmFrequency::Custom => "Custom",
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
    pub fn name(&self) -> &'static str {
        match self {
            DayOfWeek::Monday => "Monday",
            DayOfWeek::Tuesday => "Tuesday",
            DayOfWeek::Wednesday => "Wednesday",
            DayOfWeek::Thursday => "Thursday",
            DayOfWeek::Friday => "Friday",
            DayOfWeek::Saturday => "Saturday",
            DayOfWeek::Sunday => "Sunday",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            DayOfWeek::Monday => "Mon",
            DayOfWeek::Tuesday => "Tue",
            DayOfWeek::Wednesday => "Wed",
            DayOfWeek::Thursday => "Thu",
            DayOfWeek::Friday => "Fri",
            DayOfWeek::Saturday => "Sat",
            DayOfWeek::Sunday => "Sun",
        }
    }
}

/// Alarm status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlarmStatus {
    #[default]
    Active,
    Paused,
    Triggered,
    Expired,
    Cancelled,
}

impl AlarmStatus {
    pub fn name(&self) -> &'static str {
        match self {
            AlarmStatus::Active => "Active",
            AlarmStatus::Paused => "Paused",
            AlarmStatus::Triggered => "Triggered",
            AlarmStatus::Expired => "Expired",
            AlarmStatus::Cancelled => "Cancelled",
        }
    }
}

/// An alarm record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmRecord {
    /// Unique ID
    pub id: String,
    /// Natural language description
    pub description: String,
    /// What to report on
    pub topic: String,
    /// Frequency
    pub frequency: AlarmFrequency,
    /// Day of week (for weekly)
    pub day_of_week: Option<DayOfWeek>,
    /// Hour (0-23)
    pub hour: u8,
    /// Minute (0-59)
    pub minute: u8,
    /// Status
    pub status: AlarmStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Last triggered
    pub last_triggered: Option<u64>,
    /// Next trigger time
    pub next_trigger: Option<u64>,
    /// Times triggered
    pub trigger_count: u32,
}

/// Alarm scheduler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlarmScheduler {
    /// All alarms
    pub alarms: Vec<AlarmRecord>,
    /// Count by frequency
    pub by_frequency: HashMap<String, u64>,
    /// Count by topic
    pub by_topic: HashMap<String, u64>,
    /// Total triggers
    pub total_triggers: u64,
}

impl AlarmScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an alarm
    pub fn add(&mut self, alarm: AlarmRecord) {
        *self.by_frequency.entry(alarm.frequency.name().to_string()).or_insert(0) += 1;
        *self.by_topic.entry(alarm.topic.clone()).or_insert(0) += 1;
        self.alarms.push(alarm);
    }

    /// Get alarm by ID
    pub fn get(&self, id: &str) -> Option<&AlarmRecord> {
        self.alarms.iter().find(|a| a.id == id)
    }

    /// Trigger an alarm
    pub fn trigger(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].last_triggered = Some(timestamp);
            self.alarms[idx].trigger_count += 1;
            self.alarms[idx].status = AlarmStatus::Triggered;
            self.total_triggers += 1;

            // Reset status to Active for recurring
            if self.alarms[idx].frequency != AlarmFrequency::Once {
                self.alarms[idx].status = AlarmStatus::Active;
            } else {
                self.alarms[idx].status = AlarmStatus::Expired;
            }
            true
        } else {
            false
        }
    }

    /// Pause an alarm
    pub fn pause(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Paused;
            true
        } else {
            false
        }
    }

    /// Resume an alarm
    pub fn resume(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Active;
            true
        } else {
            false
        }
    }

    /// Cancel an alarm
    pub fn cancel(&mut self, id: &str) -> bool {
        let found = self.alarms.iter().position(|a| a.id == id);
        if let Some(idx) = found {
            self.alarms[idx].status = AlarmStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Get active alarms
    pub fn active(&self) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.status == AlarmStatus::Active).collect()
    }

    /// Get alarms due at given time
    pub fn due_at(&self, hour: u8, minute: u8, day: Option<DayOfWeek>) -> Vec<&AlarmRecord> {
        self.alarms
            .iter()
            .filter(|a| {
                a.status == AlarmStatus::Active
                    && a.hour == hour
                    && a.minute == minute
                    && (a.day_of_week.is_none() || a.day_of_week == day)
            })
            .collect()
    }

    /// Get alarms by topic
    pub fn by_alarm_topic(&self, topic: &str) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.topic == topic).collect()
    }

    /// Get alarms by frequency
    pub fn by_alarm_frequency(&self, freq: AlarmFrequency) -> Vec<&AlarmRecord> {
        self.alarms.iter().filter(|a| a.frequency == freq).collect()
    }

    /// Total alarm count
    pub fn total_count(&self) -> usize {
        self.alarms.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.alarms.iter().filter(|a| a.status == AlarmStatus::Active).count()
    }

    /// Most common topic
    pub fn most_common_topic(&self) -> Option<(&str, u64)> {
        self.by_topic
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

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

/// Check if query is about alarms
pub fn is_alarm_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "alarm",
        "notify me",
        "remind me",
        "schedule notification",
        "every monday",
        "every day",
        "weekly report",
        "daily report",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about alarms
pub fn alarm_fun_fact(scheduler: &AlarmScheduler) -> String {
    if scheduler.alarms.is_empty() {
        return "No alarms scheduled yet!".to_string();
    }

    let facts = [
        format!("You have {} alarms scheduled.", scheduler.total_count()),
        format!("{} alarms are currently active.", scheduler.active_count()),
        format!("Alarms have triggered {} times.", scheduler.total_triggers),
        {
            if let Some((topic, count)) = scheduler.most_common_topic() {
                format!("Most common topic: {} ({} alarms)", topic, count)
            } else {
                "No topic stats yet.".to_string()
            }
        },
    ];

    facts[scheduler.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_alarm(topic: &str, freq: AlarmFrequency) -> AlarmRecord {
        AlarmRecord {
            id: format!("ALM-{}", topic),
            description: format!("Report on {}", topic),
            topic: topic.to_string(),
            frequency: freq,
            day_of_week: None,
            hour: 9,
            minute: 0,
            status: AlarmStatus::Active,
            created_at: 1234567890,
            last_triggered: None,
            next_trigger: None,
            trigger_count: 0,
        }
    }

    #[test]
    fn test_alarm_frequency() {
        assert_eq!(AlarmFrequency::Daily.name(), "Daily");
        assert_eq!(AlarmFrequency::Weekly.name(), "Weekly");
    }

    #[test]
    fn test_day_of_week() {
        assert_eq!(DayOfWeek::Monday.name(), "Monday");
        assert_eq!(DayOfWeek::Friday.short(), "Fri");
    }

    #[test]
    fn test_alarm_status() {
        assert_eq!(AlarmStatus::Active.name(), "Active");
        assert_eq!(AlarmStatus::Paused.name(), "Paused");
    }

    #[test]
    fn test_add_alarm() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        assert_eq!(scheduler.total_count(), 1);
        assert!(scheduler.get("ALM-storage").is_some());
    }

    #[test]
    fn test_trigger_alarm() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        assert!(scheduler.trigger("ALM-storage", 1234567890));
        assert_eq!(scheduler.total_triggers, 1);
        assert_eq!(scheduler.get("ALM-storage").unwrap().trigger_count, 1);
    }

    #[test]
    fn test_once_expires() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("backup", AlarmFrequency::Once));

        scheduler.trigger("ALM-backup", 1234567890);
        assert_eq!(scheduler.get("ALM-backup").unwrap().status, AlarmStatus::Expired);
    }

    #[test]
    fn test_pause_resume() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        assert!(scheduler.pause("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Paused);

        assert!(scheduler.resume("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Active);
    }

    #[test]
    fn test_cancel() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        assert!(scheduler.cancel("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Cancelled);
    }

    #[test]
    fn test_due_at() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        let due = scheduler.due_at(9, 0, None);
        assert_eq!(due.len(), 1);

        let not_due = scheduler.due_at(10, 0, None);
        assert_eq!(not_due.len(), 0);
    }

    #[test]
    fn test_parse_day_of_week() {
        assert_eq!(parse_day_of_week("monday"), Some(DayOfWeek::Monday));
        assert_eq!(parse_day_of_week("Mon"), Some(DayOfWeek::Monday));
        assert_eq!(parse_day_of_week("invalid"), None);
    }

    #[test]
    fn test_format_alarm_scheduler() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        let output = format_alarm_scheduler(&scheduler);
        assert!(output.contains("Alarm Scheduler"));
        assert!(output.contains("storage"));
    }

    #[test]
    fn test_is_alarm_query() {
        assert!(is_alarm_query("notify me every monday"));
        assert!(is_alarm_query("set an alarm"));
        assert!(is_alarm_query("daily report on storage"));
        assert!(!is_alarm_query("what is the weather?"));
    }

    #[test]
    fn test_alarm_fun_fact() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        let fact = alarm_fun_fact(&scheduler);
        assert!(!fact.is_empty());
    }
}
