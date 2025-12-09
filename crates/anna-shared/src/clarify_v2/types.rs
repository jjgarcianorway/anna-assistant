//! Clarification types (v0.0.197).

use crate::verify::VerifyExpectation;
use serde::{Deserialize, Serialize};

/// Reserved numeric keys for escape options
pub const KEY_CANCEL: u8 = 0;
pub const KEY_OTHER: u8 = 9;

fn default_ttl() -> u32 {
    300 // 5 minutes
}

/// Clarification request (v0.0.44, updated v0.0.47)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyRequest {
    /// Unique identifier for this request
    pub id: String,
    /// The question to ask
    pub question: String,
    /// Available options (only installed tools)
    pub options: Vec<ClarifyOption>,
    /// Allow custom input (not just menu selection)
    pub allow_custom: bool,
    /// Allow cancel option
    pub allow_cancel: bool,
    /// Reason for asking
    pub reason: Option<String>,
    /// Time-to-live in seconds (0 = no expiry, default 300 = 5 min)
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u32,
}

impl ClarifyRequest {
    /// Create new request
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            question: question.into(),
            options: Vec::new(),
            allow_custom: true,
            allow_cancel: true,
            reason: None,
            ttl_seconds: default_ttl(),
        }
    }

    pub fn with_ttl(mut self, seconds: u32) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    pub fn no_custom(mut self) -> Self {
        self.allow_custom = false;
        self
    }

    pub fn add_option(mut self, opt: ClarifyOption) -> Self {
        self.options.push(opt);
        self
    }

    pub fn with_options(mut self, opts: Vec<ClarifyOption>) -> Self {
        self.options = opts;
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Format as menu for display
    pub fn format_menu(&self) -> String {
        let mut lines = vec![self.question.clone(), String::new()];

        for opt in &self.options {
            lines.push(format!("  [{}] {}", opt.key, opt.label));
        }

        lines.push(String::new());
        if self.allow_cancel {
            lines.push(format!("  [{}] Cancel", KEY_CANCEL));
        }
        if self.allow_custom {
            lines.push(format!("  [{}] Something else (type it)", KEY_OTHER));
        }

        if let Some(reason) = &self.reason {
            lines.push(String::new());
            lines.push(format!("  ({})", reason));
        }

        lines.join("\n")
    }

    pub fn get_option(&self, key: u8) -> Option<&ClarifyOption> {
        self.options.iter().find(|o| o.key == key)
    }

    /// Check if only one option (auto-select candidate)
    pub fn is_single_option(&self) -> bool {
        self.options.len() == 1
    }

    /// Get the single option value (for auto-select)
    pub fn single_option_value(&self) -> Option<&str> {
        if self.is_single_option() {
            Some(&self.options[0].value)
        } else {
            None
        }
    }
}

/// A clarification option with verification (v0.0.44)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    /// Numeric key (1-8)
    pub key: u8,
    /// Display label
    pub label: String,
    /// Value to store
    pub value: String,
    /// Verification step to run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyExpectation>,
}

impl ClarifyOption {
    pub fn new(key: u8, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            value: value.into(),
            verify: None,
        }
    }

    pub fn with_verify(mut self, verify: VerifyExpectation) -> Self {
        self.verify = Some(verify);
        self
    }

    /// Create option for an installed tool
    pub fn tool(key: u8, name: &str) -> Self {
        Self::new(key, name, name).with_verify(VerifyExpectation::CommandExists {
            name: name.to_string(),
        })
    }
}

/// Clarification response (v0.0.44)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyResponse {
    /// Selected option key (None if cancelled or free text)
    pub selected: Option<u8>,
    /// Free text if "other" selected
    pub free_text: Option<String>,
    /// Whether user cancelled
    pub cancelled: bool,
}

impl ClarifyResponse {
    pub fn selected(key: u8) -> Self {
        Self {
            selected: Some(key),
            free_text: None,
            cancelled: false,
        }
    }

    pub fn other(text: impl Into<String>) -> Self {
        Self {
            selected: None,
            free_text: Some(text.into()),
            cancelled: false,
        }
    }

    pub fn cancel() -> Self {
        Self {
            selected: None,
            free_text: None,
            cancelled: true,
        }
    }

    /// Parse user input into response
    pub fn parse(input: &str, prompt: &ClarifyRequest) -> Self {
        let trimmed = input.trim();

        // Check for cancel
        if trimmed == "0" || trimmed.eq_ignore_ascii_case("cancel") {
            return Self::cancel();
        }

        // Check for numeric selection
        if let Ok(num) = trimmed.parse::<u8>() {
            if num == KEY_CANCEL {
                return Self::cancel();
            }
            if num == KEY_OTHER && prompt.allow_custom {
                return Self::other("");
            }
            if prompt.options.iter().any(|o| o.key == num) {
                return Self::selected(num);
            }
        }

        // Treat as free text (other) if allowed
        if prompt.allow_custom {
            Self::other(trimmed)
        } else {
            // If custom not allowed, treat unknown input as cancel
            Self::cancel()
        }
    }
}

/// Result of processing a clarification (v0.0.44)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ClarifyResult {
    /// User selected a verified option
    Verified {
        value: String,
        fact_key: Option<String>,
    },
    /// Auto-selected (only one option)
    AutoSelected { value: String },
    /// User provided other input (needs verification)
    NeedsVerification { value: String },
    /// Verification failed (offer alternatives)
    VerificationFailed {
        value: String,
        error: String,
        alternatives: Vec<String>,
    },
    /// User cancelled
    Cancelled,
}

impl ClarifyResult {
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Verified { value, .. }
            | Self::AutoSelected { value }
            | Self::NeedsVerification { value } => Some(value),
            _ => None,
        }
    }

    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. } | Self::AutoSelected { .. })
    }
}

/// Track verification failures for a fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyFailureTracker {
    /// Map of fact key string to failure count
    failures: std::collections::HashMap<String, u8>,
}

impl VerifyFailureTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a verification failure
    pub fn record_failure(&mut self, key: &str) {
        let count = self.failures.entry(key.to_string()).or_insert(0);
        *count = count.saturating_add(1);
    }

    /// Get failure count
    pub fn failure_count(&self, key: &str) -> u8 {
        *self.failures.get(key).unwrap_or(&0)
    }

    /// Check if should re-clarify (2+ failures)
    pub fn should_reclarify(&self, key: &str) -> bool {
        self.failure_count(key) >= 2
    }

    /// Clear failures for key
    pub fn clear(&mut self, key: &str) {
        self.failures.remove(key);
    }
}
