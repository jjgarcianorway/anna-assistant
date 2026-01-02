// v0.0.736: Settings Coalition - Coalition (Phase 312)
// Settings coalition implementation

use super::agreement::CoalitionAgreement;
use super::config::CoalitionConfig;
use super::partner::CoalitionPartner;
use super::stats::CoalitionStats;

/// Settings coalition
#[derive(Debug, Clone, Default)]
pub struct SettingsCoalition {
    /// Config
    config: CoalitionConfig,
    /// Agreements
    agreements: Vec<CoalitionAgreement>,
    /// Partners
    partners: Vec<CoalitionPartner>,
    /// Stats
    stats: CoalitionStats,
}

impl SettingsCoalition {
    /// Create new coalition system
    pub fn new(config: CoalitionConfig) -> Self {
        Self {
            config,
            agreements: Vec::new(),
            partners: Vec::new(),
            stats: CoalitionStats::default(),
        }
    }

    /// Add agreement
    pub fn add_agreement(&mut self, agreement: CoalitionAgreement) -> bool {
        if self.agreements.len() >= self.config.max_agreements {
            return false;
        }
        self.agreements.push(agreement);
        self.update_stats();
        true
    }

    /// Get agreement
    pub fn get_agreement(&self, id: &str) -> Option<&CoalitionAgreement> {
        self.agreements.iter().find(|a| a.id == id)
    }

    /// Get agreement mut
    pub fn get_agreement_mut(&mut self, id: &str) -> Option<&mut CoalitionAgreement> {
        self.agreements.iter_mut().find(|a| a.id == id)
    }

    /// Add partner
    pub fn add_partner(&mut self, partner: CoalitionPartner) {
        self.partners.push(partner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.agreements, self.config.coalition_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CoalitionStats {
        &self.stats
    }

    /// Agreement count
    pub fn agreement_count(&self) -> usize {
        self.agreements.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coalition_new() {
        let c = SettingsCoalition::new(CoalitionConfig::default());
        assert_eq!(c.agreement_count(), 0);
    }

    #[test]
    fn test_coalition_add_agreement() {
        let mut c = SettingsCoalition::new(CoalitionConfig::default());
        c.add_agreement(CoalitionAgreement::new("a1", "Title", "Content"));
        assert_eq!(c.agreement_count(), 1);
    }
}
