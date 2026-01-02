// v0.0.724: Settings Charter - Charter module
// Main settings charter implementation

use super::config::CharterConfig;
use super::provision::{CharterProvision, CharterAmendment};
use super::stats::CharterStats;

/// Settings charter
#[derive(Debug, Clone, Default)]
pub struct SettingsCharter {
    /// Config
    config: CharterConfig,
    /// Provisions
    provisions: Vec<CharterProvision>,
    /// Amendments
    amendments: Vec<CharterAmendment>,
    /// Stats
    stats: CharterStats,
}

impl SettingsCharter {
    /// Create new charter system
    pub fn new(config: CharterConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            amendments: Vec::new(),
            stats: CharterStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: CharterProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&CharterProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut CharterProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add amendment
    pub fn add_amendment(&mut self, amendment: CharterAmendment) {
        self.amendments.push(amendment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.charter_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CharterStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}
