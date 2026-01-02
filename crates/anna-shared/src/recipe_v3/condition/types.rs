//! Recipe condition types (v0.0.423).
//!
//! Type definitions for recipe preconditions and postconditions.

use serde::{Deserialize, Serialize};

/// A recipe condition (precondition or postcondition)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeCondition {
    /// Probe must return truthy value
    ProbeTrue {
        /// Probe command to run
        probe: String,
        /// Expected output pattern (regex or substring)
        expected: Option<String>,
    },
    /// Command must exist in PATH
    CommandExists {
        /// Command name
        command: String,
    },
    /// Package must be installed
    PackageInstalled {
        /// Package name
        package: String,
    },
    /// File must exist
    FileExists {
        /// File path
        path: String,
    },
    /// File must not exist
    FileNotExists {
        /// File path
        path: String,
    },
    /// Config file must contain pattern
    ConfigContains {
        /// Config file path
        path: String,
        /// Pattern to find
        pattern: String,
    },
    /// Config file must not contain pattern
    ConfigNotContains {
        /// Config file path
        path: String,
        /// Pattern that must not exist
        pattern: String,
    },
    /// Service must be in specific state
    ServiceState {
        /// Service name
        service: String,
        /// Expected state (running, stopped, enabled, disabled)
        state: String,
    },
    /// Custom condition with description
    Custom {
        /// Description of the condition
        description: String,
        /// Probe command
        probe: String,
    },
}

impl RecipeCondition {
    /// Get a human-readable description
    pub fn describe(&self) -> String {
        match self {
            Self::ProbeTrue { probe, expected } => {
                if let Some(exp) = expected {
                    format!("Probe '{}' matches '{}'", probe, exp)
                } else {
                    format!("Probe '{}' succeeds", probe)
                }
            }
            Self::CommandExists { command } => format!("Command '{}' exists", command),
            Self::PackageInstalled { package } => format!("Package '{}' is installed", package),
            Self::FileExists { path } => format!("File '{}' exists", path),
            Self::FileNotExists { path } => format!("File '{}' does not exist", path),
            Self::ConfigContains { path, pattern } => {
                format!("Config '{}' contains '{}'", path, pattern)
            }
            Self::ConfigNotContains { path, pattern } => {
                format!("Config '{}' does not contain '{}'", path, pattern)
            }
            Self::ServiceState { service, state } => {
                format!("Service '{}' is {}", service, state)
            }
            Self::Custom { description, .. } => description.clone(),
        }
    }
}

/// Result of condition evaluation
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// Whether condition is satisfied
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Optional details (command output, etc.)
    pub details: Option<String>,
}

impl ConditionResult {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn fail(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_describe() {
        let cond = RecipeCondition::CommandExists {
            command: "vim".to_string(),
        };
        assert!(cond.describe().contains("vim"));

        let cond2 = RecipeCondition::FileExists {
            path: "/etc/hosts".to_string(),
        };
        assert!(cond2.describe().contains("/etc/hosts"));
    }
}
