// v0.0.748: Settings District (Phase 324)
// Main district management

use super::config::DistrictConfig;
use super::bylaw::DistrictBylaw;
use super::official::DistrictOfficial;
use super::stats::DistrictStats;

/// Settings district
#[derive(Debug, Clone, Default)]
pub struct SettingsDistrict {
    /// Config
    config: DistrictConfig,
    /// Bylaws
    bylaws: Vec<DistrictBylaw>,
    /// Officials
    officials: Vec<DistrictOfficial>,
    /// Stats
    stats: DistrictStats,
}

impl SettingsDistrict {
    /// Create new district system
    pub fn new(config: DistrictConfig) -> Self {
        Self {
            config,
            bylaws: Vec::new(),
            officials: Vec::new(),
            stats: DistrictStats::default(),
        }
    }

    /// Add bylaw
    pub fn add_bylaw(&mut self, bylaw: DistrictBylaw) -> bool {
        if self.bylaws.len() >= self.config.max_bylaws {
            return false;
        }
        self.bylaws.push(bylaw);
        self.update_stats();
        true
    }

    /// Get bylaw
    pub fn get_bylaw(&self, id: &str) -> Option<&DistrictBylaw> {
        self.bylaws.iter().find(|b| b.id == id)
    }

    /// Get bylaw mut
    pub fn get_bylaw_mut(&mut self, id: &str) -> Option<&mut DistrictBylaw> {
        self.bylaws.iter_mut().find(|b| b.id == id)
    }

    /// Add official
    pub fn add_official(&mut self, official: DistrictOfficial) {
        self.officials.push(official);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.bylaws, self.config.district_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DistrictStats {
        &self.stats
    }

    /// Bylaw count
    pub fn bylaw_count(&self) -> usize {
        self.bylaws.len()
    }
}
