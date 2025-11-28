//! Roles Module v0.42.0
//!
//! Tracks Junior and Senior LLM role statistics for self-improvement.
//! Junior proposes plans, Senior reviews and corrects them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Roles storage directory
pub const ROLES_DIR: &str = "/var/lib/anna/knowledge/roles";

/// Junior LLM statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JuniorStats {
    /// Total plans proposed
    pub plans_proposed: u64,
    /// Plans accepted by Senior
    pub plans_accepted: u64,
    /// Plans rejected by Senior
    pub plans_rejected: u64,
    /// Average reliability before Senior fix
    pub avg_reliability_start: f64,
    /// Average reliability after Senior correction
    pub avg_reliability_end: f64,
    /// Running count for averaging
    reliability_samples: u64,
    /// Questions answered correctly on first try
    pub first_try_success: u64,
    /// Questions needing revision
    pub needed_revision: u64,
}

impl JuniorStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a proposed plan
    pub fn record_proposal(&mut self, accepted: bool) {
        self.plans_proposed += 1;
        if accepted {
            self.plans_accepted += 1;
        } else {
            self.plans_rejected += 1;
        }
    }

    /// Record reliability improvement
    pub fn record_reliability(&mut self, before: f64, after: f64) {
        let n = self.reliability_samples as f64;
        self.avg_reliability_start = (self.avg_reliability_start * n + before) / (n + 1.0);
        self.avg_reliability_end = (self.avg_reliability_end * n + after) / (n + 1.0);
        self.reliability_samples += 1;
    }

    /// Record first try result
    pub fn record_first_try(&mut self, success: bool) {
        if success {
            self.first_try_success += 1;
        } else {
            self.needed_revision += 1;
        }
    }

    /// Acceptance rate (0.0 - 1.0)
    pub fn acceptance_rate(&self) -> f64 {
        if self.plans_proposed == 0 {
            return 0.5; // Neutral when no data
        }
        self.plans_accepted as f64 / self.plans_proposed as f64
    }

    /// Calculate Junior rank based on performance
    pub fn rank(&self) -> JuniorRank {
        let rate = self.acceptance_rate();
        if self.plans_proposed < 10 {
            JuniorRank::Trainee
        } else if rate >= 0.90 {
            JuniorRank::Expert
        } else if rate >= 0.75 {
            JuniorRank::Competent
        } else if rate >= 0.50 {
            JuniorRank::Learning
        } else {
            JuniorRank::Trainee
        }
    }
}

/// Junior rank (cosmetic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JuniorRank {
    Trainee,
    Learning,
    Competent,
    Expert,
}

impl JuniorRank {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trainee => "Trainee",
            Self::Learning => "Learning",
            Self::Competent => "Competent",
            Self::Expert => "Expert",
        }
    }
}

/// Senior LLM statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeniorStats {
    /// Draft errors detected
    pub errors_detected: u64,
    /// Fixes proposed
    pub fixes_proposed: u64,
    /// Times Senior overrode Junior completely
    pub overrides: u64,
    /// Average reliability gain per fix
    pub avg_reliability_gain: f64,
    /// Running count for averaging
    gain_samples: u64,
    /// Plans approved without changes
    pub approved_unchanged: u64,
    /// Plans improved
    pub plans_improved: u64,
}

impl SeniorStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record error detection
    pub fn record_error(&mut self) {
        self.errors_detected += 1;
    }

    /// Record a fix
    pub fn record_fix(&mut self, reliability_before: f64, reliability_after: f64) {
        self.fixes_proposed += 1;
        let gain = reliability_after - reliability_before;
        let n = self.gain_samples as f64;
        self.avg_reliability_gain = (self.avg_reliability_gain * n + gain) / (n + 1.0);
        self.gain_samples += 1;

        if gain > 0.0 {
            self.plans_improved += 1;
        }
    }

    /// Record override
    pub fn record_override(&mut self) {
        self.overrides += 1;
    }

    /// Record approval without changes
    pub fn record_approval(&mut self) {
        self.approved_unchanged += 1;
    }

    /// Calculate Senior rank based on performance
    pub fn rank(&self) -> SeniorRank {
        let total_reviews = self.approved_unchanged + self.fixes_proposed;
        if total_reviews < 10 {
            SeniorRank::Reviewer
        } else if self.avg_reliability_gain >= 0.15 && self.plans_improved > 5 {
            SeniorRank::Architect
        } else if self.avg_reliability_gain >= 0.05 {
            SeniorRank::Mentor
        } else {
            SeniorRank::Reviewer
        }
    }
}

/// Senior rank (cosmetic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeniorRank {
    Reviewer,
    Mentor,
    Architect,
}

impl SeniorRank {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reviewer => "Reviewer",
            Self::Mentor => "Mentor",
            Self::Architect => "Architect",
        }
    }
}

/// Combined role statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleStats {
    /// Junior statistics
    pub junior: JuniorStats,
    /// Senior statistics
    pub senior: SeniorStats,
    /// Last updated timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

impl RoleStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from default location
    pub fn load_default() -> anyhow::Result<Self> {
        Self::load(&Self::default_path())
    }

    /// Load from file
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let stats: RoleStats = serde_json::from_str(&content)?;
        Ok(stats)
    }

    /// Save to default location
    pub fn save_default(&self) -> anyhow::Result<()> {
        self.save(&Self::default_path())
    }

    /// Save to file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Default path for role stats
    pub fn default_path() -> PathBuf {
        PathBuf::from(ROLES_DIR).join("role_stats.json")
    }

    /// Mark as updated
    pub fn touch(&mut self) {
        self.last_updated = Some(Utc::now());
    }

    /// Check if Junior is performing well on simple questions
    pub fn junior_reliable_simple(&self) -> bool {
        self.junior.acceptance_rate() >= 0.80 && self.junior.plans_proposed >= 10
    }

    /// Check if Junior needs stricter review
    pub fn junior_needs_strict_review(&self) -> bool {
        self.junior.acceptance_rate() < 0.50 && self.junior.plans_proposed >= 5
    }

    /// Check if Senior consistently fails to improve
    pub fn senior_ineffective(&self) -> bool {
        self.senior.avg_reliability_gain < 0.02 && self.senior.fixes_proposed >= 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_junior_stats_default() {
        let stats = JuniorStats::new();
        assert_eq!(stats.plans_proposed, 0);
        assert_eq!(stats.acceptance_rate(), 0.5);
    }

    #[test]
    fn test_junior_record_proposal() {
        let mut stats = JuniorStats::new();
        stats.record_proposal(true);
        stats.record_proposal(true);
        stats.record_proposal(false);

        assert_eq!(stats.plans_proposed, 3);
        assert_eq!(stats.plans_accepted, 2);
        assert_eq!(stats.plans_rejected, 1);
        assert!((stats.acceptance_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_junior_rank() {
        let mut stats = JuniorStats::new();
        assert_eq!(stats.rank(), JuniorRank::Trainee);

        // Simulate good performance
        for _ in 0..10 {
            stats.record_proposal(true);
        }
        assert_eq!(stats.rank(), JuniorRank::Expert);

        // Simulate mixed performance
        for _ in 0..10 {
            stats.record_proposal(false);
        }
        // Now 10/20 = 50%
        assert_eq!(stats.rank(), JuniorRank::Learning);
    }

    #[test]
    fn test_senior_stats_default() {
        let stats = SeniorStats::new();
        assert_eq!(stats.errors_detected, 0);
        assert_eq!(stats.rank(), SeniorRank::Reviewer);
    }

    #[test]
    fn test_senior_record_fix() {
        let mut stats = SeniorStats::new();
        stats.record_fix(0.50, 0.70);
        stats.record_fix(0.60, 0.85);

        assert_eq!(stats.fixes_proposed, 2);
        assert_eq!(stats.plans_improved, 2);
        assert!((stats.avg_reliability_gain - 0.225).abs() < 0.01);
    }

    #[test]
    fn test_role_stats_judgments() {
        let mut stats = RoleStats::new();

        // Junior performs well
        for _ in 0..15 {
            stats.junior.record_proposal(true);
        }
        assert!(stats.junior_reliable_simple());
        assert!(!stats.junior_needs_strict_review());

        // Reset and test poor performance
        stats.junior = JuniorStats::new();
        for _ in 0..3 {
            stats.junior.record_proposal(true);
        }
        for _ in 0..7 {
            stats.junior.record_proposal(false);
        }
        assert!(!stats.junior_reliable_simple());
        assert!(stats.junior_needs_strict_review());
    }

    #[test]
    fn test_serialization() {
        let mut stats = RoleStats::new();
        stats.junior.record_proposal(true);
        stats.senior.record_fix(0.5, 0.7);
        stats.touch();

        let json = serde_json::to_string(&stats).unwrap();
        let loaded: RoleStats = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.junior.plans_proposed, 1);
        assert_eq!(loaded.senior.fixes_proposed, 1);
    }
}
