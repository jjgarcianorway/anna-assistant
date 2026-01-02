// v0.0.783: Settings Refuge - Main Refuge (Phase 359)
// Settings refuge implementation

use super::config::RefugeConfig;
use super::inhabitant::{RefugeInhabitant, RefugeWarden};
use super::stats::RefugeStats;

/// Settings refuge
#[derive(Debug, Clone, Default)]
pub struct SettingsRefuge {
    /// Config
    config: RefugeConfig,
    /// Inhabitants
    inhabitants: Vec<RefugeInhabitant>,
    /// Wardens
    wardens: Vec<RefugeWarden>,
    /// Stats
    stats: RefugeStats,
}

impl SettingsRefuge {
    /// Create new refuge system
    pub fn new(config: RefugeConfig) -> Self {
        Self {
            config,
            inhabitants: Vec::new(),
            wardens: Vec::new(),
            stats: RefugeStats::default(),
        }
    }

    /// Add inhabitant
    pub fn add_inhabitant(&mut self, inhabitant: RefugeInhabitant) -> bool {
        if self.inhabitants.len() >= self.config.max_inhabitants {
            return false;
        }
        self.inhabitants.push(inhabitant);
        self.update_stats();
        true
    }

    /// Get inhabitant
    pub fn get_inhabitant(&self, id: &str) -> Option<&RefugeInhabitant> {
        self.inhabitants.iter().find(|i| i.id == id)
    }

    /// Get inhabitant mut
    pub fn get_inhabitant_mut(&mut self, id: &str) -> Option<&mut RefugeInhabitant> {
        self.inhabitants.iter_mut().find(|i| i.id == id)
    }

    /// Add warden
    pub fn add_warden(&mut self, warden: RefugeWarden) {
        self.wardens.push(warden);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.inhabitants, self.config.refuge_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RefugeStats {
        &self.stats
    }

    /// Inhabitant count
    pub fn inhabitant_count(&self) -> usize {
        self.inhabitants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refuge_new() {
        let r = SettingsRefuge::new(RefugeConfig::default());
        assert_eq!(r.inhabitant_count(), 0);
    }

    #[test]
    fn test_refuge_add_inhabitant() {
        let mut r = SettingsRefuge::new(RefugeConfig::default());
        r.add_inhabitant(RefugeInhabitant::new("i1", "Title", "Content"));
        assert_eq!(r.inhabitant_count(), 1);
    }
}
