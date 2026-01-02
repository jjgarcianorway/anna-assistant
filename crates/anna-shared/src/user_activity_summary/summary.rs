//! User activity summary tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::activity_record::ActivityRecord;
use super::time_types::{DayOfWeek, TimeOfDay};

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
