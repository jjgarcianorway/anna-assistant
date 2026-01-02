//! LLM Assignment Types
//!
//! Core types for tracking LLM model assignments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ModelTier {
    #[default]
    Light,
    Standard,
    Heavy,
    DeepThinking,
}

impl ModelTier {
    pub fn name(&self) -> &'static str {
        match self {
            ModelTier::Light => "Light",
            ModelTier::Standard => "Standard",
            ModelTier::Heavy => "Heavy",
            ModelTier::DeepThinking => "Deep Thinking",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ModelTier::Light => "L",
            ModelTier::Standard => "S",
            ModelTier::Heavy => "H",
            ModelTier::DeepThinking => "D",
        }
    }
}

/// Model assignment reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AssignmentReason {
    #[default]
    Default,
    HardwareLimit,
    UserPreference,
    TaskComplexity,
    PerformanceOptimization,
    Fallback,
}

impl AssignmentReason {
    pub fn name(&self) -> &'static str {
        match self {
            AssignmentReason::Default => "Default",
            AssignmentReason::HardwareLimit => "Hardware Limit",
            AssignmentReason::UserPreference => "User Preference",
            AssignmentReason::TaskComplexity => "Task Complexity",
            AssignmentReason::PerformanceOptimization => "Performance",
            AssignmentReason::Fallback => "Fallback",
        }
    }
}

/// An LLM assignment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAssignment {
    /// Specialist ID or role
    pub specialist_id: String,
    /// Model name
    pub model: String,
    /// Model tier
    pub tier: ModelTier,
    /// Why this model was assigned
    pub reason: AssignmentReason,
    /// When assigned
    pub assigned_at: u64,
    /// Is currently active
    pub active: bool,
    /// Parameters used (e.g., temperature)
    pub parameters: HashMap<String, String>,
}
