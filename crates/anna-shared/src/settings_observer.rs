// v0.0.591: Settings Observer
// Observer for settings changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Observer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ObserverMode {
    #[default]
    Passive,
    Active,
    Eager,
}

impl std::fmt::Display for ObserverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passive => write!(f, "passive"),
            Self::Active => write!(f, "active"),
            Self::Eager => write!(f, "eager"),
        }
    }
}

/// Observer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverConfig {
    pub mode: ObserverMode,
    pub category: Option<SettingsCategory>,
    pub enabled: bool,
}

impl ObserverConfig {
    pub fn new(mode: ObserverMode) -> Self {
        Self { mode, category: None, enabled: true }
    }
}

/// Settings observer
#[derive(Debug, Clone, Default)]
pub struct SettingsObserver {
    configs: HashMap<String, ObserverConfig>,
}

impl SettingsObserver {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, id: String, config: ObserverConfig) {
        self.configs.insert(id, config);
    }
    pub fn count(&self) -> usize { self.configs.len() }
}

pub fn is_observer_query(query: &str) -> bool {
    query.to_lowercase().contains("observer")
}

pub fn observer_fun_fact() -> &'static str {
    "Anna's settings observers track changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", ObserverMode::Passive), "passive");
    }

    #[test]
    fn test_observer_new() {
        let o = SettingsObserver::new();
        assert_eq!(o.count(), 0);
    }
}
