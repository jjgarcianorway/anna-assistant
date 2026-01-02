// v0.0.743: Settings Domain - Core Domain (Phase 319)
// Main domain and registry implementations

use std::collections::HashMap;
use super::domain_config::DomainConfig;
use super::domain_right::{DomainRight, DomainHolder};
use super::domain_stats::DomainStats;

/// Settings domain
#[derive(Debug, Clone, Default)]
pub struct SettingsDomain {
    /// Config
    config: DomainConfig,
    /// Rights
    rights: Vec<DomainRight>,
    /// Holders
    holders: Vec<DomainHolder>,
    /// Stats
    stats: DomainStats,
}

impl SettingsDomain {
    /// Create new domain system
    pub fn new(config: DomainConfig) -> Self {
        Self {
            config,
            rights: Vec::new(),
            holders: Vec::new(),
            stats: DomainStats::default(),
        }
    }

    /// Add right
    pub fn add_right(&mut self, right: DomainRight) -> bool {
        if self.rights.len() >= self.config.max_rights {
            return false;
        }
        self.rights.push(right);
        self.update_stats();
        true
    }

    /// Get right
    pub fn get_right(&self, id: &str) -> Option<&DomainRight> {
        self.rights.iter().find(|r| r.id == id)
    }

    /// Get right mut
    pub fn get_right_mut(&mut self, id: &str) -> Option<&mut DomainRight> {
        self.rights.iter_mut().find(|r| r.id == id)
    }

    /// Add holder
    pub fn add_holder(&mut self, holder: DomainHolder) {
        self.holders.push(holder);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.rights, self.config.domain_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DomainStats {
        &self.stats
    }

    /// Right count
    pub fn right_count(&self) -> usize {
        self.rights.len()
    }
}

/// Domain registry
#[derive(Debug, Clone, Default)]
pub struct DomainRegistry {
    /// Domains by ID
    domains: HashMap<String, SettingsDomain>,
}

impl DomainRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register domain
    pub fn register(&mut self, id: impl Into<String>, domain: SettingsDomain) {
        self.domains.insert(id.into(), domain);
    }

    /// Unregister domain
    pub fn unregister(&mut self, id: &str) -> bool {
        self.domains.remove(id).is_some()
    }

    /// Get domain
    pub fn get(&self, id: &str) -> Option<&SettingsDomain> {
        self.domains.get(id)
    }

    /// Get domain mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDomain> {
        self.domains.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.domains.len()
    }
}
