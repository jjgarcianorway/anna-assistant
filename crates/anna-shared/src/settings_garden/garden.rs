// v0.0.768: Settings Garden (Phase 344)
// Settings garden implementation

use super::models::{GardenConfig, GardenPlant, GardenGardener, GardenStats};

/// Settings garden
#[derive(Debug, Clone, Default)]
pub struct SettingsGarden {
    /// Config
    config: GardenConfig,
    /// Plants
    plants: Vec<GardenPlant>,
    /// Gardeners
    gardeners: Vec<GardenGardener>,
    /// Stats
    stats: GardenStats,
}

impl SettingsGarden {
    /// Create new garden system
    pub fn new(config: GardenConfig) -> Self {
        Self {
            config,
            plants: Vec::new(),
            gardeners: Vec::new(),
            stats: GardenStats::default(),
        }
    }

    /// Add plant
    pub fn add_plant(&mut self, plant: GardenPlant) -> bool {
        if self.plants.len() >= self.config.max_plants {
            return false;
        }
        self.plants.push(plant);
        self.update_stats();
        true
    }

    /// Get plant
    pub fn get_plant(&self, id: &str) -> Option<&GardenPlant> {
        self.plants.iter().find(|p| p.id == id)
    }

    /// Get plant mut
    pub fn get_plant_mut(&mut self, id: &str) -> Option<&mut GardenPlant> {
        self.plants.iter_mut().find(|p| p.id == id)
    }

    /// Add gardener
    pub fn add_gardener(&mut self, gardener: GardenGardener) {
        self.gardeners.push(gardener);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plants, self.config.garden_type);
    }

    /// Get stats
    pub fn stats(&self) -> &GardenStats {
        &self.stats
    }

    /// Plant count
    pub fn plant_count(&self) -> usize {
        self.plants.len()
    }
}
