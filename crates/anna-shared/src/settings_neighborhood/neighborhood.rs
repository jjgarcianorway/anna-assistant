// v0.0.754: Settings Neighborhood (Phase 330)
// Main neighborhood structure

use super::config::NeighborhoodConfig;
use super::initiative::{NeighborhoodInitiative, NeighborhoodOrganizer};
use super::stats::NeighborhoodStats;

/// Settings neighborhood
#[derive(Debug, Clone, Default)]
pub struct SettingsNeighborhood {
    /// Config
    config: NeighborhoodConfig,
    /// Initiatives
    initiatives: Vec<NeighborhoodInitiative>,
    /// Organizers
    organizers: Vec<NeighborhoodOrganizer>,
    /// Stats
    stats: NeighborhoodStats,
}

impl SettingsNeighborhood {
    /// Create new neighborhood system
    pub fn new(config: NeighborhoodConfig) -> Self {
        Self {
            config,
            initiatives: Vec::new(),
            organizers: Vec::new(),
            stats: NeighborhoodStats::default(),
        }
    }

    /// Add initiative
    pub fn add_initiative(&mut self, initiative: NeighborhoodInitiative) -> bool {
        if self.initiatives.len() >= self.config.max_initiatives {
            return false;
        }
        self.initiatives.push(initiative);
        self.update_stats();
        true
    }

    /// Get initiative
    pub fn get_initiative(&self, id: &str) -> Option<&NeighborhoodInitiative> {
        self.initiatives.iter().find(|i| i.id == id)
    }

    /// Get initiative mut
    pub fn get_initiative_mut(&mut self, id: &str) -> Option<&mut NeighborhoodInitiative> {
        self.initiatives.iter_mut().find(|i| i.id == id)
    }

    /// Add organizer
    pub fn add_organizer(&mut self, organizer: NeighborhoodOrganizer) {
        self.organizers.push(organizer);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.initiatives, self.config.neighborhood_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NeighborhoodStats {
        &self.stats
    }

    /// Initiative count
    pub fn initiative_count(&self) -> usize {
        self.initiatives.len()
    }
}
