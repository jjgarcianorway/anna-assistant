// v0.0.770: Settings Greenhouse - Greenhouse Module
// Main greenhouse system

use super::config::GreenhouseConfig;
use super::crop::{GreenhouseCrop, GreenhouseGrower};
use super::stats::GreenhouseStats;

/// Settings greenhouse
#[derive(Debug, Clone, Default)]
pub struct SettingsGreenhouse {
    /// Config
    config: GreenhouseConfig,
    /// Crops
    crops: Vec<GreenhouseCrop>,
    /// Growers
    growers: Vec<GreenhouseGrower>,
    /// Stats
    stats: GreenhouseStats,
}

impl SettingsGreenhouse {
    /// Create new greenhouse system
    pub fn new(config: GreenhouseConfig) -> Self {
        Self {
            config,
            crops: Vec::new(),
            growers: Vec::new(),
            stats: GreenhouseStats::default(),
        }
    }

    /// Add crop
    pub fn add_crop(&mut self, crop: GreenhouseCrop) -> bool {
        if self.crops.len() >= self.config.max_crops {
            return false;
        }
        self.crops.push(crop);
        self.update_stats();
        true
    }

    /// Get crop
    pub fn get_crop(&self, id: &str) -> Option<&GreenhouseCrop> {
        self.crops.iter().find(|c| c.id == id)
    }

    /// Get crop mut
    pub fn get_crop_mut(&mut self, id: &str) -> Option<&mut GreenhouseCrop> {
        self.crops.iter_mut().find(|c| c.id == id)
    }

    /// Add grower
    pub fn add_grower(&mut self, grower: GreenhouseGrower) {
        self.growers.push(grower);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.crops, self.config.greenhouse_type);
    }

    /// Get stats
    pub fn stats(&self) -> &GreenhouseStats {
        &self.stats
    }

    /// Crop count
    pub fn crop_count(&self) -> usize {
        self.crops.len()
    }
}
