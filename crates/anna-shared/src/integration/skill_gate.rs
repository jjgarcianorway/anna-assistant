//! Skill Gatekeeper - Validation harness as mandatory gate.
//!
//! A skill candidate cannot be used on the host unless it has:
//! 1. Passed a sandbox test suite for the relevant subsystem
//! 2. Declared its rollback procedure
//!
//! Skill tiers:
//! - Candidate: Sandbox only, never on live system
//! - Probation: Host allowed in cautious mode with extra verification
//! - Trusted: Normal use allowed
//!
//! Promotion requires passing BOTH functional tests AND safety invariants.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill trust tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkillTier {
    /// Sandbox only - never run on live system
    Candidate,
    /// Can run on host with extra verification
    Probation,
    /// Normal use allowed
    Trusted,
}

impl SkillTier {
    /// Can this tier run on the live host?
    pub fn can_run_on_host(&self) -> bool {
        matches!(self, SkillTier::Probation | SkillTier::Trusted)
    }

    /// Does this tier require extra verification?
    pub fn requires_verification(&self) -> bool {
        matches!(self, SkillTier::Probation)
    }

    /// Get tier name
    pub fn name(&self) -> &'static str {
        match self {
            SkillTier::Candidate => "candidate",
            SkillTier::Probation => "probation",
            SkillTier::Trusted => "trusted",
        }
    }
}

/// Safety invariant that must be checked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyInvariant {
    /// Invariant name
    pub name: String,
    /// Check function description
    pub check: String,
    /// Is this invariant satisfied?
    pub satisfied: bool,
    /// Error message if not satisfied
    pub error: Option<String>,
}

/// Predefined safety invariants
pub fn standard_safety_invariants() -> Vec<SafetyInvariant> {
    vec![
        SafetyInvariant {
            name: "no_destructive_without_snapshot".to_string(),
            check: "Destructive operations require system snapshot capability".to_string(),
            satisfied: false,
            error: None,
        },
        SafetyInvariant {
            name: "no_partial_upgrades".to_string(),
            check: "Package operations must not cause partial upgrades".to_string(),
            satisfied: false,
            error: None,
        },
        SafetyInvariant {
            name: "bounded_log_scans".to_string(),
            check: "Log scans must have time/size limits".to_string(),
            satisfied: false,
            error: None,
        },
        SafetyInvariant {
            name: "rollback_declared".to_string(),
            check: "Rollback procedure must be declared and tested".to_string(),
            satisfied: false,
            error: None,
        },
        SafetyInvariant {
            name: "no_unbounded_recursion".to_string(),
            check: "No recursive operations without depth limit".to_string(),
            satisfied: false,
            error: None,
        },
    ]
}

/// A gated skill ready for promotion evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatedSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Current tier
    pub tier: SkillTier,
    /// Commands in this skill
    pub commands: Vec<String>,
    /// Rollback procedure (required)
    pub rollback: Vec<String>,
    /// Sandbox test results
    pub sandbox_tests: Vec<TestResult>,
    /// Safety invariant checks
    pub safety_checks: Vec<SafetyInvariant>,
    /// Promotion history
    pub promotion_history: Vec<PromotionEvent>,
    /// Falsification attempts
    pub falsification_log: Vec<FalsificationAttempt>,
}

/// Result of a test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Did it pass?
    pub passed: bool,
    /// Test output
    pub output: String,
    /// Duration in ms
    pub duration_ms: u64,
    /// When the test ran
    pub ran_at: String,
    /// Test environment
    pub environment: String,
}

/// A promotion/demotion event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionEvent {
    /// From tier
    pub from: SkillTier,
    /// To tier
    pub to: SkillTier,
    /// Reason
    pub reason: String,
    /// When
    pub timestamp: String,
}

/// A falsification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationAttempt {
    /// What was tried
    pub description: String,
    /// Did it break the skill?
    pub broke_skill: bool,
    /// Test environment variations
    pub variations: Vec<String>,
    /// Result details
    pub result: String,
    /// When
    pub timestamp: String,
}

/// The skill gatekeeper
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillGatekeeper {
    /// All gated skills by ID
    pub skills: HashMap<String, GatedSkill>,
    /// Skills blocked from promotion
    pub blocked: HashMap<String, String>,
    /// Promotion requirements
    pub requirements: PromotionRequirements,
}

/// Requirements for promotion between tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRequirements {
    /// Minimum sandbox tests for candidate -> probation
    pub min_sandbox_tests: usize,
    /// Minimum pass rate for candidate -> probation
    pub min_pass_rate: f32,
    /// Required safety invariants
    pub required_invariants: Vec<String>,
    /// Minimum falsification attempts
    pub min_falsification_attempts: usize,
    /// Minimum successful host runs for probation -> trusted
    pub min_host_runs: usize,
}

impl Default for PromotionRequirements {
    fn default() -> Self {
        Self {
            min_sandbox_tests: 3,
            min_pass_rate: 1.0, // 100% pass rate required
            required_invariants: vec![
                "no_destructive_without_snapshot".to_string(),
                "no_partial_upgrades".to_string(),
                "rollback_declared".to_string(),
            ],
            min_falsification_attempts: 1,
            min_host_runs: 5,
        }
    }
}

impl SkillGatekeeper {
    /// Create new gatekeeper
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new skill candidate
    pub fn register_candidate(&mut self, name: &str, commands: Vec<String>, rollback: Vec<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();

        let skill = GatedSkill {
            id: id.clone(),
            name: name.to_string(),
            tier: SkillTier::Candidate,
            commands,
            rollback,
            sandbox_tests: Vec::new(),
            safety_checks: standard_safety_invariants(),
            promotion_history: Vec::new(),
            falsification_log: Vec::new(),
        };

        self.skills.insert(id.clone(), skill);
        id
    }

    /// Check if a skill can be used on the host
    pub fn can_use_on_host(&self, skill_id: &str) -> Result<(), String> {
        let skill = self.skills.get(skill_id).ok_or("Skill not found")?;

        if self.blocked.contains_key(skill_id) {
            return Err(format!(
                "Skill blocked: {}",
                self.blocked.get(skill_id).unwrap()
            ));
        }

        if !skill.tier.can_run_on_host() {
            return Err(format!(
                "Skill '{}' is tier {:?} - sandbox only",
                skill.name, skill.tier
            ));
        }

        if skill.rollback.is_empty() {
            return Err("Skill has no rollback procedure declared".to_string());
        }

        Ok(())
    }

    /// Record a sandbox test result
    pub fn record_test(&mut self, skill_id: &str, result: TestResult) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;
        skill.sandbox_tests.push(result);
        Ok(())
    }

    /// Record a falsification attempt
    pub fn record_falsification(
        &mut self,
        skill_id: &str,
        attempt: FalsificationAttempt,
    ) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;

        // If falsification succeeded (broke the skill), block promotion
        if attempt.broke_skill {
            self.blocked.insert(
                skill_id.to_string(),
                format!("Failed falsification: {}", attempt.description),
            );
        }

        skill.falsification_log.push(attempt);
        Ok(())
    }

    /// Check if a skill can be promoted
    pub fn can_promote(&self, skill_id: &str) -> Result<SkillTier, Vec<String>> {
        let skill = self.skills.get(skill_id).ok_or_else(|| vec!["Skill not found".to_string()])?;
        let mut failures = Vec::new();

        // Check if blocked
        if let Some(reason) = self.blocked.get(skill_id) {
            failures.push(format!("Blocked: {}", reason));
            return Err(failures);
        }

        // Check rollback
        if skill.rollback.is_empty() {
            failures.push("No rollback procedure declared".to_string());
        }

        // Check sandbox tests
        if skill.sandbox_tests.len() < self.requirements.min_sandbox_tests {
            failures.push(format!(
                "Need {} sandbox tests, have {}",
                self.requirements.min_sandbox_tests,
                skill.sandbox_tests.len()
            ));
        }

        // Check pass rate
        let passed = skill.sandbox_tests.iter().filter(|t| t.passed).count();
        let total = skill.sandbox_tests.len();
        if total > 0 {
            let rate = passed as f32 / total as f32;
            if rate < self.requirements.min_pass_rate {
                failures.push(format!(
                    "Pass rate {:.0}% below required {:.0}%",
                    rate * 100.0,
                    self.requirements.min_pass_rate * 100.0
                ));
            }
        }

        // Check safety invariants
        for required in &self.requirements.required_invariants {
            let satisfied = skill
                .safety_checks
                .iter()
                .find(|c| &c.name == required)
                .map(|c| c.satisfied)
                .unwrap_or(false);

            if !satisfied {
                failures.push(format!("Safety invariant '{}' not satisfied", required));
            }
        }

        // Check falsification attempts
        if skill.falsification_log.len() < self.requirements.min_falsification_attempts {
            failures.push(format!(
                "Need {} falsification attempts, have {}",
                self.requirements.min_falsification_attempts,
                skill.falsification_log.len()
            ));
        }

        if !failures.is_empty() {
            return Err(failures);
        }

        // Determine target tier
        let target = match skill.tier {
            SkillTier::Candidate => SkillTier::Probation,
            SkillTier::Probation => SkillTier::Trusted,
            SkillTier::Trusted => SkillTier::Trusted,
        };

        Ok(target)
    }

    /// Promote a skill (if allowed)
    pub fn promote(&mut self, skill_id: &str) -> Result<SkillTier, Vec<String>> {
        let target = self.can_promote(skill_id)?;
        let skill = self.skills.get_mut(skill_id).unwrap();

        let event = PromotionEvent {
            from: skill.tier,
            to: target,
            reason: "Passed all promotion requirements".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        skill.promotion_history.push(event);
        skill.tier = target;

        Ok(target)
    }

    /// Demote a skill (e.g., after failure)
    pub fn demote(&mut self, skill_id: &str, reason: &str) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;

        let new_tier = match skill.tier {
            SkillTier::Trusted => SkillTier::Probation,
            SkillTier::Probation => SkillTier::Candidate,
            SkillTier::Candidate => SkillTier::Candidate,
        };

        let event = PromotionEvent {
            from: skill.tier,
            to: new_tier,
            reason: reason.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        skill.promotion_history.push(event);
        skill.tier = new_tier;

        Ok(())
    }

    /// Update a safety invariant check
    pub fn update_safety_check(
        &mut self,
        skill_id: &str,
        invariant_name: &str,
        satisfied: bool,
        error: Option<&str>,
    ) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;

        if let Some(check) = skill.safety_checks.iter_mut().find(|c| c.name == invariant_name) {
            check.satisfied = satisfied;
            check.error = error.map(|s| s.to_string());
        }

        Ok(())
    }
}

/// Acceptance test: a generated procedure that would break a constraint
/// must fail promotion even if it "works"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_cannot_run_on_host() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["rm -rf /tmp/test".to_string()]);

        let result = gate.can_use_on_host(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sandbox only"));
    }

    #[test]
    fn test_promotion_requires_tests() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        let result = gate.can_promote(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("sandbox tests")));
    }

    #[test]
    fn test_promotion_requires_rollback() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec![]); // No rollback

        let result = gate.can_promote(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("rollback")));
    }

    #[test]
    fn test_falsification_blocks_promotion() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Record a successful falsification (skill broke)
        gate.record_falsification(
            &id,
            FalsificationAttempt {
                description: "Tried different resolver".to_string(),
                broke_skill: true, // Skill failed!
                variations: vec!["resolver: 1.1.1.1".to_string()],
                result: "Command failed with different DNS".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        )
        .unwrap();

        // Should be blocked now
        assert!(gate.blocked.contains_key(&id));
        let result = gate.can_promote(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_tier_precedence() {
        assert!(SkillTier::Candidate < SkillTier::Probation);
        assert!(SkillTier::Probation < SkillTier::Trusted);
    }
}
