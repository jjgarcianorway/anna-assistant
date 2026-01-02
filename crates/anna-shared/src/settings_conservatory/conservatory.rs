// v0.0.771: Settings Conservatory
// Main conservatory implementation

use super::config::ConservatoryConfig;
use super::curator::ConservatoryCurator;
use super::specimen::ConservatorySpecimen;
use super::stats::ConservatoryStats;

/// Settings conservatory
#[derive(Debug, Clone, Default)]
pub struct SettingsConservatory {
    /// Config
    config: ConservatoryConfig,
    /// Specimens
    specimens: Vec<ConservatorySpecimen>,
    /// Curators
    curators: Vec<ConservatoryCurator>,
    /// Stats
    stats: ConservatoryStats,
}

impl SettingsConservatory {
    /// Create new conservatory system
    pub fn new(config: ConservatoryConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            curators: Vec::new(),
            stats: ConservatoryStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ConservatorySpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ConservatorySpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ConservatorySpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add curator
    pub fn add_curator(&mut self, curator: ConservatoryCurator) {
        self.curators.push(curator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.conservatory_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConservatoryStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}
