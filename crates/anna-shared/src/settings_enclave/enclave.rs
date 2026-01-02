// v0.0.787: Settings Enclave (Phase 363)
// Main settings enclave implementation

use super::config::EnclaveConfig;
use super::member::EnclaveMember;
use super::steward::EnclaveSteward;
use super::stats::EnclaveStats;

/// Settings enclave
#[derive(Debug, Clone, Default)]
pub struct SettingsEnclave {
    /// Config
    config: EnclaveConfig,
    /// Members
    members: Vec<EnclaveMember>,
    /// Stewards
    stewards: Vec<EnclaveSteward>,
    /// Stats
    stats: EnclaveStats,
}

impl SettingsEnclave {
    /// Create new enclave system
    pub fn new(config: EnclaveConfig) -> Self {
        Self {
            config,
            members: Vec::new(),
            stewards: Vec::new(),
            stats: EnclaveStats::default(),
        }
    }

    /// Add member
    pub fn add_member(&mut self, member: EnclaveMember) -> bool {
        if self.members.len() >= self.config.max_members {
            return false;
        }
        self.members.push(member);
        self.update_stats();
        true
    }

    /// Get member
    pub fn get_member(&self, id: &str) -> Option<&EnclaveMember> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Get member mut
    pub fn get_member_mut(&mut self, id: &str) -> Option<&mut EnclaveMember> {
        self.members.iter_mut().find(|m| m.id == id)
    }

    /// Add steward
    pub fn add_steward(&mut self, steward: EnclaveSteward) {
        self.stewards.push(steward);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.members, self.config.enclave_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EnclaveStats {
        &self.stats
    }

    /// Member count
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_new() {
        let e = SettingsEnclave::new(EnclaveConfig::default());
        assert_eq!(e.member_count(), 0);
    }

    #[test]
    fn test_enclave_add_member() {
        let mut e = SettingsEnclave::new(EnclaveConfig::default());
        e.add_member(EnclaveMember::new("m1", "Title", "Content"));
        assert_eq!(e.member_count(), 1);
    }
}
