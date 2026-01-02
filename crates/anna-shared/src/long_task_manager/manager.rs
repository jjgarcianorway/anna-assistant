// v0.0.534: Long Task Manager - Manager (Phase 110)
// Manages collection of long-running tasks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::long_task_manager::types::{LongTaskStatus, LongTaskType};
use crate::long_task_manager::record::LongTaskRecord;

/// Long task manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LongTaskManager {
    tasks: HashMap<String, LongTaskRecord>,
    next_id: u32,
    idle_threshold_minutes: u32,
}

impl LongTaskManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            idle_threshold_minutes: 5,
        }
    }

    /// Set idle threshold
    pub fn set_idle_threshold(&mut self, minutes: u32) {
        self.idle_threshold_minutes = minutes;
    }

    /// Create a long task
    pub fn create(
        &mut self,
        task_type: LongTaskType,
        description: &str,
        timestamp: &str,
    ) -> String {
        let id = format!("LTASK-{:05}", self.next_id);
        self.next_id += 1;

        let task = LongTaskRecord::new(&id, task_type, description, timestamp);
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&LongTaskRecord> {
        self.tasks.get(id)
    }

    /// Get mutable task
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LongTaskRecord> {
        self.tasks.get_mut(id)
    }

    /// Get active tasks
    pub fn active(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.is_active()).collect()
    }

    /// Get tasks waiting for idle
    pub fn waiting_for_idle(&self) -> Vec<&LongTaskRecord> {
        self.tasks
            .values()
            .filter(|t| t.status == LongTaskStatus::WaitingIdle)
            .collect()
    }

    /// Get tasks needing email
    pub fn pending_emails(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.needs_email()).collect()
    }

    /// Get tasks by status
    pub fn by_status(&self, status: LongTaskStatus) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.status == status).collect()
    }

    /// Get tasks by type
    pub fn by_type(&self, task_type: &LongTaskType) -> Vec<&LongTaskRecord> {
        self.tasks
            .values()
            .filter(|t| &t.task_type == task_type)
            .collect()
    }

    /// Status statistics
    pub fn status_stats(&self) -> HashMap<LongTaskStatus, usize> {
        let mut stats = HashMap::new();
        for t in self.tasks.values() {
            *stats.entry(t.status).or_insert(0) += 1;
        }
        stats
    }

    /// Total tasks
    pub fn total(&self) -> usize {
        self.tasks.len()
    }

    /// All tasks
    pub fn all(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().collect()
    }
}
