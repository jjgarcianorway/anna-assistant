//! Recipe steps for execution (v0.0.420).

use serde::{Deserialize, Serialize};

/// Kind of recipe step (risk level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStepKind {
    /// Gather info, no changes
    #[default]
    ProbeOnly,
    /// Can auto-confirm if policy allows
    SafeChange,
    /// Always ask confirmation
    RiskyChange,
    /// No action, explanation to user
    ExplanationOnly,
}

impl RecipeStepKind {
    /// Check if this step requires user confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, RecipeStepKind::RiskyChange)
    }

    /// Check if this step makes changes
    pub fn is_mutating(&self) -> bool {
        matches!(self, RecipeStepKind::SafeChange | RecipeStepKind::RiskyChange)
    }
}

/// Action specification for recipe steps
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeStepAction {
    /// Action type: "run_command", "backup_file", "append_line", "ensure_line", "explain"
    #[serde(rename = "type")]
    pub type_: String,
    /// Command to run (for run_command)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Arguments (for run_command)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// File path (for file operations)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Content (for append_line, ensure_line)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Template for explanation (for explain)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Variables to extract from probe output
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<String>,
}

impl RecipeStepAction {
    /// Create a run_command action
    pub fn run_command(cmd: &str) -> Self {
        Self {
            type_: "run_command".to_string(),
            command: Some(cmd.to_string()),
            ..Default::default()
        }
    }

    /// Create a run_command action with arguments
    pub fn run_command_with_args(cmd: &str, args: Vec<&str>) -> Self {
        Self {
            type_: "run_command".to_string(),
            command: Some(cmd.to_string()),
            args: Some(args.into_iter().map(String::from).collect()),
            ..Default::default()
        }
    }

    /// Create a backup_file action
    pub fn backup_file(path: &str) -> Self {
        Self {
            type_: "backup_file".to_string(),
            path: Some(path.to_string()),
            ..Default::default()
        }
    }

    /// Create an append_line action
    pub fn append_line(path: &str, content: &str) -> Self {
        Self {
            type_: "append_line".to_string(),
            path: Some(path.to_string()),
            content: Some(content.to_string()),
            ..Default::default()
        }
    }

    /// Create an ensure_line action
    pub fn ensure_line(path: &str, content: &str) -> Self {
        Self {
            type_: "ensure_line".to_string(),
            path: Some(path.to_string()),
            content: Some(content.to_string()),
            ..Default::default()
        }
    }

    /// Create an explain action
    pub fn explain(template: &str) -> Self {
        Self {
            type_: "explain".to_string(),
            template: Some(template.to_string()),
            ..Default::default()
        }
    }

    /// Create an explain action that extracts data from probe output
    pub fn explain_with_extract(template: &str, extract: &str) -> Self {
        Self {
            type_: "explain".to_string(),
            template: Some(template.to_string()),
            extract: Some(extract.to_string()),
            ..Default::default()
        }
    }
}

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStepV2 {
    /// Risk level of this step
    #[serde(default)]
    pub kind: RecipeStepKind,
    /// Human-readable description
    pub description: String,
    /// The action to perform
    pub action: RecipeStepAction,
}

impl RecipeStepV2 {
    /// Create a probe step
    pub fn probe(description: &str, command: &str) -> Self {
        Self {
            kind: RecipeStepKind::ProbeOnly,
            description: description.to_string(),
            action: RecipeStepAction::run_command(command),
        }
    }

    /// Create a safe change step
    pub fn safe_change(description: &str, action: RecipeStepAction) -> Self {
        Self {
            kind: RecipeStepKind::SafeChange,
            description: description.to_string(),
            action,
        }
    }

    /// Create a risky change step (requires confirmation)
    pub fn risky_change(description: &str, action: RecipeStepAction) -> Self {
        Self {
            kind: RecipeStepKind::RiskyChange,
            description: description.to_string(),
            action,
        }
    }

    /// Create an explanation step
    pub fn explain(description: &str, template: &str) -> Self {
        Self {
            kind: RecipeStepKind::ExplanationOnly,
            description: description.to_string(),
            action: RecipeStepAction::explain(template),
        }
    }

    /// Create an explanation with extracted data
    pub fn explain_with_extract(description: &str, template: &str, extract: &str) -> Self {
        Self {
            kind: RecipeStepKind::ExplanationOnly,
            description: description.to_string(),
            action: RecipeStepAction::explain_with_extract(template, extract),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_kind() {
        assert!(!RecipeStepKind::ProbeOnly.requires_confirmation());
        assert!(!RecipeStepKind::SafeChange.requires_confirmation());
        assert!(RecipeStepKind::RiskyChange.requires_confirmation());
        assert!(!RecipeStepKind::ExplanationOnly.requires_confirmation());

        assert!(!RecipeStepKind::ProbeOnly.is_mutating());
        assert!(RecipeStepKind::SafeChange.is_mutating());
        assert!(RecipeStepKind::RiskyChange.is_mutating());
        assert!(!RecipeStepKind::ExplanationOnly.is_mutating());
    }
}
