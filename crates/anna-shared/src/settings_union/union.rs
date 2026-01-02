// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Settings Union

use super::config::UnionConfig;
use super::provision::UnionProvision;
use super::member::UnionMember;
use super::stats::UnionStats;

/// Settings union
#[derive(Debug, Clone, Default)]
pub struct SettingsUnion {
    /// Config
    config: UnionConfig,
    /// Provisions
    provisions: Vec<UnionProvision>,
    /// Members
    members: Vec<UnionMember>,
    /// Stats
    stats: UnionStats,
}

impl SettingsUnion {
    /// Create new union system
    pub fn new(config: UnionConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            members: Vec::new(),
            stats: UnionStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: UnionProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&UnionProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut UnionProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: UnionMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.union_type);
    }

    /// Get stats
    pub fn stats(&self) -> &UnionStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}
