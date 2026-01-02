// v0.0.761: Settings Hectare (Phase 337)
// Main SettingsHectare struct

use super::config::HectareConfig;
use super::record::{HectareRecord, HectareInspector};
use super::stats::HectareStats;

/// Settings hectare
#[derive(Debug, Clone, Default)]
pub struct SettingsHectare {
    /// Config
    config: HectareConfig,
    /// Records
    records: Vec<HectareRecord>,
    /// Inspectors
    inspectors: Vec<HectareInspector>,
    /// Stats
    stats: HectareStats,
}

impl SettingsHectare {
    /// Create new hectare system
    pub fn new(config: HectareConfig) -> Self {
        Self {
            config,
            records: Vec::new(),
            inspectors: Vec::new(),
            stats: HectareStats::default(),
        }
    }

    /// Add record
    pub fn add_record(&mut self, record: HectareRecord) -> bool {
        if self.records.len() >= self.config.max_records {
            return false;
        }
        self.records.push(record);
        self.update_stats();
        true
    }

    /// Get record
    pub fn get_record(&self, id: &str) -> Option<&HectareRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get record mut
    pub fn get_record_mut(&mut self, id: &str) -> Option<&mut HectareRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Add inspector
    pub fn add_inspector(&mut self, inspector: HectareInspector) {
        self.inspectors.push(inspector);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.records, self.config.hectare_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HectareStats {
        &self.stats
    }

    /// Record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}
