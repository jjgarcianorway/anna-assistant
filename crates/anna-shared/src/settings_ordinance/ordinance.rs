// v0.0.722: Settings Ordinance Implementation (Phase 298)
// Main settings ordinance struct

use super::types::{OrdinanceAmendment, OrdinanceConfig, OrdinanceProvision, OrdinanceStats};

/// Settings ordinance
#[derive(Debug, Clone, Default)]
pub struct SettingsOrdinance {
    /// Config
    config: OrdinanceConfig,
    /// Provisions
    provisions: Vec<OrdinanceProvision>,
    /// Amendments
    amendments: Vec<OrdinanceAmendment>,
    /// Stats
    stats: OrdinanceStats,
}

impl SettingsOrdinance {
    /// Create new ordinance system
    pub fn new(config: OrdinanceConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            amendments: Vec::new(),
            stats: OrdinanceStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: OrdinanceProvision) -> bool {
        if self.provisions.len() >= self.config.max_ordinances {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&OrdinanceProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut OrdinanceProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add amendment
    pub fn add_amendment(&mut self, amendment: OrdinanceAmendment) {
        self.amendments.push(amendment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.ordinance_type);
    }

    /// Get stats
    pub fn stats(&self) -> &OrdinanceStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}
