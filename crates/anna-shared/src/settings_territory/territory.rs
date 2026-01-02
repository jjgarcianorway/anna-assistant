// v0.0.745: Settings Territory - Territory
// Main settings territory management

use super::config::TerritoryConfig;
use super::ordinance::TerritoryOrdinance;
use super::administrator::TerritoryAdministrator;
use super::stats::TerritoryStats;

/// Settings territory
#[derive(Debug, Clone, Default)]
pub struct SettingsTerritory {
    /// Config
    config: TerritoryConfig,
    /// Ordinances
    ordinances: Vec<TerritoryOrdinance>,
    /// Administrators
    administrators: Vec<TerritoryAdministrator>,
    /// Stats
    stats: TerritoryStats,
}

impl SettingsTerritory {
    /// Create new territory system
    pub fn new(config: TerritoryConfig) -> Self {
        Self {
            config,
            ordinances: Vec::new(),
            administrators: Vec::new(),
            stats: TerritoryStats::default(),
        }
    }

    /// Add ordinance
    pub fn add_ordinance(&mut self, ordinance: TerritoryOrdinance) -> bool {
        if self.ordinances.len() >= self.config.max_ordinances {
            return false;
        }
        self.ordinances.push(ordinance);
        self.update_stats();
        true
    }

    /// Get ordinance
    pub fn get_ordinance(&self, id: &str) -> Option<&TerritoryOrdinance> {
        self.ordinances.iter().find(|o| o.id == id)
    }

    /// Get ordinance mut
    pub fn get_ordinance_mut(&mut self, id: &str) -> Option<&mut TerritoryOrdinance> {
        self.ordinances.iter_mut().find(|o| o.id == id)
    }

    /// Add administrator
    pub fn add_administrator(&mut self, administrator: TerritoryAdministrator) {
        self.administrators.push(administrator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ordinances, self.config.territory_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TerritoryStats {
        &self.stats
    }

    /// Ordinance count
    pub fn ordinance_count(&self) -> usize {
        self.ordinances.len()
    }
}
