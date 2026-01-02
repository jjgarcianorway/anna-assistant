// v0.0.662: Settings Patcher Types (Phase 238)
// Core types for patch operations and modes

use serde::{Deserialize, Serialize};

/// Patch operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PatchOperation {
    /// Add new key
    Add,
    /// Remove existing key
    Remove,
    /// Replace value
    #[default]
    Replace,
    /// Copy from another key
    Copy,
    /// Move to another key
    Move,
}

impl std::fmt::Display for PatchOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Remove => write!(f, "remove"),
            Self::Replace => write!(f, "replace"),
            Self::Copy => write!(f, "copy"),
            Self::Move => write!(f, "move"),
        }
    }
}

/// Patch mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PatchMode {
    /// Strict (fail on errors)
    #[default]
    Strict,
    /// Lenient (skip errors)
    Lenient,
    /// Dry run
    DryRun,
    /// Atomic (all or nothing)
    Atomic,
}

impl std::fmt::Display for PatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::DryRun => write!(f, "dry_run"),
            Self::Atomic => write!(f, "atomic"),
        }
    }
}
