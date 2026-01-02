//! Task Priority Types

use serde::{Deserialize, Serialize};

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
    Critical,
}

impl TaskPriority {
    pub fn name(&self) -> &'static str {
        match self {
            TaskPriority::Low => "Low",
            TaskPriority::Normal => "Normal",
            TaskPriority::High => "High",
            TaskPriority::Urgent => "Urgent",
            TaskPriority::Critical => "Critical",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskPriority::Low => "▽",
            TaskPriority::Normal => "○",
            TaskPriority::High => "△",
            TaskPriority::Urgent => "◆",
            TaskPriority::Critical => "●",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            TaskPriority::Low => 1,
            TaskPriority::Normal => 2,
            TaskPriority::High => 3,
            TaskPriority::Urgent => 4,
            TaskPriority::Critical => 5,
        }
    }
}
