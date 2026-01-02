// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance - Treaty Implementation

use super::types::{TreatyConfig, TreatyProvision, TreatySignatory, TreatyStats};

/// Settings treaty
#[derive(Debug, Clone, Default)]
pub struct SettingsTreaty {
    /// Config
    config: TreatyConfig,
    /// Provisions
    provisions: Vec<TreatyProvision>,
    /// Signatories
    signatories: Vec<TreatySignatory>,
    /// Stats
    stats: TreatyStats,
}

impl SettingsTreaty {
    /// Create new treaty system
    pub fn new(config: TreatyConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            signatories: Vec::new(),
            stats: TreatyStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: TreatyProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&TreatyProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut TreatyProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add signatory
    pub fn add_signatory(&mut self, signatory: TreatySignatory) {
        self.signatories.push(signatory);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.treaty_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TreatyStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}
