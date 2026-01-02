//! Strategic Thinking Tracker - Phase 91
//!
//! Tracks senior strategic thinking during idle time.
//! VISION.md: "Seniors can think strategically about improvements during idle time"
//! "If interrupted, Anna can resume later"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::task::ThinkingTask;
use super::types::{ThinkingCategory, ThinkingPriority, ThinkingStatus};

/// Strategic thinking tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategicThinkingTracker {
    /// All tasks
    pub tasks: Vec<ThinkingTask>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Total time spent thinking
    pub total_time_secs: u64,
    /// Total recommendations made
    pub total_recommendations: u64,
}

impl StrategicThinkingTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a thinking task
    pub fn add(&mut self, task: ThinkingTask) {
        *self.by_category.entry(task.category.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(task.status.name().to_string()).or_insert(0) += 1;
        self.tasks.push(task);
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&ThinkingTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Start a task
    pub fn start(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            self.update_status_count(&old_status, ThinkingStatus::InProgress);
            self.tasks[idx].status = ThinkingStatus::InProgress;
            self.tasks[idx].started_at = Some(timestamp);
            true
        } else {
            false
        }
    }

    /// Pause a task (interrupted)
    pub fn pause(&mut self, id: &str, resume_point: Option<String>, time_spent: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            self.update_status_count(&old_status, ThinkingStatus::Paused);
            self.tasks[idx].status = ThinkingStatus::Paused;
            self.tasks[idx].interrupted = true;
            self.tasks[idx].resume_point = resume_point;
            self.tasks[idx].time_spent_secs += time_spent;
            self.total_time_secs += time_spent;
            true
        } else {
            false
        }
    }

    /// Resume a paused task
    pub fn resume(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            if self.tasks[idx].status == ThinkingStatus::Paused {
                self.update_status_count(&ThinkingStatus::Paused, ThinkingStatus::InProgress);
                self.tasks[idx].status = ThinkingStatus::InProgress;
                self.tasks[idx].started_at = Some(timestamp);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Complete a task
    pub fn complete(&mut self, id: &str, findings: Vec<String>, recommendations: Vec<String>, time_spent: u64, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            let rec_count = recommendations.len() as u64;
            self.update_status_count(&old_status, ThinkingStatus::Completed);
            self.tasks[idx].status = ThinkingStatus::Completed;
            self.tasks[idx].completed_at = Some(timestamp);
            self.tasks[idx].time_spent_secs += time_spent;
            self.tasks[idx].findings = findings;
            self.tasks[idx].recommendations = recommendations;
            self.total_time_secs += time_spent;
            self.total_recommendations += rec_count;
            true
        } else {
            false
        }
    }

    fn update_status_count(&mut self, old: &ThinkingStatus, new: ThinkingStatus) {
        if let Some(count) = self.by_status.get_mut(old.name()) {
            *count = count.saturating_sub(1);
        }
        *self.by_status.entry(new.name().to_string()).or_insert(0) += 1;
    }

    /// Get pending tasks
    pub fn pending(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Pending).collect()
    }

    /// Get paused (resumable) tasks
    pub fn paused(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Paused).collect()
    }

    /// Get completed tasks
    pub fn completed(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Completed).collect()
    }

    /// Get tasks by category
    pub fn by_thinking_category(&self, category: ThinkingCategory) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.category == category).collect()
    }

    /// Get high priority tasks
    pub fn high_priority(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| matches!(t.priority, ThinkingPriority::High | ThinkingPriority::Critical)).collect()
    }

    /// Total task count
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Completed count
    pub fn completed_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Completed).count()
    }

    /// Average time per task
    pub fn avg_time_per_task(&self) -> f64 {
        let completed = self.completed_count();
        if completed == 0 {
            return 0.0;
        }
        self.total_time_secs as f64 / completed as f64
    }
}
