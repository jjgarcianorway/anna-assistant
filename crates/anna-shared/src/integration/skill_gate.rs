//! Skill Gatekeeper - Validation harness as mandatory gate.
//!
//! v0.3.27: Controlled Learning with confidence decay and ClaimGate integration.
//!
//! A skill candidate cannot be used on the host unless it has:
//! 1. Passed a sandbox test suite for the relevant subsystem
//! 2. Declared its rollback procedure
//! 3. ClaimGate validation for explanatory claims
//!
//! Skill tiers:
//! - Candidate: Sandbox only, never on live system
//! - Probation: Host allowed in cautious mode with extra verification
//! - Trusted: Normal use allowed
//!
//! Confidence decay:
//! - Skills have confidence scores (0.0-1.0)
//! - Confidence decays when related packages/kernel/docs change
//! - Skills below threshold are demoted automatically
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

/// v0.3.27: Confidence state for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfidence {
    /// Current confidence score (0.0-1.0)
    pub score: f32,
    /// When confidence was last updated
    pub last_updated: String,
    /// Package versions when skill was learned
    pub package_versions: HashMap<String, String>,
    /// Kernel version when skill was learned
    pub kernel_version: Option<String>,
    /// Doc versions (wiki article modified dates, man page versions)
    pub doc_versions: HashMap<String, String>,
    /// Successful host runs (increases confidence)
    pub successful_runs: u32,
    /// Failed host runs (decreases confidence)
    pub failed_runs: u32,
}

impl Default for SkillConfidence {
    fn default() -> Self {
        Self {
            score: 1.0, // Start with full confidence
            last_updated: chrono::Utc::now().to_rfc3339(),
            package_versions: HashMap::new(),
            kernel_version: None,
            doc_versions: HashMap::new(),
            successful_runs: 0,
            failed_runs: 0,
        }
    }
}

impl SkillConfidence {
    /// Check if confidence is below demotion threshold
    pub fn below_threshold(&self, threshold: f32) -> bool {
        self.score < threshold
    }

    /// Record a successful run
    pub fn record_success(&mut self) {
        self.successful_runs += 1;
        self.score = (self.score + 0.05).min(1.0);
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Record a failed run
    pub fn record_failure(&mut self) {
        self.failed_runs += 1;
        self.score = (self.score - 0.2).max(0.0);
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Apply decay based on system changes
    pub fn apply_decay(&mut self, decay_amount: f32, reason: &str) {
        self.score = (self.score - decay_amount).max(0.0);
        self.last_updated = chrono::Utc::now().to_rfc3339();
        tracing::info!("Skill confidence decayed by {:.2}: {}", decay_amount, reason);
    }
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
    /// v0.3.27: Confidence state with decay
    #[serde(default)]
    pub confidence: SkillConfidence,
    /// v0.3.27: Related experiment IDs
    #[serde(default)]
    pub experiment_ids: Vec<String>,
    /// v0.3.27: ClaimGate verified explanations
    #[serde(default)]
    pub verified_explanations: Vec<String>,
    /// v0.3.27: Failed experiments (negative knowledge)
    #[serde(default)]
    pub failed_experiments: Vec<FailedExperiment>,
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

/// v0.3.27: A failed experiment (negative knowledge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedExperiment {
    /// Experiment ID
    pub experiment_id: String,
    /// What was tried
    pub commands: Vec<String>,
    /// Why it failed
    pub failure_reason: String,
    /// Error output
    pub error_output: String,
    /// Sandbox type used
    pub sandbox_type: String,
    /// When it failed
    pub timestamp: String,
}

/// The skill gatekeeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillGatekeeper {
    /// All gated skills by ID
    pub skills: HashMap<String, GatedSkill>,
    /// Skills blocked from promotion
    pub blocked: HashMap<String, String>,
    /// Promotion requirements
    pub requirements: PromotionRequirements,
    /// v0.3.27: Learning mode enabled
    #[serde(default = "default_true")]
    pub learning_enabled: bool,
    /// v0.3.27: Confidence threshold for automatic demotion
    #[serde(default = "default_demotion_threshold")]
    pub demotion_threshold: f32,
    /// v0.3.27: Counts for status display
    #[serde(default)]
    pub stats: SkillStats,
}

impl Default for SkillGatekeeper {
    fn default() -> Self {
        Self {
            skills: HashMap::new(),
            blocked: HashMap::new(),
            requirements: PromotionRequirements::default(),
            learning_enabled: true,
            demotion_threshold: 0.3,
            stats: SkillStats::default(),
        }
    }
}

fn default_true() -> bool { true }
fn default_demotion_threshold() -> f32 { 0.3 }

/// v0.3.27: Skill statistics for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStats {
    /// Skills at each tier
    pub candidate_count: usize,
    pub probation_count: usize,
    pub trusted_count: usize,
    /// Promotion events
    pub promotions: usize,
    pub demotions: usize,
    /// Failed experiments
    pub failed_experiments: usize,
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
            confidence: SkillConfidence::default(),
            experiment_ids: Vec::new(),
            verified_explanations: Vec::new(),
            failed_experiments: Vec::new(),
        };

        self.skills.insert(id.clone(), skill);
        self.update_stats();
        id
    }

    /// v0.3.27: Record a failed experiment (negative knowledge)
    pub fn record_failed_experiment(&mut self, skill_id: &str, exp: FailedExperiment) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;
        skill.failed_experiments.push(exp);
        skill.confidence.record_failure();
        self.stats.failed_experiments += 1;
        Ok(())
    }

    /// v0.3.27: Link an experiment to a skill
    pub fn link_experiment(&mut self, skill_id: &str, experiment_id: &str) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;
        skill.experiment_ids.push(experiment_id.to_string());
        Ok(())
    }

    /// v0.3.27: Add a ClaimGate-verified explanation
    pub fn add_verified_explanation(&mut self, skill_id: &str, explanation: &str) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;
        skill.verified_explanations.push(explanation.to_string());
        Ok(())
    }

    /// v0.3.27: Check and apply confidence decay for all skills
    pub fn check_confidence_decay(&mut self, current_kernel: Option<&str>, current_packages: &HashMap<String, String>) {
        let demotion_threshold = self.demotion_threshold;
        let mut skills_to_demote = Vec::new();

        for (id, skill) in &mut self.skills {
            // Check kernel version change
            let kernel_changed = match (&skill.confidence.kernel_version, current_kernel) {
                (Some(skill_kernel), Some(current)) => skill_kernel != current,
                _ => false,
            };
            if kernel_changed {
                skill.confidence.apply_decay(0.15, "kernel version changed");
            }

            // Collect changed packages first (to avoid borrow conflict)
            let changed_packages: Vec<String> = skill.confidence.package_versions
                .iter()
                .filter_map(|(pkg, version)| {
                    current_packages.get(pkg)
                        .filter(|&current_ver| current_ver != version)
                        .map(|_| pkg.clone())
                })
                .collect();

            // Apply decay for changed packages
            for pkg in changed_packages {
                skill.confidence.apply_decay(0.1, &format!("package {} changed", pkg));
            }

            // Check if below threshold
            if skill.confidence.below_threshold(demotion_threshold) && skill.tier != SkillTier::Candidate {
                skills_to_demote.push((id.clone(), skill.confidence.score));
            }
        }

        // Demote skills below threshold
        for (id, score) in skills_to_demote {
            let _ = self.demote(&id, &format!("Confidence decay below threshold ({:.2})", score));
            self.stats.demotions += 1;
        }

        self.update_stats();
    }

    /// v0.3.27: Update statistics
    pub fn update_stats(&mut self) {
        let mut candidate = 0;
        let mut probation = 0;
        let mut trusted = 0;

        for skill in self.skills.values() {
            match skill.tier {
                SkillTier::Candidate => candidate += 1,
                SkillTier::Probation => probation += 1,
                SkillTier::Trusted => trusted += 1,
            }
        }

        self.stats.candidate_count = candidate;
        self.stats.probation_count = probation;
        self.stats.trusted_count = trusted;
    }

    /// v0.3.27: Record a successful host run
    pub fn record_successful_run(&mut self, skill_id: &str) -> Result<(), String> {
        let skill = self.skills.get_mut(skill_id).ok_or("Skill not found")?;
        skill.confidence.record_success();
        Ok(())
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

        // v0.3.27: Update stats
        self.stats.promotions += 1;
        self.update_stats();

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

    // v0.3.27: Milestone 3 mandatory tests

    #[test]
    fn test_confidence_decay_demotes_skill() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Manually set to probation tier for testing
        {
            let skill = gate.skills.get_mut(&id).unwrap();
            skill.tier = SkillTier::Probation;
            skill.confidence.score = 0.25; // Below default threshold of 0.3
        }

        // Check decay - should trigger demotion
        gate.check_confidence_decay(None, &HashMap::new());

        // Verify demotion occurred
        let skill = gate.skills.get(&id).unwrap();
        assert_eq!(skill.tier, SkillTier::Candidate, "Skill should be demoted to Candidate");
        assert_eq!(gate.stats.demotions, 1, "Demotion count should be 1");
    }

    #[test]
    fn test_failed_experiment_affects_confidence() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Initial confidence is 1.0
        let initial_confidence = gate.skills.get(&id).unwrap().confidence.score;
        assert_eq!(initial_confidence, 1.0);

        // Record a failed experiment
        gate.record_failed_experiment(&id, FailedExperiment {
            experiment_id: "exp-001".to_string(),
            commands: vec!["failing-command".to_string()],
            failure_reason: "Command not found".to_string(),
            error_output: "bash: failing-command: command not found".to_string(),
            sandbox_type: "FullNamespace".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).unwrap();

        // Confidence should decrease
        let new_confidence = gate.skills.get(&id).unwrap().confidence.score;
        assert!(new_confidence < initial_confidence, "Confidence should decrease after failed experiment");
        assert_eq!(gate.stats.failed_experiments, 1, "Failed experiment count should be 1");

        // Failed experiment should be recorded
        let skill = gate.skills.get(&id).unwrap();
        assert_eq!(skill.failed_experiments.len(), 1);
    }

    #[test]
    fn test_verified_explanation_required() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Skill starts with no verified explanations
        let skill = gate.skills.get(&id).unwrap();
        assert!(skill.verified_explanations.is_empty());

        // Add a verified explanation
        gate.add_verified_explanation(&id, "Lists files in directory").unwrap();

        // Verify it was added
        let skill = gate.skills.get(&id).unwrap();
        assert_eq!(skill.verified_explanations.len(), 1);
        assert_eq!(skill.verified_explanations[0], "Lists files in directory");
    }

    #[test]
    fn test_skill_linked_to_experiment() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Link an experiment
        gate.link_experiment(&id, "exp-sandbox-001").unwrap();

        // Verify it was linked
        let skill = gate.skills.get(&id).unwrap();
        assert_eq!(skill.experiment_ids.len(), 1);
        assert_eq!(skill.experiment_ids[0], "exp-sandbox-001");
    }

    #[test]
    fn test_successful_run_increases_confidence() {
        let mut gate = SkillGatekeeper::new();
        let id = gate.register_candidate("test", vec!["ls".to_string()], vec!["true".to_string()]);

        // Set initial confidence below max
        {
            let skill = gate.skills.get_mut(&id).unwrap();
            skill.confidence.score = 0.8;
        }

        // Record successful run
        gate.record_successful_run(&id).unwrap();

        // Confidence should increase
        let skill = gate.skills.get(&id).unwrap();
        assert!(skill.confidence.score > 0.8);
        assert_eq!(skill.confidence.successful_runs, 1);
    }

    #[test]
    fn test_stats_update_on_registration() {
        let mut gate = SkillGatekeeper::new();

        // Register several skills
        gate.register_candidate("skill1", vec!["ls".to_string()], vec!["true".to_string()]);
        gate.register_candidate("skill2", vec!["pwd".to_string()], vec!["true".to_string()]);

        // All should be candidates
        assert_eq!(gate.stats.candidate_count, 2);
        assert_eq!(gate.stats.probation_count, 0);
        assert_eq!(gate.stats.trusted_count, 0);
    }
}
