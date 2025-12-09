//! Verification types (v0.0.198).

use serde::{Deserialize, Serialize};

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
    ServiceState {
        service: String,
        expected: ServiceExpectedState,
    },
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
            VerifyExpectation::CommandExists {
                name: editor.to_string(),
            },
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
        Self {
            step_id: step_id.to_string(),
            passed: true,
            actual: actual.into(),
            error: None,
        }
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
