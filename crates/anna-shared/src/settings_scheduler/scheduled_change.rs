// v0.0.568: Settings Scheduler - Scheduled Change
// Individual scheduled change implementation

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

use super::actions::ScheduledAction;
use super::triggers::ScheduleTrigger;

/// A scheduled settings change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledChange {
    /// Unique ID
    pub id: u64,
    /// Name/description
    pub name: String,
    /// Trigger condition
    pub trigger: ScheduleTrigger,
    /// Action to perform
    pub action: ScheduledAction,
    /// Is enabled
    pub enabled: bool,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Last executed
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    /// Run count
    pub run_count: u32,
    /// One-time only
    pub one_time: bool,
}

impl ScheduledChange {
    /// Create new scheduled change
    pub fn new(
        id: u64,
        name: impl Into<String>,
        trigger: ScheduleTrigger,
        action: ScheduledAction,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            trigger,
            action,
            enabled: true,
            created: chrono::Utc::now(),
            last_run: None,
            run_count: 0,
            one_time: false,
        }
    }

    /// Set as one-time
    pub fn one_time(mut self) -> Self {
        self.one_time = true;
        self
    }

    /// Enable/disable
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Mark as executed
    pub fn mark_executed(&mut self) {
        self.last_run = Some(chrono::Utc::now());
        self.run_count += 1;
        if self.one_time {
            self.enabled = false;
        }
    }

    /// Check if should run now
    pub fn should_run(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.trigger {
            ScheduleTrigger::AtTime(t) => now >= *t && self.last_run.is_none(),
            ScheduleTrigger::AfterDuration(d) => {
                now >= self.created + *d && self.last_run.is_none()
            }
            ScheduleTrigger::DailyAt { hour, minute } => {
                let now_hour = now.hour() as u8;
                let now_minute = now.minute() as u8;
                if now_hour == *hour && now_minute == *minute {
                    // Check if already run today
                    if let Some(last) = self.last_run {
                        last.date_naive() != now.date_naive()
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            ScheduleTrigger::Weekly { days, hour, minute } => {
                let weekday = now.weekday();
                if !days.contains(&weekday) {
                    return false;
                }
                let now_hour = now.hour() as u8;
                let now_minute = now.minute() as u8;
                if now_hour == *hour && now_minute == *minute {
                    if let Some(last) = self.last_run {
                        last.date_naive() != now.date_naive()
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            ScheduleTrigger::OnEvent(_) => false, // Events are triggered externally
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_scheduler::triggers::ScheduleEvent;

    #[test]
    fn test_scheduled_change_new() {
        let s = ScheduledChange::new(
            1,
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        );
        assert_eq!(s.id, 1);
        assert!(s.enabled);
        assert_eq!(s.run_count, 0);
    }

    #[test]
    fn test_scheduled_change_one_time() {
        let s = ScheduledChange::new(
            1,
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        ).one_time();
        assert!(s.one_time);
    }

    #[test]
    fn test_scheduled_change_mark_executed() {
        let mut s = ScheduledChange::new(
            1,
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        );
        s.mark_executed();
        assert_eq!(s.run_count, 1);
        assert!(s.last_run.is_some());
    }
}
