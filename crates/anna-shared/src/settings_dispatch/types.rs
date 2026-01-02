// v0.0.714: Settings Dispatch Types (Phase 290)
// Core types and enums for settings dispatch

use serde::{Deserialize, Serialize};

/// Dispatch type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DispatchType {
    /// Immediate dispatch
    #[default]
    Immediate,
    /// Scheduled dispatch
    Scheduled,
    /// Batch dispatch
    Batch,
    /// Conditional dispatch
    Conditional,
}

impl std::fmt::Display for DispatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "immediate"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Batch => write!(f, "batch"),
            Self::Conditional => write!(f, "conditional"),
        }
    }
}

/// Dispatch status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DispatchStatus {
    /// Pending
    #[default]
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

impl std::fmt::Display for DispatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_type_display() {
        assert_eq!(format!("{}", DispatchType::Immediate), "immediate");
        assert_eq!(format!("{}", DispatchType::Batch), "batch");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DispatchStatus::Pending), "pending");
        assert_eq!(format!("{}", DispatchStatus::Completed), "completed");
    }
}
