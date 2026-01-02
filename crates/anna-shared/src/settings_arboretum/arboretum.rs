// v0.0.772: Settings Arboretum Core (Phase 348)
// Main arboretum system implementation

use super::config::ArboretumConfig;
use super::specimen::{ArboretumSpecimen, ArboretumDendrologist};
use super::stats::ArboretumStats;

/// Settings arboretum
#[derive(Debug, Clone, Default)]
pub struct SettingsArboretum {
    /// Config
    config: ArboretumConfig,
    /// Specimens
    specimens: Vec<ArboretumSpecimen>,
    /// Dendrologists
    dendrologists: Vec<ArboretumDendrologist>,
    /// Stats
    stats: ArboretumStats,
}

impl SettingsArboretum {
    /// Create new arboretum system
    pub fn new(config: ArboretumConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            dendrologists: Vec::new(),
            stats: ArboretumStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ArboretumSpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ArboretumSpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ArboretumSpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add dendrologist
    pub fn add_dendrologist(&mut self, dendrologist: ArboretumDendrologist) {
        self.dendrologists.push(dendrologist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.arboretum_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ArboretumStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}
