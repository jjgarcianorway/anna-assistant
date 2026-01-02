//! Core types for clarification protocol.

use serde::{Deserialize, Serialize};

/// Types of clarification requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarificationType {
    /// Need to know which option the user prefers.
    Choice,
    /// Need a specific value or parameter.
    Value,
    /// Need confirmation before proceeding.
    Confirmation,
    /// Need more context about the situation.
    Context,
    /// Need to know the scope/target.
    Scope,
}

/// An option in a clarification request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    /// The value to use if selected.
    pub value: String,
    /// Human-readable label.
    pub label: String,
    /// Optional description.
    pub description: Option<String>,
}

impl ClarificationOption {
    /// Create a new option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            description: None,
        }
    }

    /// Create a simple option (value = label).
    pub fn simple(value: &str) -> Self {
        Self {
            value: value.to_string(),
            label: value.to_string(),
            description: None,
        }
    }

    /// Create an option with description.
    pub fn with_desc(value: &str, label: &str, description: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            description: Some(description.to_string()),
        }
    }

    /// Format for display.
    pub fn format(&self) -> String {
        match &self.description {
            Some(desc) => format!("{} - {}", self.label, desc),
            None => self.label.clone(),
        }
    }
}
