// v0.0.767: Settings Vineyard
// Main vineyard structure

use super::config::VineyardConfig;
use super::vine::VineyardVine;
use super::vintner::VineyardVintner;
use super::stats::VineyardStats;

/// Settings vineyard
#[derive(Debug, Clone, Default)]
pub struct SettingsVineyard {
    /// Config
    config: VineyardConfig,
    /// Vines
    vines: Vec<VineyardVine>,
    /// Vintners
    vintners: Vec<VineyardVintner>,
    /// Stats
    stats: VineyardStats,
}

impl SettingsVineyard {
    /// Create new vineyard system
    pub fn new(config: VineyardConfig) -> Self {
        Self {
            config,
            vines: Vec::new(),
            vintners: Vec::new(),
            stats: VineyardStats::default(),
        }
    }

    /// Add vine
    pub fn add_vine(&mut self, vine: VineyardVine) -> bool {
        if self.vines.len() >= self.config.max_vines {
            return false;
        }
        self.vines.push(vine);
        self.update_stats();
        true
    }

    /// Get vine
    pub fn get_vine(&self, id: &str) -> Option<&VineyardVine> {
        self.vines.iter().find(|v| v.id == id)
    }

    /// Get vine mut
    pub fn get_vine_mut(&mut self, id: &str) -> Option<&mut VineyardVine> {
        self.vines.iter_mut().find(|v| v.id == id)
    }

    /// Add vintner
    pub fn add_vintner(&mut self, vintner: VineyardVintner) {
        self.vintners.push(vintner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.vines, self.config.vineyard_type);
    }

    /// Get stats
    pub fn stats(&self) -> &VineyardStats {
        &self.stats
    }

    /// Vine count
    pub fn vine_count(&self) -> usize {
        self.vines.len()
    }
}
