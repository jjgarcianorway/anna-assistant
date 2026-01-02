// v0.0.568: Settings Scheduler - Scheduler Implementation
// Main scheduler for managing scheduled changes

use super::actions::ScheduledAction;
use super::scheduled_change::ScheduledChange;
use super::triggers::{ScheduleEvent, ScheduleTrigger};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
