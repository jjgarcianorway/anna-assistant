// v0.0.744: Settings Realm Core (Phase 320)
// Core settings realm implementation

use super::config::RealmConfig;
use super::decree::RealmDecree;
use super::vassal::RealmVassal;
use super::stats::RealmStats;

/// Settings realm
#[derive(Debug, Clone, Default)]
pub struct SettingsRealm {
    /// Config
    config: RealmConfig,
    /// Decrees
    decrees: Vec<RealmDecree>,
    /// Vassals
    vassals: Vec<RealmVassal>,
    /// Stats
    stats: RealmStats,
}

impl SettingsRealm {
    /// Create new realm system
    pub fn new(config: RealmConfig) -> Self {
        Self {
            config,
            decrees: Vec::new(),
            vassals: Vec::new(),
            stats: RealmStats::default(),
        }
    }

    /// Add decree
    pub fn add_decree(&mut self, decree: RealmDecree) -> bool {
        if self.decrees.len() >= self.config.max_decrees {
            return false;
        }
        self.decrees.push(decree);
        self.update_stats();
        true
    }

    /// Get decree
    pub fn get_decree(&self, id: &str) -> Option<&RealmDecree> {
        self.decrees.iter().find(|d| d.id == id)
    }

    /// Get decree mut
    pub fn get_decree_mut(&mut self, id: &str) -> Option<&mut RealmDecree> {
        self.decrees.iter_mut().find(|d| d.id == id)
    }

    /// Add vassal
    pub fn add_vassal(&mut self, vassal: RealmVassal) {
        self.vassals.push(vassal);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.decrees, self.config.realm_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RealmStats {
        &self.stats
    }

    /// Decree count
    pub fn decree_count(&self) -> usize {
        self.decrees.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_new() {
        let r = SettingsRealm::new(RealmConfig::default());
        assert_eq!(r.decree_count(), 0);
    }

    #[test]
    fn test_realm_add_decree() {
        let mut r = SettingsRealm::new(RealmConfig::default());
        r.add_decree(RealmDecree::new("d1", "Title", "Content"));
        assert_eq!(r.decree_count(), 1);
    }
}
