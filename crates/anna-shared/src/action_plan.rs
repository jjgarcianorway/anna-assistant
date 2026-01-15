//! ActionPlan - Structured executable plans for system changes.
//! Phase 16: Turn fallback into real execution.
//!
//! When Anna would have given manual instructions (blocked by sanitization),
//! she instead generates an ActionPlan that she can execute herself.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A single step in an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    /// Human-readable description of what this step does.
    pub description: String,
    /// The command to execute.
    pub command: String,
    /// Whether this command requires elevated privileges.
    pub needs_sudo: bool,
    /// Expected output pattern (for verification).
    pub expected_output: Option<String>,
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
    /// How to verify the plan succeeded.
    pub verification: Option<VerificationCheck>,
    /// Steps to undo the changes (for Phase 17).
    pub rollback: Option<Vec<ActionStep>>,
    /// When this plan was created.
    pub created_at: DateTime<Utc>,
    /// The original question that triggered this plan.
    pub original_question: String,
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

/// Result of executing an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionResult {
    /// The plan that was executed.
    pub plan_id: String,
    /// Whether all steps succeeded.
    pub success: bool,
    /// Results for each step.
    pub step_results: Vec<StepResult>,
    /// Verification result if verification was defined.
    pub verification_result: Option<VerificationResult>,
    /// When execution completed.
    pub completed_at: DateTime<Utc>,
}

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Index of the step.
    pub step_index: usize,
    /// Whether the step succeeded.
    pub success: bool,
    /// Command output.
    pub output: String,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Result of verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed.
    pub passed: bool,
    /// Actual output from verification command.
    pub actual_output: String,
    /// Explanation of result.
    pub explanation: String,
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
            rollback: None,
            created_at: Utc::now(),
            original_question: question.to_string(),
        }
    }

    /// Add a step to the plan.
    pub fn add_step(&mut self, description: &str, command: &str, needs_sudo: bool) {
        self.steps.push(ActionStep {
            description: description.to_string(),
            command: command.to_string(),
            needs_sudo,
            expected_output: None,
        });
    }

    /// Set verification check.
    pub fn set_verification(&mut self, command: &str, success_pattern: &str, description: &str) {
        self.verification = Some(VerificationCheck {
            command: command.to_string(),
            success_pattern: success_pattern.to_string(),
            description: description.to_string(),
        });
    }

    /// Check if any step requires sudo.
    pub fn requires_sudo(&self) -> bool {
        self.steps.iter().any(|s| s.needs_sudo)
    }

    /// Format the plan for user confirmation.
    pub fn format_for_confirmation(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("I'll {}.\n\n", self.summary.to_lowercase()));
        output.push_str(&self.explanation);
        output.push_str("\n\nSteps:\n");

        for (i, step) in self.steps.iter().enumerate() {
            let sudo_marker = if step.needs_sudo { " [requires sudo]" } else { "" };
            output.push_str(&format!("  {}. {}{}\n", i + 1, step.description, sudo_marker));
        }

        if self.requires_sudo() {
            output.push_str("\nThis will require administrator privileges.\n");
        }

        output.push_str("\nProceed? (yes/no)");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_plan_creation() {
        let mut plan = ActionPlan::new(
            "disable sleep",
            "Disable system sleep",
            "This will configure systemd to prevent automatic sleep.",
        );

        plan.add_step(
            "Mask sleep targets",
            "systemctl mask sleep.target suspend.target",
            true,
        );

        plan.set_verification(
            "systemctl status sleep.target",
            "masked",
            "Verify sleep target is masked",
        );

        assert_eq!(plan.steps.len(), 1);
        assert!(plan.requires_sudo());
        assert!(plan.verification.is_some());
    }

    #[test]
    fn test_format_for_confirmation() {
        let mut plan = ActionPlan::new(
            "test",
            "Test operation",
            "This is a test.",
        );
        plan.add_step("Step one", "echo hello", false);
        plan.add_step("Step two", "sudo systemctl restart test", true);

        let formatted = plan.format_for_confirmation();
        // Summary is lowercased in "I'll {summary}."
        assert!(formatted.contains("test operation"));
        assert!(formatted.contains("Step one"));
        assert!(formatted.contains("[requires sudo]"));
        assert!(formatted.contains("Proceed? (yes/no)"));
    }
}
