//! User-defined alarms and reminders (v0.0.456).
//!
//! Natural language alarm system:
//! - "Remind me every Monday at 9 about storage"
//! - "Alert me when disk is above 90%"
//! - "Notify me daily about failed services"
//!
//! v0.0.456: Initial implementation per VISION.md Phase 35.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Schedule for recurring alarms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlarmSchedule {
    /// Once at specific time
    Once { at: u64 },
    /// Daily at specific hour
    Daily { hour: u8, minute: u8 },
    /// Weekly on specific day
    Weekly { day: Weekday, hour: u8, minute: u8 },
    /// Monthly on specific day
    Monthly { day: u8, hour: u8, minute: u8 },
    /// When a condition is met
    Conditional { condition: AlarmCondition },
}

/// Days of the week
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "monday" | "mon" => Some(Self::Monday),
            "tuesday" | "tue" | "tues" => Some(Self::Tuesday),
            "wednesday" | "wed" => Some(Self::Wednesday),
            "thursday" | "thu" | "thurs" => Some(Self::Thursday),
            "friday" | "fri" => Some(Self::Friday),
            "saturday" | "sat" => Some(Self::Saturday),
            "sunday" | "sun" => Some(Self::Sunday),
            _ => None,
        }
    }

    pub fn to_num(&self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
}

/// Condition triggers for alarms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlarmCondition {
    /// Disk usage above threshold
    DiskAbove { threshold_percent: u8, path: Option<String> },
    /// Memory usage above threshold
    MemoryAbove { threshold_percent: u8 },
    /// Service in failed state
    ServiceFailed { service: String },
    /// Any service failed
    AnyServiceFailed,
    /// Custom probe result
    ProbeMatches { probe: String, pattern: String },
}

/// A user-defined alarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAlarm {
    /// Unique ID
    pub id: String,
    /// User-friendly name
    pub name: String,
    /// What to check/report
    pub topic: String,
    /// When to trigger
    pub schedule: AlarmSchedule,
    /// Notification channels to use
    pub channels: Vec<NotifyChannel>,
    /// Whether alarm is active
    pub enabled: bool,
    /// When created
    pub created_at: u64,
    /// Last triggered time
    pub last_triggered: Option<u64>,
    /// Number of times triggered
    pub trigger_count: u64,
}

/// Notification channel preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyChannel {
    Email,
    Desktop,
    Wall,
    Terminal,
}

impl UserAlarm {
    /// Create a new alarm
    pub fn new(name: &str, topic: &str, schedule: AlarmSchedule) -> Self {
        Self {
            id: generate_alarm_id(),
            name: name.to_string(),
            topic: topic.to_string(),
            schedule,
            channels: vec![NotifyChannel::Desktop],
            enabled: true,
            created_at: now_timestamp(),
            last_triggered: None,
            trigger_count: 0,
        }
    }

    /// Set notification channels
    pub fn with_channels(mut self, channels: Vec<NotifyChannel>) -> Self {
        self.channels = channels;
        self
    }

    /// Check if alarm should trigger now
    pub fn should_trigger(&self, now_ts: u64) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.schedule {
            AlarmSchedule::Once { at } => *at <= now_ts && self.last_triggered.is_none(),
            AlarmSchedule::Daily { hour, minute } => {
                self.is_time_match(now_ts, *hour, *minute) && !self.triggered_today(now_ts)
            }
            AlarmSchedule::Weekly { day, hour, minute } => {
                self.is_day_time_match(now_ts, *day, *hour, *minute)
                    && !self.triggered_this_week(now_ts, *day)
            }
            AlarmSchedule::Monthly { day, hour, minute } => {
                self.is_month_day_match(now_ts, *day, *hour, *minute)
                    && !self.triggered_this_month(now_ts)
            }
            AlarmSchedule::Conditional { .. } => {
                // Conditions are checked separately
                true
            }
        }
    }

    /// Mark as triggered
    pub fn mark_triggered(&mut self) {
        self.last_triggered = Some(now_timestamp());
        self.trigger_count += 1;
    }

    /// Check if time matches (within 5 minute window)
    fn is_time_match(&self, now_ts: u64, hour: u8, minute: u8) -> bool {
        let secs_today = now_ts % 86400;
        let current_mins = secs_today / 60;
        let target_mins = (hour as u64) * 60 + (minute as u64);
        current_mins >= target_mins && current_mins < target_mins + 5
    }

    /// Check if day and time match
    fn is_day_time_match(&self, now_ts: u64, day: Weekday, hour: u8, minute: u8) -> bool {
        let days_since_epoch = now_ts / 86400;
        // Jan 1, 1970 was Thursday (3)
        let current_day = ((days_since_epoch + 3) % 7) as u8;
        current_day == day.to_num() && self.is_time_match(now_ts, hour, minute)
    }

    /// Check if day of month and time match
    fn is_month_day_match(&self, _now_ts: u64, _day: u8, _hour: u8, _minute: u8) -> bool {
        // Simplified - use chrono for real implementation
        false
    }

    fn triggered_today(&self, now_ts: u64) -> bool {
        if let Some(last) = self.last_triggered {
            now_ts / 86400 == last / 86400
        } else {
            false
        }
    }

    fn triggered_this_week(&self, now_ts: u64, _target_day: Weekday) -> bool {
        if let Some(last) = self.last_triggered {
            (now_ts - last) < 7 * 86400
        } else {
            false
        }
    }

    fn triggered_this_month(&self, now_ts: u64) -> bool {
        if let Some(last) = self.last_triggered {
            (now_ts - last) < 30 * 86400
        } else {
            false
        }
    }
}

/// Storage for user alarms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlarmStore {
    pub alarms: HashMap<String, UserAlarm>,
}

impl AlarmStore {
    /// Load from disk
    pub fn load() -> Self {
        let path = Self::store_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Add an alarm
    pub fn add(&mut self, alarm: UserAlarm) {
        self.alarms.insert(alarm.id.clone(), alarm);
    }

    /// Remove an alarm
    pub fn remove(&mut self, id: &str) -> Option<UserAlarm> {
        self.alarms.remove(id)
    }

    /// Get alarm by ID
    pub fn get(&self, id: &str) -> Option<&UserAlarm> {
        self.alarms.get(id)
    }

    /// Get mutable alarm by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut UserAlarm> {
        self.alarms.get_mut(id)
    }

    /// List all alarms
    pub fn list(&self) -> Vec<&UserAlarm> {
        self.alarms.values().collect()
    }

    /// Get alarms that should trigger now
    pub fn due_alarms(&self, now_ts: u64) -> Vec<&UserAlarm> {
        self.alarms
            .values()
            .filter(|a| a.should_trigger(now_ts))
            .collect()
    }

    fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("alarms.json")
    }
}

/// Parse natural language alarm request
pub fn parse_alarm_request(input: &str) -> Option<UserAlarm> {
    let lower = input.to_lowercase();

    // Extract topic (after "about")
    let topic = if let Some(pos) = lower.find("about ") {
        input[pos + 6..].trim().to_string()
    } else {
        "system status".to_string()
    };

    // Try to parse schedule
    let schedule = if lower.contains("every monday") {
        Some(parse_weekly(&lower, Weekday::Monday))
    } else if lower.contains("every tuesday") {
        Some(parse_weekly(&lower, Weekday::Tuesday))
    } else if lower.contains("every wednesday") {
        Some(parse_weekly(&lower, Weekday::Wednesday))
    } else if lower.contains("every thursday") {
        Some(parse_weekly(&lower, Weekday::Thursday))
    } else if lower.contains("every friday") {
        Some(parse_weekly(&lower, Weekday::Friday))
    } else if lower.contains("every saturday") {
        Some(parse_weekly(&lower, Weekday::Saturday))
    } else if lower.contains("every sunday") {
        Some(parse_weekly(&lower, Weekday::Sunday))
    } else if lower.contains("daily") || lower.contains("every day") {
        Some(parse_daily(&lower))
    } else if lower.contains("disk") && (lower.contains("above") || lower.contains(">")) {
        Some(parse_disk_condition(&lower))
    } else if lower.contains("memory") && (lower.contains("above") || lower.contains(">")) {
        Some(parse_memory_condition(&lower))
    } else if lower.contains("service") && lower.contains("fail") {
        Some(parse_service_condition(&lower))
    } else {
        None
    };

    schedule.map(|s| UserAlarm::new(&format!("Alarm: {}", &topic), &topic, s))
}

fn parse_weekly(input: &str, day: Weekday) -> AlarmSchedule {
    let (hour, minute) = parse_time_from_input(input);
    AlarmSchedule::Weekly { day, hour, minute }
}

fn parse_daily(input: &str) -> AlarmSchedule {
    let (hour, minute) = parse_time_from_input(input);
    AlarmSchedule::Daily { hour, minute }
}

fn parse_time_from_input(input: &str) -> (u8, u8) {
    // Look for "at X" pattern
    if let Some(pos) = input.find("at ") {
        let after_at = &input[pos + 3..];
        // Try to parse time like "9", "9:00", "09:00"
        let time_part: String = after_at
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ':')
            .collect();

        if let Some(colon_pos) = time_part.find(':') {
            let hour: u8 = time_part[..colon_pos].parse().unwrap_or(9);
            let minute: u8 = time_part[colon_pos + 1..].parse().unwrap_or(0);
            return (hour, minute);
        } else if let Ok(hour) = time_part.parse::<u8>() {
            return (hour, 0);
        }
    }
    (9, 0) // Default to 9:00
}

fn parse_disk_condition(input: &str) -> AlarmSchedule {
    let threshold = extract_percent(input).unwrap_or(90);
    AlarmSchedule::Conditional {
        condition: AlarmCondition::DiskAbove {
            threshold_percent: threshold,
            path: None,
        },
    }
}

fn parse_memory_condition(input: &str) -> AlarmSchedule {
    let threshold = extract_percent(input).unwrap_or(90);
    AlarmSchedule::Conditional {
        condition: AlarmCondition::MemoryAbove {
            threshold_percent: threshold,
        },
    }
}

fn parse_service_condition(input: &str) -> AlarmSchedule {
    // Check if specific service mentioned
    if input.contains("any ") {
        AlarmSchedule::Conditional {
            condition: AlarmCondition::AnyServiceFailed,
        }
    } else {
        // Try to extract service name (after "service")
        let service = "".to_string();
        AlarmSchedule::Conditional {
            condition: AlarmCondition::ServiceFailed { service },
        }
    }
}

fn extract_percent(input: &str) -> Option<u8> {
    // Find patterns like "90%", "90 %", "> 90"
    let re_patterns = [
        r"(\d+)\s*%",
        r">\s*(\d+)",
        r"above\s*(\d+)",
    ];

    for pattern in re_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(input) {
                if let Some(num) = caps.get(1) {
                    if let Ok(n) = num.as_str().parse::<u8>() {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn generate_alarm_id() -> String {
    format!(
        "ALM-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_from_str() {
        assert_eq!(Weekday::from_str("monday"), Some(Weekday::Monday));
        assert_eq!(Weekday::from_str("Mon"), Some(Weekday::Monday));
        assert_eq!(Weekday::from_str("fri"), Some(Weekday::Friday));
        assert_eq!(Weekday::from_str("invalid"), None);
    }

    #[test]
    fn test_alarm_creation() {
        let alarm = UserAlarm::new(
            "Storage check",
            "disk usage",
            AlarmSchedule::Daily { hour: 9, minute: 0 },
        );
        assert!(alarm.enabled);
        assert!(alarm.id.starts_with("ALM-"));
    }

    #[test]
    fn test_parse_weekly_alarm() {
        let alarm = parse_alarm_request("remind me every monday at 9 about storage");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(a.schedule, AlarmSchedule::Weekly { day: Weekday::Monday, .. }));
    }

    #[test]
    fn test_parse_daily_alarm() {
        let alarm = parse_alarm_request("notify me daily at 10:30 about failed services");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(a.schedule, AlarmSchedule::Daily { hour: 10, minute: 30 }));
    }

    #[test]
    fn test_parse_disk_condition() {
        let alarm = parse_alarm_request("alert me when disk is above 90%");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(
            a.schedule,
            AlarmSchedule::Conditional { condition: AlarmCondition::DiskAbove { threshold_percent: 90, .. } }
        ));
    }

    #[test]
    fn test_time_parsing() {
        assert_eq!(parse_time_from_input("at 9"), (9, 0));
        assert_eq!(parse_time_from_input("at 9:30"), (9, 30));
        assert_eq!(parse_time_from_input("at 14:00"), (14, 0));
    }
}
