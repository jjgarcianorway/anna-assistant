// v0.0.534: Long Task Manager - Task Record (Phase 110)
// Individual task record with lifecycle management

use serde::{Deserialize, Serialize};
use crate::long_task_manager::types::{LongTaskStatus, LongTaskType};

/// Individual long task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTaskRecord {
    pub id: String,
    pub task_type: LongTaskType,
    pub description: String,
    pub status: LongTaskStatus,
    pub ticket_id: Option<String>,
    pub progress_pct: u8,
    pub estimated_minutes: Option<u32>,
    pub email_on_complete: bool,
    pub user_email: Option<String>,
    pub chain_of_thought: Vec<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl LongTaskRecord {
    /// Create new long task
    pub fn new(id: &str, task_type: LongTaskType, description: &str, timestamp: &str) -> Self {
        Self {
            id: id.to_string(),
            task_type,
            description: description.to_string(),
            status: LongTaskStatus::Queued,
            ticket_id: None,
            progress_pct: 0,
            estimated_minutes: None,
            email_on_complete: false,
            user_email: None,
            chain_of_thought: Vec::new(),
            result: None,
            error: None,
            created_at: timestamp.to_string(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Enable email notification
    pub fn enable_email(&mut self, email: &str) {
        self.email_on_complete = true;
        self.user_email = Some(email.to_string());
    }

    /// Link to ticket
    pub fn link_ticket(&mut self, ticket_id: &str) {
        self.ticket_id = Some(ticket_id.to_string());
    }

    /// Set estimated time
    pub fn set_estimate(&mut self, minutes: u32) {
        self.estimated_minutes = Some(minutes);
    }

    /// Start task (wait for idle)
    pub fn wait_for_idle(&mut self) {
        self.status = LongTaskStatus::WaitingIdle;
    }

    /// Start running
    pub fn start(&mut self, timestamp: &str) {
        self.status = LongTaskStatus::Running;
        self.started_at = Some(timestamp.to_string());
    }

    /// Update progress
    pub fn update_progress(&mut self, pct: u8) {
        self.progress_pct = pct.min(100);
    }

    /// Add thought to chain
    pub fn add_thought(&mut self, thought: &str) {
        self.chain_of_thought.push(thought.to_string());
    }

    /// Pause task
    pub fn pause(&mut self) {
        self.status = LongTaskStatus::Paused;
    }

    /// Resume task
    pub fn resume(&mut self) {
        self.status = LongTaskStatus::Running;
    }

    /// Complete task
    pub fn complete(&mut self, result: &str, timestamp: &str) {
        self.status = LongTaskStatus::Completed;
        self.result = Some(result.to_string());
        self.completed_at = Some(timestamp.to_string());
        self.progress_pct = 100;
    }

    /// Fail task
    pub fn fail(&mut self, error: &str, timestamp: &str) {
        self.status = LongTaskStatus::Failed;
        self.error = Some(error.to_string());
        self.completed_at = Some(timestamp.to_string());
    }

    /// Cancel task
    pub fn cancel(&mut self) {
        self.status = LongTaskStatus::Cancelled;
    }

    /// Is task active?
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            LongTaskStatus::Queued | LongTaskStatus::WaitingIdle | LongTaskStatus::Running | LongTaskStatus::Paused
        )
    }

    /// Needs email notification?
    pub fn needs_email(&self) -> bool {
        self.email_on_complete
            && self.user_email.is_some()
            && matches!(self.status, LongTaskStatus::Completed | LongTaskStatus::Failed)
    }
}
