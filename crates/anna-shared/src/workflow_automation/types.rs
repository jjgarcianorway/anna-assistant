//! Workflow Automation Types
//!
//! Core types for workflow automation including triggers, statuses, steps, and records.

use serde::{Deserialize, Serialize};

/// Workflow trigger type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkflowTrigger {
    #[default]
    Manual,
    Scheduled,
    Event,
    Condition,
    Webhook,
}

impl WorkflowTrigger {
    pub fn name(&self) -> &'static str {
        match self {
            WorkflowTrigger::Manual => "Manual",
            WorkflowTrigger::Scheduled => "Scheduled",
            WorkflowTrigger::Event => "Event",
            WorkflowTrigger::Condition => "Condition",
            WorkflowTrigger::Webhook => "Webhook",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            WorkflowTrigger::Manual => "▶",
            WorkflowTrigger::Scheduled => "⏰",
            WorkflowTrigger::Event => "⚡",
            WorkflowTrigger::Condition => "?",
            WorkflowTrigger::Webhook => "↯",
        }
    }
}

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkflowStatus {
    #[default]
    Active,
    Paused,
    Running,
    Completed,
    Failed,
    Disabled,
}

impl WorkflowStatus {
    pub fn name(&self) -> &'static str {
        match self {
            WorkflowStatus::Active => "Active",
            WorkflowStatus::Paused => "Paused",
            WorkflowStatus::Running => "Running",
            WorkflowStatus::Completed => "Completed",
            WorkflowStatus::Failed => "Failed",
            WorkflowStatus::Disabled => "Disabled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            WorkflowStatus::Active => "✓",
            WorkflowStatus::Paused => "◐",
            WorkflowStatus::Running => "◉",
            WorkflowStatus::Completed => "●",
            WorkflowStatus::Failed => "✗",
            WorkflowStatus::Disabled => "-",
        }
    }
}

/// A workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Step action
    pub action: String,
    /// Step order
    pub order: u32,
    /// Completed
    pub completed: bool,
}

/// A workflow record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRecord {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: String,
    /// Trigger type
    pub trigger: WorkflowTrigger,
    /// Current status
    pub status: WorkflowStatus,
    /// Steps
    pub steps: Vec<WorkflowStep>,
    /// Run count
    pub run_count: u64,
    /// Success count
    pub success_count: u64,
    /// Created timestamp
    pub created_at: u64,
    /// Last run timestamp
    pub last_run: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_trigger() {
        assert_eq!(WorkflowTrigger::Scheduled.name(), "Scheduled");
        assert_eq!(WorkflowTrigger::Event.symbol(), "⚡");
    }

    #[test]
    fn test_workflow_status() {
        assert_eq!(WorkflowStatus::Running.name(), "Running");
        assert_eq!(WorkflowStatus::Failed.symbol(), "✗");
    }
}
