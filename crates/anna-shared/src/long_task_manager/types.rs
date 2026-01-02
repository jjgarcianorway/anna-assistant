// v0.0.534: Long Task Manager - Types (Phase 110)
// Defines task status and type enumerations

use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LongTaskStatus {
    #[default]
    Queued,
    WaitingIdle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for LongTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::WaitingIdle => write!(f, "Waiting for Idle"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LongTaskType {
    Research,
    Installation,
    Backup,
    Download,
    Analysis,
    Compilation,
    Custom(String),
}

impl Default for LongTaskType {
    fn default() -> Self {
        Self::Research
    }
}

impl std::fmt::Display for LongTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Research => write!(f, "Research"),
            Self::Installation => write!(f, "Installation"),
            Self::Backup => write!(f, "Backup"),
            Self::Download => write!(f, "Download"),
            Self::Analysis => write!(f, "Analysis"),
            Self::Compilation => write!(f, "Compilation"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}
