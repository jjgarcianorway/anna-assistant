// v0.0.763: Settings Meadow Core
// Main meadow and registry implementations

use super::config::MeadowConfig;
use super::data::{MeadowGrass, MeadowKeeper};
use super::stats::MeadowStats;
use std::collections::HashMap;

/// Settings meadow
#[derive(Debug, Clone, Default)]
pub struct SettingsMeadow {
    /// Config
    config: MeadowConfig,
    /// Grasses
    grasses: Vec<MeadowGrass>,
    /// Keepers
    keepers: Vec<MeadowKeeper>,
    /// Stats
    stats: MeadowStats,
}

impl SettingsMeadow {
    /// Create new meadow system
    pub fn new(config: MeadowConfig) -> Self {
        Self {
            config,
            grasses: Vec::new(),
            keepers: Vec::new(),
            stats: MeadowStats::default(),
        }
    }

    /// Add grass
    pub fn add_grass(&mut self, grass: MeadowGrass) -> bool {
        if self.grasses.len() >= self.config.max_grasses {
            return false;
        }
        self.grasses.push(grass);
        self.update_stats();
        true
    }

    /// Get grass
    pub fn get_grass(&self, id: &str) -> Option<&MeadowGrass> {
        self.grasses.iter().find(|g| g.id == id)
    }

    /// Get grass mut
    pub fn get_grass_mut(&mut self, id: &str) -> Option<&mut MeadowGrass> {
        self.grasses.iter_mut().find(|g| g.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: MeadowKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.grasses, self.config.meadow_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MeadowStats {
        &self.stats
    }

    /// Grass count
    pub fn grass_count(&self) -> usize {
        self.grasses.len()
    }
}

/// Meadow registry
#[derive(Debug, Clone, Default)]
pub struct MeadowRegistry {
    /// Meadows by ID
    meadows: HashMap<String, SettingsMeadow>,
}

impl MeadowRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register meadow
    pub fn register(&mut self, id: impl Into<String>, meadow: SettingsMeadow) {
        self.meadows.insert(id.into(), meadow);
    }

    /// Unregister meadow
    pub fn unregister(&mut self, id: &str) -> bool {
        self.meadows.remove(id).is_some()
    }

    /// Get meadow
    pub fn get(&self, id: &str) -> Option<&SettingsMeadow> {
        self.meadows.get(id)
    }

    /// Get meadow mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMeadow> {
        self.meadows.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.meadows.len()
    }
}
