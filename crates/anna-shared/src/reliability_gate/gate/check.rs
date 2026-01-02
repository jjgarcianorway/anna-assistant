//! Individual gate check results.

use serde::{Deserialize, Serialize};

/// Individual gate check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    /// Check name
    pub name: String,
    /// Whether it passed
    pub passed: bool,
    /// Details if failed
    pub details: Option<String>,
}

impl GateCheck {
    /// Create a passing check.
    pub fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            details: None,
        }
    }

    /// Create a failing check.
    pub fn fail(name: &str, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            details: Some(details.to_string()),
        }
    }
}
