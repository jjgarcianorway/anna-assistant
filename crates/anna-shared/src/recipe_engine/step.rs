//! Recipe step definitions and implementations.

use super::types::RecipeStepType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step identifier (unique within recipe)
    pub id: String,
    /// What kind of step this is
    pub kind: RecipeStepType,
    /// Human-readable description
    pub description: String,
    /// Step-specific parameters (serialized)
    pub params: HashMap<String, String>,
    /// Step IDs that must complete first
    pub depends_on: Vec<String>,
}

impl RecipeStep {
    /// Create a new run_probe step
    pub fn probe(id: &str, probe_id: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("probe_id".to_string(), probe_id.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RunProbe,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Create a new run_command step
    pub fn command(id: &str, cmd: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("command".to_string(), cmd.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RunCommand,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Create a new render_answer step
    pub fn render(id: &str, template: &str, description: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("template".to_string(), template.to_string());
        Self {
            id: id.to_string(),
            kind: RecipeStepType::RenderAnswer,
            description: description.to_string(),
            params,
            depends_on: vec![],
        }
    }

    /// Add dependency
    pub fn depends(mut self, step_id: &str) -> Self {
        self.depends_on.push(step_id.to_string());
        self
    }

    /// Check if step is read-only
    pub fn is_read_only(&self) -> bool {
        matches!(
            self.kind,
            RecipeStepType::RunProbe
                | RecipeStepType::CheckCondition
                | RecipeStepType::RenderAnswer
        )
    }

    /// Check if step requires user confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self.kind,
            RecipeStepType::EditFile | RecipeStepType::RunCommand
        ) && !self.is_safe_command()
    }

    /// Check if command is in the safe list
    fn is_safe_command(&self) -> bool {
        if self.kind != RecipeStepType::RunCommand {
            return true;
        }
        let cmd = self.params.get("command").map(|s| s.as_str()).unwrap_or("");
        // Safe read-only commands
        let safe_prefixes = [
            "cat ",
            "head ",
            "tail ",
            "ls ",
            "df ",
            "free ",
            "ps ",
            "top -bn1",
            "systemctl status",
            "systemctl is-",
            "journalctl ",
            "lsblk",
            "lscpu",
            "ip addr",
            "ip link",
            "ip route",
            "ss -",
            "pacman -Q",
            "which ",
            "echo ",
            "printf ",
            "test ",
            "stat ",
            "file ",
            "wc ",
            "grep ",
        ];
        safe_prefixes.iter().any(|p| cmd.starts_with(p))
    }
}
