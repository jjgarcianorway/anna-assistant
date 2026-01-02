//! Clarification response types.

use serde::{Deserialize, Serialize};

/// Response to a clarification request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationResponse {
    /// The value provided.
    pub value: String,
    /// Whether this was the default.
    pub is_default: bool,
    /// Additional notes from user.
    pub notes: Option<String>,
}

impl ClarificationResponse {
    /// Create a response with a value.
    pub fn with_value(value: &str) -> Self {
        Self {
            value: value.to_string(),
            is_default: false,
            notes: None,
        }
    }

    /// Create a default response.
    pub fn default_value(value: &str) -> Self {
        Self {
            value: value.to_string(),
            is_default: true,
            notes: None,
        }
    }

    /// Create a response with notes.
    pub fn with_notes(value: &str, notes: &str) -> Self {
        Self {
            value: value.to_string(),
            is_default: false,
            notes: Some(notes.to_string()),
        }
    }
}
