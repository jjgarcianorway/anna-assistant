//! Core types for user-defined alarms and reminders.

use serde::{Deserialize, Serialize};

use super::utils::{generate_alarm_id, now_timestamp};

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

/// Notification channel preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyChannel {
    Email,
    Desktop,
    Wall,
    Terminal,
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
