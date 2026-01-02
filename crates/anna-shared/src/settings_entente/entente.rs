// v0.0.734: Settings Entente (Phase 310)
// Settings entente main implementation

use super::config::EntenteConfig;
use super::understanding::EntenteUnderstanding;
use super::partner::EntentePartner;
use super::stats::EntenteStats;

/// Settings entente
#[derive(Debug, Clone, Default)]
pub struct SettingsEntente {
    /// Config
    config: EntenteConfig,
    /// Understandings
    understandings: Vec<EntenteUnderstanding>,
    /// Partners
    partners: Vec<EntentePartner>,
    /// Stats
    stats: EntenteStats,
}

impl SettingsEntente {
    /// Create new entente system
    pub fn new(config: EntenteConfig) -> Self {
        Self {
            config,
            understandings: Vec::new(),
            partners: Vec::new(),
            stats: EntenteStats::default(),
        }
    }

    /// Add understanding
    pub fn add_understanding(&mut self, understanding: EntenteUnderstanding) -> bool {
        if self.understandings.len() >= self.config.max_understandings {
            return false;
        }
        self.understandings.push(understanding);
        self.update_stats();
        true
    }

    /// Get understanding
    pub fn get_understanding(&self, id: &str) -> Option<&EntenteUnderstanding> {
        self.understandings.iter().find(|u| u.id == id)
    }

    /// Get understanding mut
    pub fn get_understanding_mut(&mut self, id: &str) -> Option<&mut EntenteUnderstanding> {
        self.understandings.iter_mut().find(|u| u.id == id)
    }

    /// Add partner
    pub fn add_partner(&mut self, partner: EntentePartner) {
        self.partners.push(partner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.understandings, self.config.entente_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EntenteStats {
        &self.stats
    }

    /// Understanding count
    pub fn understanding_count(&self) -> usize {
        self.understandings.len()
    }
}
