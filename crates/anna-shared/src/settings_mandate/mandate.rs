// v0.0.721: Settings Mandate Core (Phase 297)
// Main mandate system implementation

use super::types::{MandateConfig, MandateEvidence, MandateRequirement, MandateStats};

/// Settings mandate
#[derive(Debug, Clone, Default)]
pub struct SettingsMandate {
    /// Config
    config: MandateConfig,
    /// Requirements
    requirements: Vec<MandateRequirement>,
    /// Evidence
    evidence: Vec<MandateEvidence>,
    /// Stats
    stats: MandateStats,
}

impl SettingsMandate {
    /// Create new mandate system
    pub fn new(config: MandateConfig) -> Self {
        Self {
            config,
            requirements: Vec::new(),
            evidence: Vec::new(),
            stats: MandateStats::default(),
        }
    }

    /// Add requirement
    pub fn add_requirement(&mut self, requirement: MandateRequirement) -> bool {
        if self.requirements.len() >= self.config.max_mandates {
            return false;
        }
        self.requirements.push(requirement);
        self.update_stats();
        true
    }

    /// Get requirement
    pub fn get_requirement(&self, id: &str) -> Option<&MandateRequirement> {
        self.requirements.iter().find(|r| r.id == id)
    }

    /// Get requirement mut
    pub fn get_requirement_mut(&mut self, id: &str) -> Option<&mut MandateRequirement> {
        self.requirements.iter_mut().find(|r| r.id == id)
    }

    /// Add evidence
    pub fn add_evidence(&mut self, ev: MandateEvidence) {
        self.evidence.push(ev);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.requirements, self.config.mandate_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MandateStats {
        &self.stats
    }

    /// Requirement count
    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }
}
