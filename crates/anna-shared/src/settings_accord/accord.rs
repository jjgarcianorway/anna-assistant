// v0.0.730: Settings Accord (Phase 306)
// Settings accord implementation

use super::types::{AccordConfig, AccordProvision, AccordSignatory, AccordStats};

/// Settings accord
#[derive(Debug, Clone, Default)]
pub struct SettingsAccord {
    /// Config
    config: AccordConfig,
    /// Provisions
    provisions: Vec<AccordProvision>,
    /// Signatories
    signatories: Vec<AccordSignatory>,
    /// Stats
    stats: AccordStats,
}

impl SettingsAccord {
    /// Create new accord system
    pub fn new(config: AccordConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            signatories: Vec::new(),
            stats: AccordStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: AccordProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&AccordProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut AccordProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add signatory
    pub fn add_signatory(&mut self, signatory: AccordSignatory) {
        self.signatories.push(signatory);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.accord_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AccordStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}
