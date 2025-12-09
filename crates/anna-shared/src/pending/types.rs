//! Pending clarification types (v0.0.227).

use crate::clarify::{ClarifyKind, ClarifyOption};
use crate::facts::FactKey;
use serde::{Deserialize, Serialize};

/// A pending clarification awaiting user response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingClarification {
    /// Unique request ID this clarification belongs to
    pub request_id: String,
    /// The question being asked
    pub question: String,
    /// Available options (numbered for easy selection)
    pub options: Vec<ClarifyOption>,
    /// What kind of clarification this is
    pub kind: ClarifyKind,
    /// What fact key to set if clarification succeeds (optional)
    pub fact_key: Option<FactKey>,
    /// Verification command template (e.g., "which {}")
    pub verify_command: Option<String>,
    /// Original query that triggered this clarification
    pub original_query: String,
    /// Timestamp when clarification was created
    pub created_at: u64,
}

impl PendingClarification {
    /// Create new pending clarification
    pub fn new(
        request_id: &str,
        question: &str,
        options: Vec<ClarifyOption>,
        kind: ClarifyKind,
        original_query: &str,
    ) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            request_id: request_id.to_string(),
            question: question.to_string(),
            options,
            kind,
            fact_key: None,
            verify_command: None,
            original_query: original_query.to_string(),
            created_at,
        }
    }

    /// Set fact key to be populated on resolution
    pub fn with_fact_key(mut self, key: FactKey) -> Self {
        self.fact_key = Some(key);
        self
    }

    /// Set verification command template
    pub fn with_verify(mut self, cmd: &str) -> Self {
        self.verify_command = Some(cmd.to_string());
        self
    }

    /// Format as display text for user
    pub fn format_prompt(&self) -> String {
        let mut lines = vec![self.question.clone()];

        for (i, opt) in self.options.iter().enumerate() {
            let evidence = if opt.evidence.is_empty() {
                String::new()
            } else {
                format!(" ({})", opt.evidence.join(", "))
            };
            lines.push(format!("  {}) {}{}", i + 1, opt.label, evidence));
        }

        lines.push(String::new());
        lines.push("Enter number, name, or 'cancel':".to_string());

        lines.join("\n")
    }

    /// Parse user input and return selected option key
    pub fn parse_input(&self, input: &str) -> ParseResult {
        let input = input.trim().to_lowercase();

        // Check for cancel
        if input == "cancel" || input == "c" || input == "0" {
            return ParseResult::Cancelled;
        }

        // Check for number selection
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= self.options.len() {
                return ParseResult::Selected(self.options[num - 1].key.clone());
            }
            return ParseResult::Invalid("Invalid option number".to_string());
        }

        // Check for direct key/label match
        for opt in &self.options {
            if opt.key.to_lowercase() == input || opt.label.to_lowercase() == input {
                return ParseResult::Selected(opt.key.clone());
            }
        }

        // Treat as custom "other" input
        ParseResult::Custom(input)
    }

    /// Check if pending clarification is stale (>1 hour old)
    pub fn is_stale(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.created_at) > 3600 // 1 hour
    }
}

/// Result of parsing user input for clarification
#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult {
    /// User selected a specific option
    Selected(String),
    /// User provided custom input
    Custom(String),
    /// User cancelled
    Cancelled,
    /// Invalid input
    Invalid(String),
}

/// Verification result for clarification answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifyResult {
    /// Answer verified successfully
    Verified { value: String },
    /// Answer not verified, but close alternative exists
    AlternativeFound {
        requested: String,
        available: String,
    },
    /// Answer could not be verified
    NotVerified { value: String, reason: String },
}
