//! Task State Types

use serde::{Deserialize, Serialize};

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TaskState {
    #[default]
    Pending,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

impl TaskState {
    pub fn name(&self) -> &'static str {
        match self {
            TaskState::Pending => "Pending",
            TaskState::InProgress => "In Progress",
            TaskState::Blocked => "Blocked",
            TaskState::Completed => "Completed",
            TaskState::Cancelled => "Cancelled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskState::Pending => "○",
            TaskState::InProgress => "◐",
            TaskState::Blocked => "✗",
            TaskState::Completed => "✓",
            TaskState::Cancelled => "-",
        }
    }
}
