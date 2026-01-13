//! Postmortem Analysis - Learn from failures systematically.
//!
//! Key principles:
//! 1. Explicit falsification - actively try to break hypotheses
//! 2. Negative memory - store what DIDN'T work and why
//! 3. Decay functions - knowledge decays with system drift
//! 4. Time-aware learning - some truths are temporal
//! 5. "I don't know" state - switch to investigator mode when uncertain
//!
//! v0.3.16: Initial implementation

mod analysis;
mod negative_memory;
mod decay;

pub use analysis::{
    analyze_failure, FailureAnalysis, FailureCause, FailureCategory, CountermeasureType,
};
pub use negative_memory::{NegativeMemory, FailedAttempt, load_negative_memory, save_negative_memory};
pub use decay::{DecayingBelief, BeliefStrength, calculate_decay};

use serde::{Deserialize, Serialize};

/// System drift events that affect belief confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemDrift {
    /// Kernel update
    KernelUpdate { from: String, to: String },
    /// Major package update
    MajorPackageUpdate { package: String, from: String, to: String },
    /// Config file change
    ConfigChange { path: String },
    /// Service restart
    ServiceRestart { service: String },
    /// Boot (new session)
    Boot { timestamp: String },
}

/// Learning mode based on novelty vs evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningMode {
    /// High confidence, can provide answers
    Solver,
    /// Low confidence, should generate experiments
    Investigator,
    /// In between, needs more data
    Cautious,
}

impl LearningMode {
    /// Determine mode from novelty and evidence scores
    pub fn from_scores(novelty: f32, evidence: f32) -> Self {
        let ratio = if evidence > 0.0 {
            novelty / evidence
        } else {
            f32::MAX
        };

        if ratio > 2.0 {
            // Novelty greatly exceeds evidence
            LearningMode::Investigator
        } else if ratio > 1.0 {
            // Some novelty
            LearningMode::Cautious
        } else {
            // Well-understood territory
            LearningMode::Solver
        }
    }
}

/// Command cost for selection policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCost {
    /// How much does it change the system? (0.0 = read-only, 1.0 = destructive)
    pub intrusiveness: f32,
    /// How easy to undo? (0.0 = irreversible, 1.0 = trivial rollback)
    pub reversibility: f32,
    /// Expected execution time (seconds)
    pub latency: f32,
    /// How much information does it provide? (0.0 = noise, 1.0 = definitive answer)
    pub information_gain: f32,
}

impl CommandCost {
    /// Calculate overall cost (lower is better)
    pub fn total_cost(&self) -> f32 {
        // Prefer low intrusiveness, high reversibility, low latency, high information gain
        let intrusive_penalty = self.intrusiveness * 2.0;
        let reversibility_bonus = (1.0 - self.reversibility) * 1.5;
        let latency_penalty = (self.latency / 10.0).min(1.0); // Normalize to ~1.0 for 10s commands
        let info_bonus = (1.0 - self.information_gain) * 2.0;

        intrusive_penalty + reversibility_bonus + latency_penalty + info_bonus
    }

    /// Classify a command based on known patterns
    pub fn for_command(cmd: &str) -> Self {
        let cmd_lower = cmd.to_lowercase();

        // Read-only commands
        if cmd_lower.starts_with("cat ")
            || cmd_lower.starts_with("ls ")
            || cmd_lower.starts_with("stat ")
            || cmd_lower.starts_with("head ")
            || cmd_lower.starts_with("tail ")
            || cmd_lower.starts_with("grep ")
        {
            return Self {
                intrusiveness: 0.0,
                reversibility: 1.0,
                latency: 0.1,
                information_gain: 0.5,
            };
        }

        // Status commands
        if cmd_lower.contains("status")
            || cmd_lower.contains("--version")
            || cmd_lower.contains("-Qi")
        {
            return Self {
                intrusiveness: 0.0,
                reversibility: 1.0,
                latency: 0.2,
                information_gain: 0.7,
            };
        }

        // Package installation
        if cmd_lower.contains("pacman -S") || cmd_lower.contains("yay -S") {
            return Self {
                intrusiveness: 0.7,
                reversibility: 0.8, // Can uninstall
                latency: 10.0,
                information_gain: 0.3,
            };
        }

        // Service modification
        if cmd_lower.contains("systemctl start")
            || cmd_lower.contains("systemctl stop")
            || cmd_lower.contains("systemctl restart")
        {
            return Self {
                intrusiveness: 0.5,
                reversibility: 0.9,
                latency: 1.0,
                information_gain: 0.4,
            };
        }

        // Destructive commands
        if cmd_lower.contains("rm -rf") || cmd_lower.contains("dd ") || cmd_lower.contains("mkfs") {
            return Self {
                intrusiveness: 1.0,
                reversibility: 0.0,
                latency: 5.0,
                information_gain: 0.1,
            };
        }

        // Default: moderate caution
        Self {
            intrusiveness: 0.3,
            reversibility: 0.5,
            latency: 1.0,
            information_gain: 0.5,
        }
    }
}

/// Time-aware truth - some facts are only valid in certain time windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTruth {
    /// The observation
    pub observation: String,
    /// When it was observed
    pub observed_at: String,
    /// Valid for how long (seconds)
    pub valid_for: u64,
    /// What might invalidate it
    pub invalidators: Vec<String>,
}

impl TemporalTruth {
    /// Check if this truth is still valid
    pub fn is_valid(&self) -> bool {
        if let Ok(observed) = chrono::DateTime::parse_from_rfc3339(&self.observed_at) {
            let now = chrono::Utc::now();
            let age = (now - observed.with_timezone(&chrono::Utc)).num_seconds() as u64;
            age < self.valid_for
        } else {
            false
        }
    }

    /// Categories of temporal phenomena
    pub fn temporal_categories() -> Vec<(&'static str, u64)> {
        vec![
            ("boot_time", 0), // Only valid until next reboot
            ("process_state", 60), // Processes can change quickly
            ("network_state", 300), // Network can converge over minutes
            ("dns_cache", 3600), // DNS TTLs
            ("package_state", 86400), // Packages change daily at most
            ("hardware_state", 604800), // Hardware rarely changes
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_mode() {
        assert_eq!(LearningMode::from_scores(0.5, 1.0), LearningMode::Solver);
        assert_eq!(LearningMode::from_scores(1.5, 1.0), LearningMode::Cautious);
        assert_eq!(LearningMode::from_scores(3.0, 1.0), LearningMode::Investigator);
    }

    #[test]
    fn test_command_cost() {
        let read_cost = CommandCost::for_command("cat /etc/passwd");
        let write_cost = CommandCost::for_command("rm -rf /tmp/test");

        assert!(read_cost.total_cost() < write_cost.total_cost());
        assert_eq!(read_cost.intrusiveness, 0.0);
        assert_eq!(write_cost.intrusiveness, 1.0);
    }
}
