// v0.0.756: Settings Lot (Phase 332)
// Settings lot main struct

use super::config::LotConfig;
use super::deed::LotDeed;
use super::assessor::LotAssessor;
use super::stats::LotStats;

/// Settings lot
#[derive(Debug, Clone, Default)]
pub struct SettingsLot {
    /// Config
    config: LotConfig,
    /// Deeds
    deeds: Vec<LotDeed>,
    /// Assessors
    assessors: Vec<LotAssessor>,
    /// Stats
    stats: LotStats,
}

impl SettingsLot {
    /// Create new lot system
    pub fn new(config: LotConfig) -> Self {
        Self {
            config,
            deeds: Vec::new(),
            assessors: Vec::new(),
            stats: LotStats::default(),
        }
    }

    /// Add deed
    pub fn add_deed(&mut self, deed: LotDeed) -> bool {
        if self.deeds.len() >= self.config.max_deeds {
            return false;
        }
        self.deeds.push(deed);
        self.update_stats();
        true
    }

    /// Get deed
    pub fn get_deed(&self, id: &str) -> Option<&LotDeed> {
        self.deeds.iter().find(|d| d.id == id)
    }

    /// Get deed mut
    pub fn get_deed_mut(&mut self, id: &str) -> Option<&mut LotDeed> {
        self.deeds.iter_mut().find(|d| d.id == id)
    }

    /// Add assessor
    pub fn add_assessor(&mut self, assessor: LotAssessor) {
        self.assessors.push(assessor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.deeds, self.config.lot_type);
    }

    /// Get stats
    pub fn stats(&self) -> &LotStats {
        &self.stats
    }

    /// Deed count
    pub fn deed_count(&self) -> usize {
        self.deeds.len()
    }
}
