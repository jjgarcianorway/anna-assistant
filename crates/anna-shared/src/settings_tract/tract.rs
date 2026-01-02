// v0.0.759: Settings Tract (Phase 335)
// Main tract structure

use super::config::TractConfig;
use super::grant::TractGrant;
use super::ranger::TractRanger;
use super::stats::TractStats;

/// Settings tract
#[derive(Debug, Clone, Default)]
pub struct SettingsTract {
    /// Config
    config: TractConfig,
    /// Grants
    grants: Vec<TractGrant>,
    /// Rangers
    rangers: Vec<TractRanger>,
    /// Stats
    stats: TractStats,
}

impl SettingsTract {
    /// Create new tract system
    pub fn new(config: TractConfig) -> Self {
        Self {
            config,
            grants: Vec::new(),
            rangers: Vec::new(),
            stats: TractStats::default(),
        }
    }

    /// Add grant
    pub fn add_grant(&mut self, grant: TractGrant) -> bool {
        if self.grants.len() >= self.config.max_grants {
            return false;
        }
        self.grants.push(grant);
        self.update_stats();
        true
    }

    /// Get grant
    pub fn get_grant(&self, id: &str) -> Option<&TractGrant> {
        self.grants.iter().find(|g| g.id == id)
    }

    /// Get grant mut
    pub fn get_grant_mut(&mut self, id: &str) -> Option<&mut TractGrant> {
        self.grants.iter_mut().find(|g| g.id == id)
    }

    /// Add ranger
    pub fn add_ranger(&mut self, ranger: TractRanger) {
        self.rangers.push(ranger);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.grants, self.config.tract_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TractStats {
        &self.stats
    }

    /// Grant count
    pub fn grant_count(&self) -> usize {
        self.grants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tract_new() {
        let t = SettingsTract::new(TractConfig::default());
        assert_eq!(t.grant_count(), 0);
    }

    #[test]
    fn test_tract_add_grant() {
        let mut t = SettingsTract::new(TractConfig::default());
        t.add_grant(TractGrant::new("g1", "Title", "Content"));
        assert_eq!(t.grant_count(), 1);
    }
}
