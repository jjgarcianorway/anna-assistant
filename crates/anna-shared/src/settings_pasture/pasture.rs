// v0.0.764: Settings Pasture - Main Pasture (Phase 340)

use super::config::PastureConfig;
use super::herd::{PastureHerd, PastureHerder};
use super::stats::PastureStats;

/// Settings pasture
#[derive(Debug, Clone, Default)]
pub struct SettingsPasture {
    /// Config
    config: PastureConfig,
    /// Herds
    herds: Vec<PastureHerd>,
    /// Herders
    herders: Vec<PastureHerder>,
    /// Stats
    stats: PastureStats,
}

impl SettingsPasture {
    /// Create new pasture system
    pub fn new(config: PastureConfig) -> Self {
        Self {
            config,
            herds: Vec::new(),
            herders: Vec::new(),
            stats: PastureStats::default(),
        }
    }

    /// Add herd
    pub fn add_herd(&mut self, herd: PastureHerd) -> bool {
        if self.herds.len() >= self.config.max_herds {
            return false;
        }
        self.herds.push(herd);
        self.update_stats();
        true
    }

    /// Get herd
    pub fn get_herd(&self, id: &str) -> Option<&PastureHerd> {
        self.herds.iter().find(|h| h.id == id)
    }

    /// Get herd mut
    pub fn get_herd_mut(&mut self, id: &str) -> Option<&mut PastureHerd> {
        self.herds.iter_mut().find(|h| h.id == id)
    }

    /// Add herder
    pub fn add_herder(&mut self, herder: PastureHerder) {
        self.herders.push(herder);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.herds, self.config.pasture_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PastureStats {
        &self.stats
    }

    /// Herd count
    pub fn herd_count(&self) -> usize {
        self.herds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pasture_new() {
        let p = SettingsPasture::new(PastureConfig::default());
        assert_eq!(p.herd_count(), 0);
    }

    #[test]
    fn test_pasture_add_herd() {
        let mut p = SettingsPasture::new(PastureConfig::default());
        p.add_herd(PastureHerd::new("h1", "Title", "Content"));
        assert_eq!(p.herd_count(), 1);
    }
}
