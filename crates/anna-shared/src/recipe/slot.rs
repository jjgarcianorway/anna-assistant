//! Recipe slot and clarification prerequisite types (v0.0.177).

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// Slot definition for clarification template recipes (v0.0.31)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeSlot {
    /// Slot name (e.g., "editor_name", "config_path")
    pub name: String,
    /// Question ID to use
    pub question_id: String,
    /// Whether this slot is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Verification type (e.g., "binary", "unit", "mount")
    #[serde(default)]
    pub verify_type: String,
}

impl RecipeSlot {
    pub fn new(name: &str, question_id: &str) -> Self {
        Self {
            name: name.to_string(),
            question_id: question_id.to_string(),
            required: true,
            verify_type: String::new(),
        }
    }

    pub fn with_verify(mut self, verify_type: &str) -> Self {
        self.verify_type = verify_type.to_string();
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Prerequisite for recipe execution requiring clarification (v0.45.5)
/// When a recipe has a ClarifyPrereq, the system must ensure the fact
/// is known and verified before executing the recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarifyPrereq {
    /// The fact key that must be known (e.g., "preferred_editor")
    pub fact_key: String,
    /// Question ID to use if fact is unknown
    pub question_id: String,
    /// Must offer only installed/verified options
    #[serde(default = "default_true")]
    pub evidence_only: bool,
    /// Verification command template (e.g., "command -v {}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_template: Option<String>,
}

impl ClarifyPrereq {
    pub fn new(fact_key: impl Into<String>, question_id: impl Into<String>) -> Self {
        Self {
            fact_key: fact_key.into(),
            question_id: question_id.into(),
            evidence_only: true,
            verify_template: None,
        }
    }

    pub fn with_verify(mut self, template: impl Into<String>) -> Self {
        self.verify_template = Some(template.into());
        self
    }

    /// Create prereq for editor selection
    pub fn editor() -> Self {
        Self::new("preferred_editor", "editor_select").with_verify("command -v {}")
    }
}
