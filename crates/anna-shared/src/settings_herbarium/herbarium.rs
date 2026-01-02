// v0.0.774: Settings Herbarium - Herbarium
// Main herbarium management

use super::config::HerbariumConfig;
use super::specimen::HerbariumSpecimen;
use super::taxonomist::HerbariumTaxonomist;
use super::stats::HerbariumStats;

/// Settings herbarium
#[derive(Debug, Clone, Default)]
pub struct SettingsHerbarium {
    /// Config
    config: HerbariumConfig,
    /// Specimens
    specimens: Vec<HerbariumSpecimen>,
    /// Taxonomists
    taxonomists: Vec<HerbariumTaxonomist>,
    /// Stats
    stats: HerbariumStats,
}

impl SettingsHerbarium {
    /// Create new herbarium system
    pub fn new(config: HerbariumConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            taxonomists: Vec::new(),
            stats: HerbariumStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: HerbariumSpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&HerbariumSpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut HerbariumSpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add taxonomist
    pub fn add_taxonomist(&mut self, taxonomist: HerbariumTaxonomist) {
        self.taxonomists.push(taxonomist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.herbarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HerbariumStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}
