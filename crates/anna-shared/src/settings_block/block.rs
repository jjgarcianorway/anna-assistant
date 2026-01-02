// v0.0.755: Settings Block (Phase 331)
// Settings block main struct

use super::config::BlockConfig;
use super::plat::{BlockPlat, BlockSurveyor};
use super::stats::BlockStats;

/// Settings block
#[derive(Debug, Clone, Default)]
pub struct SettingsBlock {
    /// Config
    config: BlockConfig,
    /// Plats
    plats: Vec<BlockPlat>,
    /// Surveyors
    surveyors: Vec<BlockSurveyor>,
    /// Stats
    stats: BlockStats,
}

impl SettingsBlock {
    /// Create new block system
    pub fn new(config: BlockConfig) -> Self {
        Self {
            config,
            plats: Vec::new(),
            surveyors: Vec::new(),
            stats: BlockStats::default(),
        }
    }

    /// Add plat
    pub fn add_plat(&mut self, plat: BlockPlat) -> bool {
        if self.plats.len() >= self.config.max_plats {
            return false;
        }
        self.plats.push(plat);
        self.update_stats();
        true
    }

    /// Get plat
    pub fn get_plat(&self, id: &str) -> Option<&BlockPlat> {
        self.plats.iter().find(|p| p.id == id)
    }

    /// Get plat mut
    pub fn get_plat_mut(&mut self, id: &str) -> Option<&mut BlockPlat> {
        self.plats.iter_mut().find(|p| p.id == id)
    }

    /// Add surveyor
    pub fn add_surveyor(&mut self, surveyor: BlockSurveyor) {
        self.surveyors.push(surveyor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plats, self.config.block_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BlockStats {
        &self.stats
    }

    /// Plat count
    pub fn plat_count(&self) -> usize {
        self.plats.len()
    }
}
