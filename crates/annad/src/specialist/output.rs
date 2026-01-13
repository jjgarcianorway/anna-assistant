//! Specialist Output - Structured results from specialist execution.
//!
//! CRITICAL: Specialists return structured data, NOT user-facing text.
//! The display layer renders this into human-readable output.

use serde::{Deserialize, Serialize};

/// Structured output from specialist execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialistOutput {
    /// Work completed successfully.
    Completed {
        specialist_id: String,
        specialist_name: String,
        commands_executed: Vec<String>,
        outputs: Vec<String>,
        confidence: f32,
        recipe_used: Option<String>,
        /// If true, this resolution should be learned as a recipe.
        should_learn: bool,
    },

    /// Specialist needs helper tools installed.
    NeedsHelpers {
        specialist_id: String,
        missing: Vec<String>,
    },

    /// Specialist cannot handle, escalation needed.
    NeedsEscalation {
        specialist_id: String,
        reason: String,
    },

    /// Execution failed.
    Failed {
        specialist_id: String,
        reason: String,
        can_escalate: bool,
    },
}

impl SpecialistOutput {
    /// Check if this output should trigger recipe learning.
    pub fn should_learn_recipe(&self) -> bool {
        matches!(
            self,
            SpecialistOutput::Completed {
                should_learn: true,
                ..
            }
        )
    }

    /// Get confidence if available.
    pub fn confidence(&self) -> Option<f32> {
        match self {
            SpecialistOutput::Completed { confidence, .. } => Some(*confidence),
            _ => None,
        }
    }

    /// Check if escalation is possible.
    pub fn can_escalate(&self) -> bool {
        matches!(
            self,
            SpecialistOutput::NeedsEscalation { .. }
                | SpecialistOutput::Failed {
                    can_escalate: true,
                    ..
                }
        )
    }

    /// Get specialist ID.
    pub fn specialist_id(&self) -> &str {
        match self {
            SpecialistOutput::Completed { specialist_id, .. }
            | SpecialistOutput::NeedsHelpers { specialist_id, .. }
            | SpecialistOutput::NeedsEscalation { specialist_id, .. }
            | SpecialistOutput::Failed { specialist_id, .. } => specialist_id,
        }
    }

    /// Check if output indicates success.
    pub fn is_success(&self) -> bool {
        matches!(self, SpecialistOutput::Completed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completed_output() {
        let output = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec!["df -h".to_string()],
            outputs: vec!["Filesystem      Size  Used Avail Use%".to_string()],
            confidence: 0.9,
            recipe_used: Some("disk-usage".to_string()),
            should_learn: false,
        };
        assert!(output.is_success());
        assert_eq!(output.confidence(), Some(0.9));
        assert!(!output.should_learn_recipe());
    }

    #[test]
    fn test_needs_helpers() {
        let output = SpecialistOutput::NeedsHelpers {
            specialist_id: "audio-jr".to_string(),
            missing: vec!["wpctl".to_string()],
        };
        assert!(!output.is_success());
        assert!(!output.can_escalate());
    }

    #[test]
    fn test_needs_escalation() {
        let output = SpecialistOutput::NeedsEscalation {
            specialist_id: "net-jr".to_string(),
            reason: "Complex routing issue".to_string(),
        };
        assert!(output.can_escalate());
    }

    #[test]
    fn test_failed_can_escalate() {
        let output = SpecialistOutput::Failed {
            specialist_id: "stor-jr".to_string(),
            reason: "Permission denied".to_string(),
            can_escalate: true,
        };
        assert!(output.can_escalate());
    }

    #[test]
    fn test_failed_cannot_escalate() {
        let output = SpecialistOutput::Failed {
            specialist_id: "stor-sr".to_string(),
            reason: "Disk not found".to_string(),
            can_escalate: false,
        };
        assert!(!output.can_escalate());
    }

    #[test]
    fn test_should_learn_recipe() {
        let learn = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec!["free -h".to_string()],
            outputs: vec!["mem output".to_string()],
            confidence: 0.95,
            recipe_used: None,
            should_learn: true,
        };
        assert!(learn.should_learn_recipe());

        let no_learn = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec!["free -h".to_string()],
            outputs: vec!["mem output".to_string()],
            confidence: 0.6,
            recipe_used: Some("memory-usage".to_string()),
            should_learn: false,
        };
        assert!(!no_learn.should_learn_recipe());
    }
}
