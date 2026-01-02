//! Action definitions for SRC v1.

use serde::{Deserialize, Serialize};

use super::types::SrcActionType;

/// A proposed action in SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcAction {
    /// Action type.
    #[serde(rename = "type")]
    pub action_type: SrcActionType,
    /// Short title.
    pub title: String,
    /// Shell command to execute, or null.
    #[serde(default)]
    pub command: Option<String>,
    /// Why this action is needed.
    pub why: String,
    /// Expected outcome.
    pub expected: String,
    /// Rollback command if change fails, or null.
    #[serde(default)]
    pub rollback: Option<String>,
}

impl SrcAction {
    /// Create a probe action.
    pub fn probe(title: &str, command: &str, why: &str, expected: &str) -> Self {
        Self {
            action_type: SrcActionType::Probe,
            title: title.to_string(),
            command: Some(command.to_string()),
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: None,
        }
    }

    /// Create an explain action.
    pub fn explain(title: &str, why: &str, expected: &str) -> Self {
        Self {
            action_type: SrcActionType::Explain,
            title: title.to_string(),
            command: None,
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: None,
        }
    }

    /// Create a change action.
    pub fn change(
        title: &str,
        command: &str,
        why: &str,
        expected: &str,
        rollback: Option<&str>,
    ) -> Self {
        Self {
            action_type: SrcActionType::Change,
            title: title.to_string(),
            command: Some(command.to_string()),
            why: why.to_string(),
            expected: expected.to_string(),
            rollback: rollback.map(String::from),
        }
    }

    /// Validate the action.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.is_empty() {
            errors.push("action title cannot be empty".to_string());
        }
        if self.why.is_empty() {
            errors.push("action why cannot be empty".to_string());
        }
        if self.expected.is_empty() {
            errors.push("action expected cannot be empty".to_string());
        }

        // Probe and Change need commands
        if matches!(
            self.action_type,
            SrcActionType::Probe | SrcActionType::Change
        ) {
            if self.command.is_none() {
                errors.push(format!("{:?} action needs a command", self.action_type));
            }
        }

        // Risky changes should have rollback
        if self.action_type == SrcActionType::Change && self.rollback.is_none() {
            // Warning, not error - some changes can't be rolled back
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_action_probe() {
        let action = SrcAction::probe(
            "Check boot time",
            "systemd-analyze",
            "Need boot breakdown",
            "Time breakdown",
        );
        assert!(action.validate().is_ok());
        assert_eq!(action.action_type, SrcActionType::Probe);
    }
}
