// v0.0.752: Settings Ward (Phase 328)
// Main ward structure

use super::config::WardConfig;
use super::motion::{WardMotion, WardDelegate};
use super::stats::WardStats;

/// Settings ward
#[derive(Debug, Clone, Default)]
pub struct SettingsWard {
    /// Config
    config: WardConfig,
    /// Motions
    motions: Vec<WardMotion>,
    /// Delegates
    delegates: Vec<WardDelegate>,
    /// Stats
    stats: WardStats,
}

impl SettingsWard {
    /// Create new ward system
    pub fn new(config: WardConfig) -> Self {
        Self {
            config,
            motions: Vec::new(),
            delegates: Vec::new(),
            stats: WardStats::default(),
        }
    }

    /// Add motion
    pub fn add_motion(&mut self, motion: WardMotion) -> bool {
        if self.motions.len() >= self.config.max_motions {
            return false;
        }
        self.motions.push(motion);
        self.update_stats();
        true
    }

    /// Get motion
    pub fn get_motion(&self, id: &str) -> Option<&WardMotion> {
        self.motions.iter().find(|m| m.id == id)
    }

    /// Get motion mut
    pub fn get_motion_mut(&mut self, id: &str) -> Option<&mut WardMotion> {
        self.motions.iter_mut().find(|m| m.id == id)
    }

    /// Add delegate
    pub fn add_delegate(&mut self, delegate: WardDelegate) {
        self.delegates.push(delegate);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.motions, self.config.ward_type);
    }

    /// Get stats
    pub fn stats(&self) -> &WardStats {
        &self.stats
    }

    /// Motion count
    pub fn motion_count(&self) -> usize {
        self.motions.len()
    }
}
