//! Task Priority Manager Implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::priority::TaskPriority;
use super::state::TaskState;
use super::task::ManagedTask;

/// Task priority manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskPriorityManager {
    /// All tasks
    pub tasks: Vec<ManagedTask>,
    /// Count by priority
    pub by_priority: HashMap<String, u64>,
    /// Count by state
    pub by_state: HashMap<String, u64>,
    /// Total completed
    pub total_completed: u64,
    /// Total cancelled
    pub total_cancelled: u64,
}

impl TaskPriorityManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task
    pub fn add(&mut self, id: String, description: String, priority: TaskPriority, timestamp: u64) {
        let task = ManagedTask {
            id,
            description,
            priority,
            state: TaskState::Pending,
            created_at: timestamp,
            started_at: None,
            completed_at: None,
            blocked_reason: None,
        };
        *self.by_priority.entry(priority.name().to_string()).or_insert(0) += 1;
        *self.by_state.entry(TaskState::Pending.name().to_string()).or_insert(0) += 1;
        self.tasks.push(task);
    }

    /// Start a task
    pub fn start(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::InProgress.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::InProgress;
            self.tasks[idx].started_at = Some(timestamp);
            true
        } else {
            false
        }
    }

    /// Complete a task
    pub fn complete(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Completed.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Completed;
            self.tasks[idx].completed_at = Some(timestamp);
            self.total_completed += 1;
            true
        } else {
            false
        }
    }

    /// Block a task
    pub fn block(&mut self, id: &str, reason: &str) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Blocked.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Blocked;
            self.tasks[idx].blocked_reason = Some(reason.to_string());
            true
        } else {
            false
        }
    }

    /// Cancel a task
    pub fn cancel(&mut self, id: &str) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Cancelled.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Cancelled;
            self.total_cancelled += 1;
            true
        } else {
            false
        }
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&ManagedTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get next task (highest priority pending)
    pub fn next(&self) -> Option<&ManagedTask> {
        self.tasks
            .iter()
            .filter(|t| t.state == TaskState::Pending)
            .max_by_key(|t| t.priority.score())
    }

    /// Get pending tasks sorted by priority
    pub fn pending(&self) -> Vec<&ManagedTask> {
        let mut pending: Vec<&ManagedTask> =
            self.tasks.iter().filter(|t| t.state == TaskState::Pending).collect();
        pending.sort_by(|a, b| b.priority.cmp(&a.priority));
        pending
    }

    /// Get in-progress tasks
    pub fn in_progress(&self) -> Vec<&ManagedTask> {
        self.tasks.iter().filter(|t| t.state == TaskState::InProgress).collect()
    }

    /// Get blocked tasks
    pub fn blocked(&self) -> Vec<&ManagedTask> {
        self.tasks.iter().filter(|t| t.state == TaskState::Blocked).collect()
    }

    /// Total task count
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending().len()
    }
}
