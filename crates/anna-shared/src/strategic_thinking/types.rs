//! Strategic Thinking Types - Phase 91
//!
//! Core type definitions for strategic thinking.

use serde::{Deserialize, Serialize};

/// Thinking status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingStatus {
    #[default]
    Pending,
    InProgress,
    Paused,
    Completed,
    Abandoned,
}

impl ThinkingStatus {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingStatus::Pending => "Pending",
            ThinkingStatus::InProgress => "In Progress",
            ThinkingStatus::Paused => "Paused",
            ThinkingStatus::Completed => "Completed",
            ThinkingStatus::Abandoned => "Abandoned",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ThinkingStatus::Pending => ".",
            ThinkingStatus::InProgress => "*",
            ThinkingStatus::Paused => "~",
            ThinkingStatus::Completed => "✓",
            ThinkingStatus::Abandoned => "x",
        }
    }
}

/// Category of strategic thinking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingCategory {
    #[default]
    Optimization,
    Security,
    Maintenance,
    Learning,
    UserExperience,
    SystemHealth,
    Performance,
}

impl ThinkingCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingCategory::Optimization => "Optimization",
            ThinkingCategory::Security => "Security",
            ThinkingCategory::Maintenance => "Maintenance",
            ThinkingCategory::Learning => "Learning",
            ThinkingCategory::UserExperience => "User Experience",
            ThinkingCategory::SystemHealth => "System Health",
            ThinkingCategory::Performance => "Performance",
        }
    }
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl ThinkingPriority {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingPriority::Low => "Low",
            ThinkingPriority::Medium => "Medium",
            ThinkingPriority::High => "High",
            ThinkingPriority::Critical => "Critical",
        }
    }
}
