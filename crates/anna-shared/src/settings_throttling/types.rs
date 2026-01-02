// v0.0.589: Settings Throttling - Types (Phase 165)
// Throttle action types and results

use serde::{Deserialize, Serialize};

/// Throttle action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThrottleAction {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Export operation
    Export,
    /// Import operation
    Import,
    /// Sync operation
    Sync,
    /// Any operation
    Any,
}

impl std::fmt::Display for ThrottleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Export => write!(f, "export"),
            Self::Import => write!(f, "import"),
            Self::Sync => write!(f, "sync"),
            Self::Any => write!(f, "any"),
        }
    }
}

/// Throttle result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottleResult {
    /// Allowed
    Allowed,
    /// Rate limited
    Limited,
    /// Blocked
    Blocked,
}

impl std::fmt::Display for ThrottleResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allowed => write!(f, "allowed"),
            Self::Limited => write!(f, "limited"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

/// Throttle statistics
#[derive(Debug, Clone)]
pub struct ThrottleStats {
    /// Action
    pub action: ThrottleAction,
    /// Request count
    pub requests: usize,
    /// Limit configuration
    pub limit: Option<super::RateLimit>,
}
