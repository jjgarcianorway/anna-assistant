// v0.0.571: Settings Hooks Types
// Basic types and enums for settings hooks

use serde::{Deserialize, Serialize};

/// Hook trigger point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookTrigger {
    /// Before a change is applied
    BeforeChange,
    /// After a change is applied
    AfterChange,
    /// Before settings are loaded
    BeforeLoad,
    /// After settings are loaded
    AfterLoad,
    /// Before settings are saved
    BeforeSave,
    /// After settings are saved
    AfterSave,
    /// On validation
    OnValidate,
}

impl std::fmt::Display for HookTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeChange => write!(f, "Before Change"),
            Self::AfterChange => write!(f, "After Change"),
            Self::BeforeLoad => write!(f, "Before Load"),
            Self::AfterLoad => write!(f, "After Load"),
            Self::BeforeSave => write!(f, "Before Save"),
            Self::AfterSave => write!(f, "After Save"),
            Self::OnValidate => write!(f, "On Validate"),
        }
    }
}

/// Hook result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookResult {
    /// Continue with operation
    Continue,
    /// Skip the operation
    Skip,
    /// Abort the operation
    Abort,
    /// Modify and continue
    Modify,
}

impl std::fmt::Display for HookResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continue => write!(f, "Continue"),
            Self::Skip => write!(f, "Skip"),
            Self::Abort => write!(f, "Abort"),
            Self::Modify => write!(f, "Modify"),
        }
    }
}

/// Hook priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HookPriority {
    /// Run first
    High = 0,
    /// Run normally
    Normal = 50,
    /// Run last
    Low = 100,
}

impl Default for HookPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for HookPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Normal => write!(f, "Normal"),
            Self::Low => write!(f, "Low"),
        }
    }
}
