// v0.0.585: Settings Logging - Log Target (Phase 161)
// Log target/component definitions

use serde::{Deserialize, Serialize};

/// Log target/component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogTarget {
    /// Settings core
    Core,
    /// Persistence layer
    Persistence,
    /// Validation
    Validation,
    /// Migration
    Migration,
    /// Backup
    Backup,
    /// Sync
    Sync,
    /// API
    Api,
}

impl std::fmt::Display for LogTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Persistence => write!(f, "persistence"),
            Self::Validation => write!(f, "validation"),
            Self::Migration => write!(f, "migration"),
            Self::Backup => write!(f, "backup"),
            Self::Sync => write!(f, "sync"),
            Self::Api => write!(f, "api"),
        }
    }
}
