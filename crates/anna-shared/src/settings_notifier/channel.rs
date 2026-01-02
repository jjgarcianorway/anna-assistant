// v0.0.639: Settings Notifier - Channel (Phase 215)
// Notification channel types

use serde::{Deserialize, Serialize};

/// Notification channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotifyChannel {
    /// Internal channel
    #[default]
    Internal,
    /// Log channel
    Log,
    /// Event channel
    Event,
    /// Callback channel
    Callback,
    /// External channel
    External,
}

impl std::fmt::Display for NotifyChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "internal"),
            Self::Log => write!(f, "log"),
            Self::Event => write!(f, "event"),
            Self::Callback => write!(f, "callback"),
            Self::External => write!(f, "external"),
        }
    }
}
