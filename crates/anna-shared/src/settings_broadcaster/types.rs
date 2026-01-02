// v0.0.635: Settings Broadcaster Types (Phase 211)
// Core types for the settings broadcaster

use serde::{Deserialize, Serialize};

/// Broadcast channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BroadcastChannel {
    /// Default channel
    #[default]
    Default,
    /// System channel
    System,
    /// Application channel
    Application,
    /// Priority channel
    Priority,
    /// Debug channel
    Debug,
}

impl std::fmt::Display for BroadcastChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::System => write!(f, "system"),
            Self::Application => write!(f, "application"),
            Self::Priority => write!(f, "priority"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

/// Broadcast mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BroadcastMode {
    /// Synchronous broadcast
    #[default]
    Sync,
    /// Asynchronous broadcast
    Async,
    /// Fire and forget
    FireAndForget,
    /// Queued broadcast
    Queued,
}

impl std::fmt::Display for BroadcastMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync => write!(f, "sync"),
            Self::Async => write!(f, "async"),
            Self::FireAndForget => write!(f, "fire_and_forget"),
            Self::Queued => write!(f, "queued"),
        }
    }
}
