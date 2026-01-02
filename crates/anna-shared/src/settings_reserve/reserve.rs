// v0.0.782: Settings Reserve - Reserve
// Main settings reserve implementation

use super::config::ReserveConfig;
use super::species::ReserveSpecies;
use super::ranger::ReserveRanger;
use super::stats::ReserveStats;

/// Settings reserve
#[derive(Debug, Clone, Default)]
pub struct SettingsReserve {
    /// Config
    config: ReserveConfig,
    /// Species
    species: Vec<ReserveSpecies>,
    /// Rangers
    rangers: Vec<ReserveRanger>,
    /// Stats
    stats: ReserveStats,
}

impl SettingsReserve {
    /// Create new reserve system
    pub fn new(config: ReserveConfig) -> Self {
        Self {
            config,
            species: Vec::new(),
            rangers: Vec::new(),
            stats: ReserveStats::default(),
        }
    }

    /// Add species
    pub fn add_species(&mut self, species: ReserveSpecies) -> bool {
        if self.species.len() >= self.config.max_species {
            return false;
        }
        self.species.push(species);
        self.update_stats();
        true
    }

    /// Get species
    pub fn get_species(&self, id: &str) -> Option<&ReserveSpecies> {
        self.species.iter().find(|s| s.id == id)
    }

    /// Get species mut
    pub fn get_species_mut(&mut self, id: &str) -> Option<&mut ReserveSpecies> {
        self.species.iter_mut().find(|s| s.id == id)
    }

    /// Add ranger
    pub fn add_ranger(&mut self, ranger: ReserveRanger) {
        self.rangers.push(ranger);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.species, self.config.reserve_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ReserveStats {
        &self.stats
    }

    /// Species count
    pub fn species_count(&self) -> usize {
        self.species.len()
    }
}
