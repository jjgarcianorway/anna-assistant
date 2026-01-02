// v0.0.610: Settings Task Scheduler - Types (Phase 186)
// Task scheduling types and enums

use serde::{Deserialize, Serialize};

/// Task frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskFrequency {
    /// Run once
    #[default]
    Once,
    /// Every minute
    Minutely,
    /// Every hour
    Hourly,
    /// Every day
    Daily,
    /// Every week
    Weekly,
    /// Every month
    Monthly,
}

impl std::fmt::Display for TaskFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Once => write!(f, "once"),
            Self::Minutely => write!(f, "minutely"),
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Backup task
    Backup,
    /// Sync task
    Sync,
    /// Validation task
    Validation,
    /// Report task
    Report,
    /// Cleanup task
    Cleanup,
    /// Custom task
    Custom,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backup => write!(f, "backup"),
            Self::Sync => write!(f, "sync"),
            Self::Validation => write!(f, "validation"),
            Self::Report => write!(f, "report"),
            Self::Cleanup => write!(f, "cleanup"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskState {
    /// Pending
    #[default]
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}
