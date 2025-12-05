//! Verification steps for pre-action and post-action checks (v0.0.44).
//!
//! Ensures Anna verifies tool existence before action, and confirms
//! outcomes after applying changes.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// What we expect to verify
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VerifyExpectation {
    /// Command exists and is executable (command -v succeeds)
    CommandExists { name: String },
    /// Command exits with expected code
    ExitCode { command: String, expected: i32 },
    /// File exists at path
    FileExists { path: String },
    /// File contains specific line/pattern
    FileContainsLine { path: String, pattern: String },
    /// Package is installed (pacman -Q or similar)
    PackageInstalled { package: String },
    /// Systemd service is in expected state
    ServiceState { service: String, expected: ServiceExpectedState },
    /// Output contains pattern
    OutputContains { command: String, pattern: String },
}

/// Expected service states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceExpectedState {
    Active,
    Inactive,
    Enabled,
    Disabled,
}

impl std::fmt::Display for ServiceExpectedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Enabled => write!(f, "enabled"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// A verification step with description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// Unique identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// What we're verifying
    pub expectation: VerifyExpectation,
    /// Is this step mandatory (failure blocks action)?
    pub mandatory: bool,
}

impl VerificationStep {
    /// Create a new verification step
    pub fn new(id: impl Into<String>, desc: impl Into<String>, exp: VerifyExpectation) -> Self {
        Self {
            id: id.into(),
            description: desc.into(),
            expectation: exp,
            mandatory: true,
        }
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.mandatory = false;
        self
    }

    /// Verify editor is installed
    pub fn editor_installed(editor: &str) -> Self {
        Self::new(
            format!("verify_{}_installed", editor),
            format!("Verify {} is installed", editor),
            VerifyExpectation::CommandExists { name: editor.to_string() },
        )
    }

    /// Verify file contains line (for post-change verification)
    pub fn file_has_line(path: &str, line: &str) -> Self {
        Self::new(
            "verify_config_line",
            format!("Verify config contains: {}", line),
            VerifyExpectation::FileContainsLine {
                path: path.to_string(),
                pattern: line.to_string(),
            },
        )
    }

    /// Verify service state
    pub fn service_is(service: &str, state: ServiceExpectedState) -> Self {
        Self::new(
            format!("verify_{}_{}", service, state),
            format!("Verify {} is {}", service, state),
            VerifyExpectation::ServiceState {
                service: service.to_string(),
                expected: state,
            },
        )
    }
}

/// Result of running a verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    /// The step that was verified
    pub step_id: String,
    /// Whether verification passed
    pub passed: bool,
    /// Actual output/state observed
    pub actual: String,
    /// Error message if failed
    pub error: Option<String>,
}

impl VerifyResult {
    pub fn pass(step_id: &str, actual: impl Into<String>) -> Self {
        Self { step_id: step_id.to_string(), passed: true, actual: actual.into(), error: None }
    }

    pub fn fail(step_id: &str, actual: impl Into<String>, err: impl Into<String>) -> Self {
        Self {
            step_id: step_id.to_string(),
            passed: false,
            actual: actual.into(),
            error: Some(err.into()),
        }
    }
}

/// Run a verification step
pub fn run_verification(step: &VerificationStep) -> VerifyResult {
    match &step.expectation {
        VerifyExpectation::CommandExists { name } => verify_command_exists(&step.id, name),
        VerifyExpectation::ExitCode { command, expected } => {
            verify_exit_code(&step.id, command, *expected)
        }
        VerifyExpectation::FileExists { path } => verify_file_exists(&step.id, path),
        VerifyExpectation::FileContainsLine { path, pattern } => {
            verify_file_contains(&step.id, path, pattern)
        }
        VerifyExpectation::PackageInstalled { package } => {
            verify_package_installed(&step.id, package)
        }
        VerifyExpectation::ServiceState { service, expected } => {
            verify_service_state(&step.id, service, *expected)
        }
        VerifyExpectation::OutputContains { command, pattern } => {
            verify_output_contains(&step.id, command, pattern)
        }
    }
}

fn verify_command_exists(step_id: &str, name: &str) -> VerifyResult {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            VerifyResult::pass(step_id, path)
        }
        Ok(_) => VerifyResult::fail(step_id, "not found", format!("{} is not installed", name)),
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to check: {}", e)),
    }
}

fn verify_exit_code(step_id: &str, command: &str, expected: i32) -> VerifyResult {
    let output = Command::new("sh").arg("-c").arg(command).output();

    match output {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            if code == expected {
                VerifyResult::pass(step_id, format!("exit code {}", code))
            } else {
                VerifyResult::fail(step_id, format!("exit code {}", code),
                    format!("Expected exit code {}, got {}", expected, code))
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to run: {}", e)),
    }
}

fn verify_file_exists(step_id: &str, path: &str) -> VerifyResult {
    // Expand ~ to home directory
    let expanded = expand_path(path);
    if std::path::Path::new(&expanded).exists() {
        VerifyResult::pass(step_id, expanded)
    } else {
        VerifyResult::fail(step_id, "not found", format!("File {} does not exist", path))
    }
}

fn verify_file_contains(step_id: &str, path: &str, pattern: &str) -> VerifyResult {
    let expanded = expand_path(path);
    match std::fs::read_to_string(&expanded) {
        Ok(content) => {
            if content.contains(pattern) {
                VerifyResult::pass(step_id, "pattern found")
            } else {
                VerifyResult::fail(step_id, "pattern not found",
                    format!("File does not contain: {}", pattern))
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Cannot read file: {}", e)),
    }
}

fn verify_package_installed(step_id: &str, package: &str) -> VerifyResult {
    // Try pacman first (Arch)
    let output = Command::new("pacman").arg("-Q").arg(package).output();

    match output {
        Ok(out) if out.status.success() => {
            let info = String::from_utf8_lossy(&out.stdout).trim().to_string();
            VerifyResult::pass(step_id, info)
        }
        Ok(_) => VerifyResult::fail(step_id, "not installed",
            format!("Package {} is not installed", package)),
        Err(_) => {
            // Try dpkg (Debian/Ubuntu)
            let dpkg = Command::new("dpkg").arg("-s").arg(package).output();
            match dpkg {
                Ok(out) if out.status.success() => VerifyResult::pass(step_id, "installed (dpkg)"),
                _ => VerifyResult::fail(step_id, "unknown", "Cannot determine package status"),
            }
        }
    }
}

fn verify_service_state(step_id: &str, service: &str, expected: ServiceExpectedState) -> VerifyResult {
    let check_cmd = match expected {
        ServiceExpectedState::Active | ServiceExpectedState::Inactive => "is-active",
        ServiceExpectedState::Enabled | ServiceExpectedState::Disabled => "is-enabled",
    };

    let output = Command::new("systemctl").arg(check_cmd).arg(service).output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let matches = match expected {
                ServiceExpectedState::Active => state == "active",
                ServiceExpectedState::Inactive => state == "inactive",
                ServiceExpectedState::Enabled => state == "enabled",
                ServiceExpectedState::Disabled => state == "disabled",
            };
            if matches {
                VerifyResult::pass(step_id, state)
            } else {
                VerifyResult::fail(step_id, &state,
                    format!("Expected {}, got {}", expected, state))
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to check: {}", e)),
    }
}

fn verify_output_contains(step_id: &str, command: &str, pattern: &str) -> VerifyResult {
    let output = Command::new("sh").arg("-c").arg(command).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains(pattern) {
                VerifyResult::pass(step_id, "pattern found in output")
            } else {
                VerifyResult::fail(step_id, "pattern not found",
                    format!("Output does not contain: {}", pattern))
            }
        }
        Err(e) => VerifyResult::fail(step_id, "", format!("Failed to run: {}", e)),
    }
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var("HOME").ok() {
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

/// Pre-action verification batch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreActionVerify {
    pub steps: Vec<VerificationStep>,
    pub results: Vec<VerifyResult>,
    pub all_passed: bool,
}

impl PreActionVerify {
    pub fn new() -> Self { Self::default() }

    pub fn add(mut self, step: VerificationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Run all verification steps
    pub fn run(mut self) -> Self {
        self.results = self.steps.iter().map(run_verification).collect();
        self.all_passed = self.results.iter()
            .zip(&self.steps)
            .all(|(r, s)| r.passed || !s.mandatory);
        self
    }

    /// Get failed mandatory steps
    pub fn failed_mandatory(&self) -> Vec<(&VerificationStep, &VerifyResult)> {
        self.steps.iter().zip(&self.results)
            .filter(|(s, r)| s.mandatory && !r.passed)
            .collect()
    }

    /// Summary for transcript
    pub fn summary(&self) -> String {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        if self.all_passed {
            format!("Verified {}/{} checks passed", passed, total)
        } else {
            let failed: Vec<_> = self.failed_mandatory()
                .iter()
                .map(|(s, r)| format!("{}: {}", s.description, r.error.as_deref().unwrap_or("failed")))
                .collect();
            format!("Verification failed: {}", failed.join("; "))
        }
    }
}

/// Post-action verification batch
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostActionVerify {
    pub steps: Vec<VerificationStep>,
    pub results: Vec<VerifyResult>,
    pub success: bool,
}

impl PostActionVerify {
    pub fn new() -> Self { Self::default() }

    pub fn add(mut self, step: VerificationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Run all verification steps
    pub fn run(mut self) -> Self {
        self.results = self.steps.iter().map(run_verification).collect();
        self.success = self.results.iter().all(|r| r.passed);
        self
    }

    /// Get confirmation message for transcript
    pub fn confirmation(&self) -> String {
        if self.success {
            "Change verified successfully".to_string()
        } else {
            let failed: Vec<_> = self.results.iter()
                .filter(|r| !r.passed)
                .map(|r| r.error.as_deref().unwrap_or("unknown"))
                .collect();
            format!("Change may not have applied: {}", failed.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_command_exists_sh() {
        let step = VerificationStep::editor_installed("sh");
        let result = run_verification(&step);
        assert!(result.passed, "sh should exist on Unix systems");
    }

    #[test]
    fn test_verify_command_not_exists() {
        let step = VerificationStep::editor_installed("definitely_not_a_real_command_xyz");
        let result = run_verification(&step);
        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_pre_action_verify_batch() {
        let verify = PreActionVerify::new()
            .add(VerificationStep::editor_installed("sh"))
            .add(VerificationStep::editor_installed("nonexistent_xyz").optional())
            .run();

        assert!(verify.all_passed); // sh passes, nonexistent is optional
    }

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("~/.vimrc");
        assert!(!expanded.starts_with("~"));
    }

    #[test]
    fn test_verification_step_constructors() {
        let step = VerificationStep::editor_installed("vim");
        assert!(step.id.contains("vim"));
        assert!(step.mandatory);

        let step = VerificationStep::file_has_line("/etc/hosts", "localhost");
        assert!(step.description.contains("localhost"));

        let step = VerificationStep::service_is("sshd", ServiceExpectedState::Active);
        assert!(step.description.contains("sshd"));
    }
}
