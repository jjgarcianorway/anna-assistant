// v0.0.747: Settings Region (Phase 323)
// Settings region main struct

use super::config::RegionConfig;
use super::policy::RegionPolicy;
use super::coordinator::RegionCoordinator;
use super::stats::RegionStats;

/// Settings region
#[derive(Debug, Clone, Default)]
pub struct SettingsRegion {
    /// Config
    config: RegionConfig,
    /// Policies
    policies: Vec<RegionPolicy>,
    /// Coordinators
    coordinators: Vec<RegionCoordinator>,
    /// Stats
    stats: RegionStats,
}

impl SettingsRegion {
    /// Create new region system
    pub fn new(config: RegionConfig) -> Self {
        Self {
            config,
            policies: Vec::new(),
            coordinators: Vec::new(),
            stats: RegionStats::default(),
        }
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: RegionPolicy) -> bool {
        if self.policies.len() >= self.config.max_policies {
            return false;
        }
        self.policies.push(policy);
        self.update_stats();
        true
    }

    /// Get policy
    pub fn get_policy(&self, id: &str) -> Option<&RegionPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }

    /// Get policy mut
    pub fn get_policy_mut(&mut self, id: &str) -> Option<&mut RegionPolicy> {
        self.policies.iter_mut().find(|p| p.id == id)
    }

    /// Add coordinator
    pub fn add_coordinator(&mut self, coordinator: RegionCoordinator) {
        self.coordinators.push(coordinator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.policies, self.config.region_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RegionStats {
        &self.stats
    }

    /// Policy count
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_new() {
        let r = SettingsRegion::new(RegionConfig::default());
        assert_eq!(r.policy_count(), 0);
    }

    #[test]
    fn test_region_add_policy() {
        let mut r = SettingsRegion::new(RegionConfig::default());
        r.add_policy(RegionPolicy::new("p1", "Title", "Content"));
        assert_eq!(r.policy_count(), 1);
    }
}
