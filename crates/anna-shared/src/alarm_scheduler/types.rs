//! Alarm types and data structures

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
