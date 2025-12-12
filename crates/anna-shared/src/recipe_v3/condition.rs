//! Recipe preconditions and postconditions (v0.0.423).
//!
//! Conditions that must be true before/after recipe execution.

use serde::{Deserialize, Serialize};
use std::process::Command;

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
    /// Evaluate the condition, returns (success, message)
    pub fn evaluate(
        &self,
        variables: &std::collections::HashMap<String, String>,
    ) -> ConditionResult {
        match self {
            Self::ProbeTrue { probe, expected } => {
                let probe = substitute_vars(probe, variables);
                eval_probe_true(&probe, expected.as_deref())
            }
            Self::CommandExists { command } => {
                let cmd = substitute_vars(command, variables);
                eval_command_exists(&cmd)
            }
            Self::PackageInstalled { package } => {
                let pkg = substitute_vars(package, variables);
                eval_package_installed(&pkg)
            }
            Self::FileExists { path } => {
                let p = substitute_vars(path, variables);
                eval_file_exists(&p)
            }
            Self::FileNotExists { path } => {
                let p = substitute_vars(path, variables);
                eval_file_not_exists(&p)
            }
            Self::ConfigContains { path, pattern } => {
                let p = substitute_vars(path, variables);
                let pat = substitute_vars(pattern, variables);
                eval_config_contains(&p, &pat)
            }
            Self::ConfigNotContains { path, pattern } => {
                let p = substitute_vars(path, variables);
                let pat = substitute_vars(pattern, variables);
                eval_config_not_contains(&p, &pat)
            }
            Self::ServiceState { service, state } => {
                let svc = substitute_vars(service, variables);
                let st = substitute_vars(state, variables);
                eval_service_state(&svc, &st)
            }
            Self::Custom { description, probe } => {
                let probe = substitute_vars(probe, variables);
                let result = eval_probe_true(&probe, None);
                ConditionResult {
                    success: result.success,
                    message: if result.success {
                        format!("Custom condition met: {}", description)
                    } else {
                        format!("Custom condition not met: {}", description)
                    },
                    details: result.details,
                }
            }
        }
    }

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

/// Substitute variables in a string
fn substitute_vars(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }
    result
}

/// Evaluate probe condition
fn eval_probe_true(probe: &str, expected: Option<&str>) -> ConditionResult {
    let output = Command::new("sh").args(["-c", probe]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let success = if let Some(exp) = expected {
                stdout.contains(exp) || out.status.success()
            } else {
                out.status.success()
            };
            ConditionResult {
                success,
                message: if success {
                    format!("Probe succeeded: {}", probe)
                } else {
                    format!("Probe failed: {}", probe)
                },
                details: Some(stdout),
            }
        }
        Err(e) => ConditionResult::fail(&format!("Failed to run probe: {}", e)),
    }
}

/// Check if command exists
fn eval_command_exists(command: &str) -> ConditionResult {
    let output = Command::new("which").arg(command).output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            ConditionResult::ok(&format!("Command '{}' found at {}", command, path))
        }
        _ => ConditionResult::fail(&format!("Command '{}' not found", command)),
    }
}

/// Check if package is installed
fn eval_package_installed(package: &str) -> ConditionResult {
    // Try pacman first (Arch)
    let output = Command::new("pacman").args(["-Q", package]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let info = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return ConditionResult::ok(&format!("Package installed: {}", info));
        }
    }

    ConditionResult::fail(&format!("Package '{}' not installed", package))
}

/// Check if file exists
fn eval_file_exists(path: &str) -> ConditionResult {
    if std::path::Path::new(path).exists() {
        ConditionResult::ok(&format!("File exists: {}", path))
    } else {
        ConditionResult::fail(&format!("File not found: {}", path))
    }
}

/// Check if file does not exist
fn eval_file_not_exists(path: &str) -> ConditionResult {
    if !std::path::Path::new(path).exists() {
        ConditionResult::ok(&format!("File does not exist: {}", path))
    } else {
        ConditionResult::fail(&format!("File exists (should not): {}", path))
    }
}

/// Check if config contains pattern
fn eval_config_contains(path: &str, pattern: &str) -> ConditionResult {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if content.contains(pattern) {
                ConditionResult::ok(&format!("Config '{}' contains pattern", path))
            } else {
                ConditionResult::fail(&format!("Pattern not found in '{}'", path))
            }
        }
        Err(e) => ConditionResult::fail(&format!("Cannot read '{}': {}", path, e)),
    }
}

/// Check if config does not contain pattern
fn eval_config_not_contains(path: &str, pattern: &str) -> ConditionResult {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if !content.contains(pattern) {
                ConditionResult::ok(&format!("Config '{}' does not contain pattern", path))
            } else {
                ConditionResult::fail(&format!("Unwanted pattern found in '{}'", path))
            }
        }
        Err(_) => {
            // File not existing means pattern not present
            ConditionResult::ok(&format!(
                "Config '{}' does not exist (pattern absent)",
                path
            ))
        }
    }
}

/// Check service state
fn eval_service_state(service: &str, expected_state: &str) -> ConditionResult {
    let state_lower = expected_state.to_lowercase();

    // Check active/inactive
    if state_lower == "running" || state_lower == "active" {
        let output = Command::new("systemctl")
            .args(["is-active", service])
            .output();

        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "active" {
                return ConditionResult::ok(&format!("Service '{}' is running", service));
            }
        }
        return ConditionResult::fail(&format!("Service '{}' is not running", service));
    }

    if state_lower == "stopped" || state_lower == "inactive" {
        let output = Command::new("systemctl")
            .args(["is-active", service])
            .output();

        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "inactive" {
                return ConditionResult::ok(&format!("Service '{}' is stopped", service));
            }
        }
        return ConditionResult::fail(&format!("Service '{}' is not stopped", service));
    }

    // Check enabled/disabled
    if state_lower == "enabled" {
        let output = Command::new("systemctl")
            .args(["is-enabled", service])
            .output();

        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "enabled" {
                return ConditionResult::ok(&format!("Service '{}' is enabled", service));
            }
        }
        return ConditionResult::fail(&format!("Service '{}' is not enabled", service));
    }

    if state_lower == "disabled" {
        let output = Command::new("systemctl")
            .args(["is-enabled", service])
            .output();

        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "disabled" {
                return ConditionResult::ok(&format!("Service '{}' is disabled", service));
            }
        }
        return ConditionResult::fail(&format!("Service '{}' is not disabled", service));
    }

    ConditionResult::fail(&format!("Unknown service state: {}", expected_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_substitute_vars() {
        let mut vars = HashMap::new();
        vars.insert("service".to_string(), "nginx".to_string());
        vars.insert("port".to_string(), "8080".to_string());

        assert_eq!(
            substitute_vars("systemctl status ${service}", &vars),
            "systemctl status nginx"
        );
        assert_eq!(
            substitute_vars("curl localhost:$port", &vars),
            "curl localhost:8080"
        );
    }

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

    #[test]
    fn test_file_exists_condition() {
        let result = eval_file_exists("/etc/hosts");
        assert!(result.success);

        let result2 = eval_file_exists("/nonexistent/path/12345");
        assert!(!result2.success);
    }
}
