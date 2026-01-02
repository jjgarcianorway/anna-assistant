// v0.0.779: Settings Apiary - Apiary (Phase 355)
// Settings apiary core

use super::config::ApiaryConfig;
use super::hive::ApiaryHive;
use super::beekeeper::ApiaryBeekeeper;
use super::stats::ApiaryStats;

/// Settings apiary
#[derive(Debug, Clone, Default)]
pub struct SettingsApiary {
    /// Config
    config: ApiaryConfig,
    /// Hives
    hives: Vec<ApiaryHive>,
    /// Beekeepers
    beekeepers: Vec<ApiaryBeekeeper>,
    /// Stats
    stats: ApiaryStats,
}

impl SettingsApiary {
    /// Create new apiary system
    pub fn new(config: ApiaryConfig) -> Self {
        Self {
            config,
            hives: Vec::new(),
            beekeepers: Vec::new(),
            stats: ApiaryStats::default(),
        }
    }

    /// Add hive
    pub fn add_hive(&mut self, hive: ApiaryHive) -> bool {
        if self.hives.len() >= self.config.max_hives {
            return false;
        }
        self.hives.push(hive);
        self.update_stats();
        true
    }

    /// Get hive
    pub fn get_hive(&self, id: &str) -> Option<&ApiaryHive> {
        self.hives.iter().find(|h| h.id == id)
    }

    /// Get hive mut
    pub fn get_hive_mut(&mut self, id: &str) -> Option<&mut ApiaryHive> {
        self.hives.iter_mut().find(|h| h.id == id)
    }

    /// Add beekeeper
    pub fn add_beekeeper(&mut self, beekeeper: ApiaryBeekeeper) {
        self.beekeepers.push(beekeeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.hives, self.config.apiary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ApiaryStats {
        &self.stats
    }

    /// Hive count
    pub fn hive_count(&self) -> usize {
        self.hives.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apiary_new() {
        let a = SettingsApiary::new(ApiaryConfig::default());
        assert_eq!(a.hive_count(), 0);
    }

    #[test]
    fn test_apiary_add_hive() {
        let mut a = SettingsApiary::new(ApiaryConfig::default());
        a.add_hive(ApiaryHive::new("h1", "Title", "Content"));
        assert_eq!(a.hive_count(), 1);
    }
}
