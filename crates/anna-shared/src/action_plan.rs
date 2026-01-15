//! ActionPlan - Structured executable plans for system changes.
//! Phase 16: Turn fallback into real execution.
//! Phase 17: Verification and rollback support.
//! Phase 25: Execution safety and reversibility hardening.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Preflight result - structured outcome of preflight check (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PreflightResult {
    /// Preflight passed, safe to proceed
    #[default]
    Passed,
    /// Preflight blocked, changes not needed (idempotent skip)
    Blocked,
    /// Preflight could not determine state (treat as blocked, outcome = Cancelled)
    Unknown,
}

/// Verification status - structured verification outcome (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    /// Verification passed - observable state change confirmed
    Passed,
    /// Verification failed - state change not observed
    Failed,
    /// Verification could not determine state (treat as failed)
    #[default]
    Unknown,
}

/// Reversibility classification (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// Action can be rolled back
    #[default]
    Reversible,
    /// Action cannot be rolled back - needs elevated confirmation
    NonReversible,
}

/// A single step in an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    /// Human-readable description of what this step does.
    pub description: String,
    /// The command to execute.
    pub command: String,
    /// Whether this command requires elevated privileges.
    pub needs_sudo: bool,
    /// Files that will be modified (for backup).
    pub affects_files: Vec<String>,
    /// Systemd units that will be modified (for state capture).
    pub affects_units: Vec<String>,
    /// Per-step verification command (quick check).
    pub verify_command: Option<String>,
    /// Expected pattern in verify output.
    pub verify_pattern: Option<String>,
    /// Rollback command if this step needs undoing.
    pub rollback_command: Option<String>,
}

impl ActionStep {
    /// Create a new step with minimal required fields.
    pub fn new(description: &str, command: &str, needs_sudo: bool) -> Self {
        Self {
            description: description.to_string(),
            command: command.to_string(),
            needs_sudo,
            affects_files: Vec::new(),
            affects_units: Vec::new(),
            verify_command: None,
            verify_pattern: None,
            rollback_command: None,
        }
    }

    /// Builder: Add affected files.
    pub fn with_files(mut self, files: &[&str]) -> Self {
        self.affects_files = files.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Builder: Add affected units.
    pub fn with_units(mut self, units: &[&str]) -> Self {
        self.affects_units = units.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Builder: Add per-step verification.
    pub fn with_verify(mut self, command: &str, pattern: &str) -> Self {
        self.verify_command = Some(command.to_string());
        self.verify_pattern = Some(pattern.to_string());
        self
    }

    /// Builder: Add rollback command.
    pub fn with_rollback(mut self, command: &str) -> Self {
        self.rollback_command = Some(command.to_string());
        self
    }
}

/// Verification check to confirm plan succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// Command to run to verify success.
    pub command: String,
    /// Expected output pattern (success if contains this).
    pub success_pattern: String,
    /// Description of what we're checking.
    pub description: String,
}

/// Rollback information for a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Whether rollback is possible.
    pub possible: bool,
    /// Reason if rollback not possible.
    pub reason: Option<String>,
    /// Explicit rollback steps (if not using captured state).
    pub steps: Vec<ActionStep>,
}

impl Default for RollbackInfo {
    fn default() -> Self {
        Self {
            possible: true,
            reason: None,
            steps: Vec::new(),
        }
    }
}

impl RollbackInfo {
    /// Phase 25: Get reversibility classification.
    pub fn reversibility(&self) -> Reversibility {
        if self.possible {
            Reversibility::Reversible
        } else {
            Reversibility::NonReversible
        }
    }
}

/// A complete action plan that Anna can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// Unique identifier for this plan.
    pub id: String,
    /// Short summary of what this plan does (shown to user).
    pub summary: String,
    /// Detailed explanation of the changes.
    pub explanation: String,
    /// The steps to execute.
    pub steps: Vec<ActionStep>,
    /// Final verification (authoritative check).
    pub verification: Option<VerificationCheck>,
    /// Rollback information.
    pub rollback: RollbackInfo,
    /// When this plan was created.
    pub created_at: DateTime<Utc>,
    /// The original question that triggered this plan.
    pub original_question: String,
    /// Whether preflight determined changes are needed.
    pub changes_needed: bool,
    /// Reason if no changes needed.
    pub skip_reason: Option<String>,
    /// Phase 25: Structured preflight result.
    #[serde(default)]
    pub preflight_result: PreflightResult,
    /// Phase 25: Reason if preflight blocked/unknown.
    #[serde(default)]
    pub preflight_reason: Option<String>,
}

impl ActionPlan {
    /// Create a new action plan.
    pub fn new(question: &str, summary: &str, explanation: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            summary: summary.to_string(),
            explanation: explanation.to_string(),
            steps: Vec::new(),
            verification: None,
            rollback: RollbackInfo::default(),
            created_at: Utc::now(),
            original_question: question.to_string(),
            changes_needed: true,
            skip_reason: None,
            preflight_result: PreflightResult::Passed,
            preflight_reason: None,
        }
    }

    /// Add a step to the plan (legacy method for compatibility).
    pub fn add_step(&mut self, description: &str, command: &str, needs_sudo: bool) {
        self.steps.push(ActionStep::new(description, command, needs_sudo));
    }

    /// Add a step with full configuration.
    pub fn add_step_full(&mut self, step: ActionStep) {
        self.steps.push(step);
    }

    /// Set final verification check.
    pub fn set_verification(&mut self, command: &str, success_pattern: &str, description: &str) {
        self.verification = Some(VerificationCheck {
            command: command.to_string(),
            success_pattern: success_pattern.to_string(),
            description: description.to_string(),
        });
    }

    /// Mark plan as no changes needed (idempotent).
    pub fn mark_no_changes(&mut self, reason: &str) {
        self.changes_needed = false;
        self.skip_reason = Some(reason.to_string());
        self.preflight_result = PreflightResult::Blocked;
        self.preflight_reason = Some(reason.to_string());
        self.steps.clear();
    }

    /// Phase 25: Mark plan as preflight unknown (cannot safely proceed).
    pub fn mark_preflight_unknown(&mut self, reason: &str) {
        self.changes_needed = false;
        self.skip_reason = Some(reason.to_string());
        self.preflight_result = PreflightResult::Unknown;
        self.preflight_reason = Some(reason.to_string());
        self.steps.clear();
    }

    /// Phase 25: Get reversibility classification.
    pub fn reversibility(&self) -> Reversibility {
        self.rollback.reversibility()
    }

    /// Mark rollback as not possible.
    pub fn set_no_rollback(&mut self, reason: &str) {
        self.rollback.possible = false;
        self.rollback.reason = Some(reason.to_string());
    }

    /// Check if any step requires sudo.
    pub fn requires_sudo(&self) -> bool {
        self.steps.iter().any(|s| s.needs_sudo)
    }

    /// Format the plan for user confirmation.
    pub fn format_for_confirmation(&self) -> String {
        if !self.changes_needed {
            return format!(
                "No changes needed. {}",
                self.skip_reason.as_deref().unwrap_or("Already configured.")
            );
        }

        let mut output = String::new();
        output.push_str(&format!("I'll {}.\n\n", self.summary.to_lowercase()));
        output.push_str(&self.explanation);
        output.push_str("\n\nSteps:\n");

        for (i, step) in self.steps.iter().enumerate() {
            let sudo_marker = if step.needs_sudo { " [sudo]" } else { "" };
            output.push_str(&format!("  {}. {}{}\n", i + 1, step.description, sudo_marker));
        }

        if self.requires_sudo() {
            output.push_str("\nRequires administrator privileges.\n");
        }

        if !self.rollback.possible {
            if let Some(ref reason) = self.rollback.reason {
                output.push_str(&format!("\nNote: {}\n", reason));
            }
        }

        output.push_str("\nProceed? (yes/no)");
        output
    }
}

/// Result of executing an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionResult {
    pub plan_id: String,
    pub success: bool,
    pub step_results: Vec<StepResult>,
    pub verification_result: Option<VerificationResult>,
    /// Phase 25: Structured verification status.
    #[serde(default)]
    pub verification_status: VerificationStatus,
    pub rollback_performed: bool,
    pub rollback_success: Option<bool>,
    pub completed_at: DateTime<Utc>,
}

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub verified: Option<bool>,
}

/// Result of verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub actual_output: String,
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_step_builder() {
        let step = ActionStep::new("Test", "echo test", false)
            .with_files(&["/etc/test.conf"])
            .with_units(&["test.service"])
            .with_verify("test -f /etc/test.conf", "")
            .with_rollback("rm /etc/test.conf");

        assert_eq!(step.affects_files.len(), 1);
        assert_eq!(step.affects_units.len(), 1);
        assert!(step.rollback_command.is_some());
    }

    #[test]
    fn test_plan_no_changes() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.mark_no_changes("Already configured");

        assert!(!plan.changes_needed);
        assert!(plan.steps.is_empty());
        assert!(plan.format_for_confirmation().contains("No changes needed"));
    }

    #[test]
    fn test_plan_no_rollback() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.set_no_rollback("Destructive operation");

        assert!(!plan.rollback.possible);
    }

    #[test]
    fn test_phase25_preflight_result() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        assert_eq!(plan.preflight_result, PreflightResult::Passed);

        plan.mark_no_changes("Already configured");
        assert_eq!(plan.preflight_result, PreflightResult::Blocked);

        let mut plan2 = ActionPlan::new("test", "Test", "Testing");
        plan2.mark_preflight_unknown("Cannot determine state");
        assert_eq!(plan2.preflight_result, PreflightResult::Unknown);
    }

    #[test]
    fn test_phase25_reversibility() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        assert_eq!(plan.reversibility(), Reversibility::Reversible);

        plan.set_no_rollback("Destructive operation");
        assert_eq!(plan.reversibility(), Reversibility::NonReversible);
    }

    #[test]
    fn test_phase25_verification_status_serialization() {
        let status = VerificationStatus::Passed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"passed\"");

        let status: VerificationStatus = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(status, VerificationStatus::Unknown);
    }
}
