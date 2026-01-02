// v0.0.735: Settings Alliance (Phase 311)
// Settings alliance core

use super::config::AllianceConfig;
use super::commitment::AllianceCommitment;
use super::member::AllianceMember;
use super::stats::AllianceStats;

/// Settings alliance
#[derive(Debug, Clone, Default)]
pub struct SettingsAlliance {
    /// Config
    config: AllianceConfig,
    /// Commitments
    commitments: Vec<AllianceCommitment>,
    /// Members
    members: Vec<AllianceMember>,
    /// Stats
    stats: AllianceStats,
}

impl SettingsAlliance {
    /// Create new alliance system
    pub fn new(config: AllianceConfig) -> Self {
        Self {
            config,
            commitments: Vec::new(),
            members: Vec::new(),
            stats: AllianceStats::default(),
        }
    }

    /// Add commitment
    pub fn add_commitment(&mut self, commitment: AllianceCommitment) -> bool {
        if self.commitments.len() >= self.config.max_commitments {
            return false;
        }
        self.commitments.push(commitment);
        self.update_stats();
        true
    }

    /// Get commitment
    pub fn get_commitment(&self, id: &str) -> Option<&AllianceCommitment> {
        self.commitments.iter().find(|c| c.id == id)
    }

    /// Get commitment mut
    pub fn get_commitment_mut(&mut self, id: &str) -> Option<&mut AllianceCommitment> {
        self.commitments.iter_mut().find(|c| c.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: AllianceMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.commitments, self.config.alliance_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AllianceStats {
        &self.stats
    }

    /// Commitment count
    pub fn commitment_count(&self) -> usize {
        self.commitments.len()
    }
}
