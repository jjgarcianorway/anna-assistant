//! Progress types (v0.0.204).

use serde::{Deserialize, Serialize};

/// Maximum length for diagnostic text fields (error messages, details).
pub const MAX_DIAGNOSTIC_LENGTH: usize = 100;

/// Diagnostic text with enforced length cap.
/// Prevents accidental content leakage through progress events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiagnosticText(String);

impl DiagnosticText {
    /// Create diagnostic text, truncating if over limit.
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.len() > MAX_DIAGNOSTIC_LENGTH {
            Self(format!("{}...", &s[..MAX_DIAGNOSTIC_LENGTH - 3]))
        } else {
            Self(s)
        }
    }

    /// Get the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagnosticText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for DiagnosticText {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for DiagnosticText {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::ops::Deref for DiagnosticText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Stage of request processing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStage {
    Translator,
    Probes,
    Specialist,
    Supervisor,
}

impl std::fmt::Display for RequestStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Translator => write!(f, "translator"),
            Self::Probes => write!(f, "probes"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
        }
    }
}

/// Timeout configuration for each stage
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    pub translator_secs: u64,
    pub probe_each_secs: u64,
    pub probes_total_secs: u64,
    pub specialist_secs: u64,
    pub supervisor_secs: u64,
    pub heartbeat_interval_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            translator_secs: 8,
            probe_each_secs: 4,
            probes_total_secs: 10,
            specialist_secs: 12,
            supervisor_secs: 8,
            heartbeat_interval_secs: 3,
        }
    }
}
