// v0.0.545: Escalation Policy Config - Types (Phase 121)
// Escalation trigger conditions, priorities, modes, and notification preferences

use serde::{Deserialize, Serialize};

/// Escalation trigger condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscalationTrigger {
    LowConfidence,
    HighRisk,
    SecurityRelated,
    MultiDepartment,
    UserRequest,
    TimeoutExceeded,
    RepeatedFailure,
    ComplexQuery,
}

impl std::fmt::Display for EscalationTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LowConfidence => write!(f, "Low Confidence"),
            Self::HighRisk => write!(f, "High Risk"),
            Self::SecurityRelated => write!(f, "Security Related"),
            Self::MultiDepartment => write!(f, "Multi-Department"),
            Self::UserRequest => write!(f, "User Request"),
            Self::TimeoutExceeded => write!(f, "Timeout Exceeded"),
            Self::RepeatedFailure => write!(f, "Repeated Failure"),
            Self::ComplexQuery => write!(f, "Complex Query"),
        }
    }
}

/// Escalation priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationPriority {
    #[default]
    Normal,
    High,
    Critical,
    Immediate,
}

impl std::fmt::Display for EscalationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
            Self::Immediate => write!(f, "Immediate"),
        }
    }
}

/// Escalation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationMode {
    #[default]
    Automatic,
    SemiAutomatic,
    Manual,
    Disabled,
}

impl std::fmt::Display for EscalationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "Automatic"),
            Self::SemiAutomatic => write!(f, "Semi-Automatic (ask first)"),
            Self::Manual => write!(f, "Manual (user decides)"),
            Self::Disabled => write!(f, "Disabled"),
        }
    }
}

/// Notification preference for escalations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationNotify {
    #[default]
    Always,
    OnlyHighPriority,
    OnlyImmediate,
    Never,
}

impl std::fmt::Display for EscalationNotify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "Always Notify"),
            Self::OnlyHighPriority => write!(f, "High Priority Only"),
            Self::OnlyImmediate => write!(f, "Immediate Only"),
            Self::Never => write!(f, "Never Notify"),
        }
    }
}
