// v0.0.777: Settings Terrarium (Phase 353)
// Main terrarium implementation

use crate::settings_terrarium::config::TerrariumConfig;
use crate::settings_terrarium::plant::{TerrariumPlant, TerrariumCreator};
use crate::settings_terrarium::stats::TerrariumStats;

/// Settings terrarium
#[derive(Debug, Clone, Default)]
pub struct SettingsTerrarium {
    /// Config
    config: TerrariumConfig,
    /// Plants
    plants: Vec<TerrariumPlant>,
    /// Creators
    creators: Vec<TerrariumCreator>,
    /// Stats
    stats: TerrariumStats,
}

impl SettingsTerrarium {
    /// Create new terrarium system
    pub fn new(config: TerrariumConfig) -> Self {
        Self {
            config,
            plants: Vec::new(),
            creators: Vec::new(),
            stats: TerrariumStats::default(),
        }
    }

    /// Add plant
    pub fn add_plant(&mut self, plant: TerrariumPlant) -> bool {
        if self.plants.len() >= self.config.max_plants {
            return false;
        }
        self.plants.push(plant);
        self.update_stats();
        true
    }

    /// Get plant
    pub fn get_plant(&self, id: &str) -> Option<&TerrariumPlant> {
        self.plants.iter().find(|p| p.id == id)
    }

    /// Get plant mut
    pub fn get_plant_mut(&mut self, id: &str) -> Option<&mut TerrariumPlant> {
        self.plants.iter_mut().find(|p| p.id == id)
    }

    /// Add creator
    pub fn add_creator(&mut self, creator: TerrariumCreator) {
        self.creators.push(creator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plants, self.config.terrarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TerrariumStats {
        &self.stats
    }

    /// Plant count
    pub fn plant_count(&self) -> usize {
        self.plants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrarium_new() {
        let t = SettingsTerrarium::new(TerrariumConfig::default());
        assert_eq!(t.plant_count(), 0);
    }

    #[test]
    fn test_terrarium_add_plant() {
        let mut t = SettingsTerrarium::new(TerrariumConfig::default());
        t.add_plant(TerrariumPlant::new("p1", "Title", "Content"));
        assert_eq!(t.plant_count(), 1);
    }
}
