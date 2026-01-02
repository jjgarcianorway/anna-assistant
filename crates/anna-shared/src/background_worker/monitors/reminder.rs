//! Reminder system (v0.0.430).
//!
//! Reminders fire at scheduled times.

use crate::background_worker::job::BackgroundJob;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A user reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// Unique reminder ID
    pub id: String,
    /// What to remind about
    pub message: String,
    /// Schedule (cron-like or specific time)
    pub schedule: ReminderSchedule,
    /// Whether enabled
    pub enabled: bool,
    /// When created
    pub created_at: u64,
    /// Next trigger time
    pub next_trigger: Option<u64>,
    /// Last triggered
    pub last_triggered: Option<u64>,
}

/// Reminder schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReminderSchedule {
    /// One-time reminder at specific timestamp
    Once { at: u64 },
    /// Daily at specific hour:minute
    Daily { hour: u8, minute: u8 },
    /// Weekly on specific day and time (0=Sun, 1=Mon, etc.)
    Weekly { day: u8, hour: u8, minute: u8 },
    /// Monthly on specific day and time
    Monthly { day: u8, hour: u8, minute: u8 },
}

impl Reminder {
    /// Create a new reminder
    pub fn new(id: &str, message: &str, schedule: ReminderSchedule) -> Self {
        let mut reminder = Self {
            id: id.to_string(),
            message: message.to_string(),
            schedule,
            enabled: true,
            created_at: now_timestamp(),
            next_trigger: None,
            last_triggered: None,
        };
        reminder.calculate_next_trigger();
        reminder
    }

    /// Calculate next trigger time
    pub fn calculate_next_trigger(&mut self) {
        let now = now_timestamp();
        self.next_trigger = match &self.schedule {
            ReminderSchedule::Once { at } => {
                if *at > now {
                    Some(*at)
                } else {
                    None
                }
            }
            ReminderSchedule::Daily { hour, minute } => {
                Some(next_daily_time(now, *hour as u32, *minute as u32))
            }
            ReminderSchedule::Weekly { day, hour, minute } => {
                Some(next_weekly_time(now, *day, *hour as u32, *minute as u32))
            }
            ReminderSchedule::Monthly { day, hour, minute } => {
                Some(next_monthly_time(now, *day, *hour as u32, *minute as u32))
            }
        };
    }

    /// Check if reminder is due
    pub fn is_due(&self, now: u64) -> bool {
        self.enabled && self.next_trigger.map(|t| t <= now).unwrap_or(false)
    }

    /// Mark as triggered and calculate next
    pub fn trigger(&mut self) {
        self.last_triggered = Some(now_timestamp());
        self.calculate_next_trigger();
    }

    /// Create a background job for this reminder
    pub fn to_job(&self) -> BackgroundJob {
        BackgroundJob::reminder(&self.id)
    }
}

// Time calculation helpers

fn next_daily_time(now: u64, hour: u32, minute: u32) -> u64 {
    let secs_today = now % 86400;
    let target_secs = hour * 3600 + minute * 60;
    let day_start = now - secs_today;

    if secs_today < target_secs as u64 {
        day_start + target_secs as u64
    } else {
        day_start + 86400 + target_secs as u64
    }
}

fn next_weekly_time(now: u64, day: u8, hour: u32, minute: u32) -> u64 {
    // Simplified: just add 7 days from next daily
    next_daily_time(now, hour, minute) + (day as u64 * 86400)
}

fn next_monthly_time(now: u64, day: u8, hour: u32, minute: u32) -> u64 {
    // Simplified: assume ~30 days
    let daily = next_daily_time(now, hour, minute);
    daily + ((day.saturating_sub(1)) as u64 * 86400)
}

fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reminder_creation() {
        let reminder = Reminder::new(
            "weekly-report",
            "Generate weekly report",
            ReminderSchedule::Weekly {
                day: 1,
                hour: 9,
                minute: 0,
            },
        );
        assert!(reminder.next_trigger.is_some());
    }
}
