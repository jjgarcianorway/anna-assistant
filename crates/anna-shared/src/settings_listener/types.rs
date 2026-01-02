// v0.0.636: Settings Listener Types (Phase 212)
// Basic types for settings listeners

use serde::{Deserialize, Serialize};

/// Listener type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ListenerType {
    /// Passive listener
    #[default]
    Passive,
    /// Active listener
    Active,
    /// Reactive listener
    Reactive,
    /// Selective listener
    Selective,
    /// Persistent listener
    Persistent,
}

impl std::fmt::Display for ListenerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passive => write!(f, "passive"),
            Self::Active => write!(f, "active"),
            Self::Reactive => write!(f, "reactive"),
            Self::Selective => write!(f, "selective"),
            Self::Persistent => write!(f, "persistent"),
        }
    }
}

/// Listener state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ListenerState {
    /// Idle state
    #[default]
    Idle,
    /// Listening state
    Listening,
    /// Processing state
    Processing,
    /// Paused state
    Paused,
    /// Stopped state
    Stopped,
}

impl std::fmt::Display for ListenerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Listening => write!(f, "listening"),
            Self::Processing => write!(f, "processing"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}
