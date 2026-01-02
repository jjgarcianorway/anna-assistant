// v0.0.773: Settings Botanical Core (Phase 349)
// Main botanical system implementation

use super::config::BotanicalConfig;
use super::collection::{BotanicalCollection, BotanicalBotanist};
use super::stats::BotanicalStats;

/// Settings botanical
#[derive(Debug, Clone, Default)]
pub struct SettingsBotanical {
    /// Config
    config: BotanicalConfig,
    /// Collections
    collections: Vec<BotanicalCollection>,
    /// Botanists
    botanists: Vec<BotanicalBotanist>,
    /// Stats
    stats: BotanicalStats,
}

impl SettingsBotanical {
    /// Create new botanical system
    pub fn new(config: BotanicalConfig) -> Self {
        Self {
            config,
            collections: Vec::new(),
            botanists: Vec::new(),
            stats: BotanicalStats::default(),
        }
    }

    /// Add collection
    pub fn add_collection(&mut self, collection: BotanicalCollection) -> bool {
        if self.collections.len() >= self.config.max_collections {
            return false;
        }
        self.collections.push(collection);
        self.update_stats();
        true
    }

    /// Get collection
    pub fn get_collection(&self, id: &str) -> Option<&BotanicalCollection> {
        self.collections.iter().find(|c| c.id == id)
    }

    /// Get collection mut
    pub fn get_collection_mut(&mut self, id: &str) -> Option<&mut BotanicalCollection> {
        self.collections.iter_mut().find(|c| c.id == id)
    }

    /// Add botanist
    pub fn add_botanist(&mut self, botanist: BotanicalBotanist) {
        self.botanists.push(botanist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.collections, self.config.botanical_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BotanicalStats {
        &self.stats
    }

    /// Collection count
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
}
