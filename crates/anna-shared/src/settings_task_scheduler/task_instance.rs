// v0.0.610: Settings Task Scheduler - Task Instance (Phase 186)
// Task instance structure and lifecycle management

use serde::{Deserialize, Serialize};

use super::types::TaskState;

/// Task instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    /// Instance ID
    pub instance_id: String,
    /// Definition ID
    pub definition_id: String,
    /// State
    pub state: TaskState,
    /// Scheduled time
    pub scheduled_at: u64,
    /// Started time
    pub started_at: Option<u64>,
    /// Completed time
    pub completed_at: Option<u64>,
    /// Result message
    pub result: Option<String>,
}

impl TaskInstance {
    /// Create new instance
    pub fn new(instance_id: impl Into<String>, definition_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            definition_id: definition_id.into(),
            state: TaskState::Pending,
            scheduled_at: 0,
            started_at: None,
            completed_at: None,
            result: None,
        }
    }

    /// Set scheduled time
    pub fn scheduled_at(mut self, ts: u64) -> Self {
        self.scheduled_at = ts;
        self
    }

    /// Start task
    pub fn start(&mut self, ts: u64) {
        self.state = TaskState::Running;
        self.started_at = Some(ts);
    }

    /// Complete task
    pub fn complete(&mut self, ts: u64, result: impl Into<String>) {
        self.state = TaskState::Completed;
        self.completed_at = Some(ts);
        self.result = Some(result.into());
    }

    /// Fail task
    pub fn fail(&mut self, ts: u64, error: impl Into<String>) {
        self.state = TaskState::Failed;
        self.completed_at = Some(ts);
        self.result = Some(error.into());
    }

    /// Cancel task
    pub fn cancel(&mut self) {
        self.state = TaskState::Cancelled;
    }
}
