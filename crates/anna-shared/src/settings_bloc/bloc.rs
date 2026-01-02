// v0.0.740: Settings Bloc Core (Phase 316)
// Core bloc system implementation

use super::config::BlocConfig;
use super::policy::BlocPolicy;
use super::member::BlocMember;
use super::stats::BlocStats;

/// Settings bloc
#[derive(Debug, Clone, Default)]
pub struct SettingsBloc {
    /// Config
    config: BlocConfig,
    /// Policies
    policies: Vec<BlocPolicy>,
    /// Members
    members: Vec<BlocMember>,
    /// Stats
    stats: BlocStats,
}

impl SettingsBloc {
    /// Create new bloc system
    pub fn new(config: BlocConfig) -> Self {
        Self {
            config,
            policies: Vec::new(),
            members: Vec::new(),
            stats: BlocStats::default(),
        }
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: BlocPolicy) -> bool {
        if self.policies.len() >= self.config.max_policies {
            return false;
        }
        self.policies.push(policy);
        self.update_stats();
        true
    }

    /// Get policy
    pub fn get_policy(&self, id: &str) -> Option<&BlocPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }

    /// Get policy mut
    pub fn get_policy_mut(&mut self, id: &str) -> Option<&mut BlocPolicy> {
        self.policies.iter_mut().find(|p| p.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: BlocMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.policies, self.config.bloc_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BlocStats {
        &self.stats
    }

    /// Policy count
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}
