// v0.0.637: Settings Poller Types (Phase 213)
// Type definitions for settings watcher

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Watcher type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WatcherType {
    /// Polling watcher
    #[default]
    Polling,
    /// Event-based watcher
    EventBased,
    /// Hybrid watcher
    Hybrid,
    /// Lazy watcher
    Lazy,
    /// Eager watcher
    Eager,
}

impl std::fmt::Display for WatcherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polling => write!(f, "polling"),
            Self::EventBased => write!(f, "event_based"),
            Self::Hybrid => write!(f, "hybrid"),
            Self::Lazy => write!(f, "lazy"),
            Self::Eager => write!(f, "eager"),
        }
    }
}

/// Watch interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WatchInterval {
    /// Immediate
    Immediate,
    /// Fast (100ms)
    Fast,
    /// Normal (1s)
    #[default]
    Normal,
    /// Slow (5s)
    Slow,
    /// Custom interval (ms)
    Custom(u64),
}

impl std::fmt::Display for WatchInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "immediate"),
            Self::Fast => write!(f, "fast"),
            Self::Normal => write!(f, "normal"),
            Self::Slow => write!(f, "slow"),
            Self::Custom(ms) => write!(f, "custom_{}ms", ms),
        }
    }
}

/// Watcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Watcher type
    pub watcher_type: WatcherType,
    /// Interval
    pub interval: WatchInterval,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Key pattern
    pub key_pattern: Option<String>,
    /// Active
    pub active: bool,
}

impl WatcherConfig {
    /// Create new config
    pub fn new(watcher_type: WatcherType) -> Self {
        Self {
            watcher_type,
            interval: WatchInterval::Normal,
            category: None,
            key_pattern: None,
            active: true,
        }
    }

    /// Set interval
    pub fn interval(mut self, interval: WatchInterval) -> Self {
        self.interval = interval;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key pattern
    pub fn key_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.key_pattern = Some(pattern.into());
        self
    }
}

/// Watcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatcherStats {
    /// Total polls
    pub total_polls: usize,
    /// Events detected
    pub events_detected: usize,
    /// Changes detected
    pub changes_detected: usize,
}

impl WatcherStats {
    /// Record poll
    pub fn record_poll(&mut self) {
        self.total_polls += 1;
    }

    /// Record event
    pub fn record_event(&mut self) {
        self.events_detected += 1;
    }

    /// Record change
    pub fn record_change(&mut self) {
        self.changes_detected += 1;
    }

    /// Change rate
    pub fn change_rate(&self) -> f64 {
        if self.total_polls == 0 {
            0.0
        } else {
            self.changes_detected as f64 / self.total_polls as f64
        }
    }
}
