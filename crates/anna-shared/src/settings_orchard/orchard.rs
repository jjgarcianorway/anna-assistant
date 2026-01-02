// v0.0.766: Settings Orchard
// Main orchard structure

use super::config::OrchardConfig;
use super::fruit::OrchardFruit;
use super::picker::OrchardPicker;
use super::stats::OrchardStats;

/// Settings orchard
#[derive(Debug, Clone, Default)]
pub struct SettingsOrchard {
    /// Config
    config: OrchardConfig,
    /// Fruits
    fruits: Vec<OrchardFruit>,
    /// Pickers
    pickers: Vec<OrchardPicker>,
    /// Stats
    stats: OrchardStats,
}

impl SettingsOrchard {
    /// Create new orchard system
    pub fn new(config: OrchardConfig) -> Self {
        Self {
            config,
            fruits: Vec::new(),
            pickers: Vec::new(),
            stats: OrchardStats::default(),
        }
    }

    /// Add fruit
    pub fn add_fruit(&mut self, fruit: OrchardFruit) -> bool {
        if self.fruits.len() >= self.config.max_fruits {
            return false;
        }
        self.fruits.push(fruit);
        self.update_stats();
        true
    }

    /// Get fruit
    pub fn get_fruit(&self, id: &str) -> Option<&OrchardFruit> {
        self.fruits.iter().find(|f| f.id == id)
    }

    /// Get fruit mut
    pub fn get_fruit_mut(&mut self, id: &str) -> Option<&mut OrchardFruit> {
        self.fruits.iter_mut().find(|f| f.id == id)
    }

    /// Add picker
    pub fn add_picker(&mut self, picker: OrchardPicker) {
        self.pickers.push(picker);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.fruits, self.config.orchard_type);
    }

    /// Get stats
    pub fn stats(&self) -> &OrchardStats {
        &self.stats
    }

    /// Fruit count
    pub fn fruit_count(&self) -> usize {
        self.fruits.len()
    }
}
