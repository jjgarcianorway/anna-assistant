// v0.0.568: Settings Scheduler (Phase 144)
// Schedule settings changes for specific times or conditions

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Schedule trigger type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleTrigger {
    /// At a specific time
    AtTime(chrono::DateTime<chrono::Utc>),
    /// After a duration from now
    AfterDuration(chrono::Duration),
    /// Daily at a specific hour (0-23)
    DailyAt { hour: u8, minute: u8 },
    /// On specific weekdays
    Weekly { days: Vec<chrono::Weekday>, hour: u8, minute: u8 },
    /// On system event
    OnEvent(ScheduleEvent),
}

impl std::fmt::Display for ScheduleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtTime(t) => write!(f, "At {}", t.format("%Y-%m-%d %H:%M")),
            Self::AfterDuration(d) => write!(f, "After {} seconds", d.num_seconds()),
            Self::DailyAt { hour, minute } => write!(f, "Daily at {:02}:{:02}", hour, minute),
            Self::Weekly { days, hour, minute } => {
                let day_names: Vec<_> = days.iter().map(|d| format!("{:?}", d)).collect();
                write!(f, "{} at {:02}:{:02}", day_names.join(", "), hour, minute)
            }
            Self::OnEvent(e) => write!(f, "On {}", e),
        }
    }
}

/// System events that can trigger schedule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleEvent {
    /// System startup
    Startup,
    /// System shutdown
    Shutdown,
    /// Network connected
    NetworkConnected,
    /// Network disconnected
    NetworkDisconnected,
    /// Battery low
    BatteryLow,
    /// Battery charging
    BatteryCharging,
}

impl std::fmt::Display for ScheduleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Startup => write!(f, "Startup"),
            Self::Shutdown => write!(f, "Shutdown"),
            Self::NetworkConnected => write!(f, "Network Connected"),
            Self::NetworkDisconnected => write!(f, "Network Disconnected"),
            Self::BatteryLow => write!(f, "Battery Low"),
            Self::BatteryCharging => write!(f, "Battery Charging"),
        }
    }
}

/// Scheduled action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduledAction {
    /// Switch to a profile
    SwitchProfile(String),
    /// Apply specific settings
    ApplySettings(Box<UnifiedSettings>),
    /// Change single setting
    ChangeSetting { category: SettingsCategory, field: String, value: String },
    /// Reset category to defaults
    ResetCategory(SettingsCategory),
    /// Enable/disable sync
    SetSyncEnabled(bool),
}

impl std::fmt::Display for ScheduledAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SwitchProfile(p) => write!(f, "Switch to profile '{}'", p),
            Self::ApplySettings(_) => write!(f, "Apply settings"),
            Self::ChangeSetting { category, field, value } => {
                write!(f, "Set {}.{} = {}", category, field, value)
            }
            Self::ResetCategory(c) => write!(f, "Reset {} to defaults", c),
            Self::SetSyncEnabled(e) => write!(f, "Set sync enabled = {}", e),
        }
    }
}

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

/// Settings scheduler
#[derive(Debug, Clone, Default)]
pub struct SettingsScheduler {
    /// All scheduled changes
    schedules: Vec<ScheduledChange>,
    /// Next ID
    next_id: u64,
}

impl SettingsScheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a schedule
    pub fn add(&mut self, name: &str, trigger: ScheduleTrigger, action: ScheduledAction) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let schedule = ScheduledChange::new(id, name, trigger, action);
        self.schedules.push(schedule);
        id
    }

    /// Add one-time schedule
    pub fn add_once(&mut self, name: &str, trigger: ScheduleTrigger, action: ScheduledAction) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let schedule = ScheduledChange::new(id, name, trigger, action).one_time();
        self.schedules.push(schedule);
        id
    }

    /// Remove a schedule
    pub fn remove(&mut self, id: u64) -> Option<ScheduledChange> {
        if let Some(pos) = self.schedules.iter().position(|s| s.id == id) {
            Some(self.schedules.remove(pos))
        } else {
            None
        }
    }

    /// Get schedule by ID
    pub fn get(&self, id: u64) -> Option<&ScheduledChange> {
        self.schedules.iter().find(|s| s.id == id)
    }

    /// Get mutable schedule
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ScheduledChange> {
        self.schedules.iter_mut().find(|s| s.id == id)
    }

    /// List all schedules
    pub fn list(&self) -> &[ScheduledChange] {
        &self.schedules
    }

    /// Get enabled schedules
    pub fn enabled(&self) -> Vec<&ScheduledChange> {
        self.schedules.iter().filter(|s| s.enabled).collect()
    }

    /// Get pending schedules (should run now)
    pub fn pending(&self) -> Vec<&ScheduledChange> {
        let now = chrono::Utc::now();
        self.schedules.iter().filter(|s| s.should_run(now)).collect()
    }

    /// Enable/disable schedule
    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(s) = self.schedules.iter_mut().find(|s| s.id == id) {
            s.set_enabled(enabled);
            return true;
        }
        false
    }

    /// Handle event trigger
    pub fn on_event(&self, event: ScheduleEvent) -> Vec<&ScheduledChange> {
        self.schedules
            .iter()
            .filter(|s| {
                s.enabled && matches!(&s.trigger, ScheduleTrigger::OnEvent(e) if *e == event)
            })
            .collect()
    }

    /// Count schedules
    pub fn count(&self) -> usize {
        self.schedules.len()
    }
}

/// Format schedules for display
pub fn format_schedules(scheduler: &SettingsScheduler) -> String {
    let mut output = String::new();

    output.push_str("=== Scheduled Settings ===\n\n");

    if scheduler.count() == 0 {
        output.push_str("No scheduled changes.\n");
        return output;
    }

    for s in scheduler.list() {
        let status = if s.enabled { "enabled" } else { "disabled" };
        output.push_str(&format!(
            "• {} [{}]\n  Trigger: {}\n  Action: {}\n  Runs: {}\n\n",
            s.name, status, s.trigger, s.action, s.run_count
        ));
    }

    output
}

/// Check if query is about scheduling
pub fn is_schedule_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("schedule")
        || lower.contains("at time")
        || lower.contains("every day")
        || lower.contains("automate settings")
}

/// Fun fact about scheduling
pub fn scheduler_fun_fact() -> &'static str {
    "You can schedule Anna to automatically switch settings at specific times!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_display() {
        let t = ScheduleTrigger::DailyAt { hour: 9, minute: 30 };
        assert_eq!(format!("{}", t), "Daily at 09:30");
    }

    #[test]
    fn test_event_display() {
        assert_eq!(format!("{}", ScheduleEvent::Startup), "Startup");
    }

    #[test]
    fn test_action_display() {
        let a = ScheduledAction::SwitchProfile("work".to_string());
        assert!(format!("{}", a).contains("work"));
    }

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

    #[test]
    fn test_scheduler_new() {
        let scheduler = SettingsScheduler::new();
        assert_eq!(scheduler.count(), 0);
    }

    #[test]
    fn test_scheduler_add() {
        let mut scheduler = SettingsScheduler::new();
        let id = scheduler.add(
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        );
        assert_eq!(id, 0);
        assert_eq!(scheduler.count(), 1);
    }

    #[test]
    fn test_scheduler_remove() {
        let mut scheduler = SettingsScheduler::new();
        let id = scheduler.add(
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        );
        assert!(scheduler.remove(id).is_some());
        assert_eq!(scheduler.count(), 0);
    }

    #[test]
    fn test_scheduler_enable_disable() {
        let mut scheduler = SettingsScheduler::new();
        let id = scheduler.add(
            "Test",
            ScheduleTrigger::OnEvent(ScheduleEvent::Startup),
            ScheduledAction::SetSyncEnabled(true),
        );
        scheduler.set_enabled(id, false);
        assert!(!scheduler.get(id).unwrap().enabled);
    }

    #[test]
    fn test_format_schedules() {
        let scheduler = SettingsScheduler::new();
        let output = format_schedules(&scheduler);
        assert!(output.contains("Scheduled"));
    }

    #[test]
    fn test_is_schedule_query() {
        assert!(is_schedule_query("schedule settings change"));
        assert!(is_schedule_query("every day at 9am"));
        assert!(!is_schedule_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = scheduler_fun_fact();
        assert!(fact.contains("schedule"));
    }
}
