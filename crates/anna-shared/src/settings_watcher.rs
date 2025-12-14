// v0.0.560: Settings Watcher
// Watcher for settings file changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Watch mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WatchMode {
    #[default]
    Poll,
    Event,
    Hybrid,
}

impl std::fmt::Display for WatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Poll => write!(f, "poll"),
            Self::Event => write!(f, "event"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Watcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub mode: WatchMode,
    pub category: Option<SettingsCategory>,
    pub enabled: bool,
}

impl WatcherConfig {
    pub fn new(mode: WatchMode) -> Self {
        Self { mode, category: None, enabled: true }
    }
}

/// Settings watcher
#[derive(Debug, Clone, Default)]
pub struct SettingsWatcher {
    configs: HashMap<String, WatcherConfig>,
}

impl SettingsWatcher {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, id: String, config: WatcherConfig) {
        self.configs.insert(id, config);
    }
    pub fn count(&self) -> usize { self.configs.len() }
}

pub fn is_watcher_query(query: &str) -> bool {
    query.to_lowercase().contains("watcher")
}

pub fn watcher_fun_fact() -> &'static str {
    "Anna's settings watchers monitor file changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", WatchMode::Poll), "poll");
    }

    #[test]
    fn test_watcher_new() {
        let w = SettingsWatcher::new();
        assert_eq!(w.count(), 0);
    }
}
