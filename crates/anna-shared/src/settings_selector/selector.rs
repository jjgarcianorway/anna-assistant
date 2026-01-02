// v0.0.673: Settings Selector (Phase 249)
// Main settings selector implementation

use std::collections::HashMap;
use super::config::SelectorConfig;
use super::criteria::SelectionCriteria;
use super::result::SelectionResult;
use super::stats::SelectorStats;
use super::types::{SelectorType, MatchMode};

/// Settings selector
#[derive(Debug, Clone, Default)]
pub struct SettingsSelector {
    /// Config
    config: SelectorConfig,
    /// Stats
    stats: SelectorStats,
}

impl SettingsSelector {
    /// Create new selector
    pub fn new(config: SelectorConfig) -> Self {
        Self {
            config,
            stats: SelectorStats::default(),
        }
    }

    /// Select by criteria
    pub fn select(&mut self, settings: &HashMap<String, String>, criteria: &SelectionCriteria) -> SelectionResult {
        let scanned = settings.len();
        let entries: Vec<(String, String)> = settings.iter()
            .filter(|(k, v)| criteria.matches(k, v, self.config.case_insensitive))
            .take(self.config.max_selections)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let result = SelectionResult::success(entries, scanned);
        self.stats.record(&result, self.config.default_type);
        result
    }

    /// Select first N
    pub fn select_first(&mut self, settings: &HashMap<String, String>, n: usize) -> SelectionResult {
        let scanned = settings.len();
        let entries: Vec<(String, String)> = settings.iter()
            .take(n.min(self.config.max_selections))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let result = SelectionResult::success(entries, scanned);
        self.stats.record(&result, SelectorType::First);
        result
    }

    /// Select by key prefix
    pub fn select_by_prefix(&mut self, settings: &HashMap<String, String>, prefix: &str) -> SelectionResult {
        let criteria = SelectionCriteria::key(prefix, MatchMode::Prefix);
        self.select(settings, &criteria)
    }

    /// Get stats
    pub fn stats(&self) -> &SelectorStats {
        &self.stats
    }
}
