//! Clarification request types.

use serde::{Deserialize, Serialize};

use super::clarification_types::{ClarificationOption, ClarificationType};

/// A clarification request from a specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequest {
    /// Type of clarification needed.
    pub clarification_type: ClarificationType,
    /// The question to ask the user.
    pub question: String,
    /// Why this clarification is needed.
    pub reason: String,
    /// Available options (for Choice type).
    pub options: Vec<ClarificationOption>,
    /// Default option if user skips.
    pub default: Option<String>,
    /// Whether this is blocking (must answer) or optional.
    pub required: bool,
    /// Context about what the specialist is trying to do.
    pub context: Option<String>,
}

impl ClarificationRequest {
    /// Create a new choice clarification.
    pub fn choice(question: &str, options: Vec<&str>) -> Self {
        Self {
            clarification_type: ClarificationType::Choice,
            question: question.to_string(),
            reason: String::new(),
            options: options
                .into_iter()
                .map(|o| ClarificationOption::simple(o))
                .collect(),
            default: None,
            required: true,
            context: None,
        }
    }

    /// Create a value request.
    pub fn value(question: &str) -> Self {
        Self {
            clarification_type: ClarificationType::Value,
            question: question.to_string(),
            reason: String::new(),
            options: Vec::new(),
            default: None,
            required: true,
            context: None,
        }
    }

    /// Create a confirmation request.
    pub fn confirmation(question: &str) -> Self {
        Self {
            clarification_type: ClarificationType::Confirmation,
            question: question.to_string(),
            reason: String::new(),
            options: vec![
                ClarificationOption::new("yes", "Proceed"),
                ClarificationOption::new("no", "Cancel"),
            ],
            default: Some("no".to_string()),
            required: true,
            context: None,
        }
    }

    /// Create a context request.
    pub fn context(question: &str) -> Self {
        Self {
            clarification_type: ClarificationType::Context,
            question: question.to_string(),
            reason: String::new(),
            options: Vec::new(),
            default: None,
            required: false,
            context: None,
        }
    }

    /// Create a scope request.
    pub fn scope(question: &str, scopes: Vec<&str>) -> Self {
        Self {
            clarification_type: ClarificationType::Scope,
            question: question.to_string(),
            reason: String::new(),
            options: scopes
                .into_iter()
                .map(|s| ClarificationOption::simple(s))
                .collect(),
            default: None,
            required: true,
            context: None,
        }
    }

    /// Set the reason for clarification.
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = reason.to_string();
        self
    }

    /// Set a default value.
    pub fn with_default(mut self, default: &str) -> Self {
        self.default = Some(default.to_string());
        self
    }

    /// Set context.
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    /// Make optional.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Format as a user-facing question.
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Context if provided
        if let Some(ctx) = &self.context {
            output.push_str(ctx);
            output.push_str("\n\n");
        }

        // Question
        output.push_str(&self.question);

        // Reason if provided
        if !self.reason.is_empty() {
            output.push_str(&format!("\n({})", self.reason));
        }

        // Options
        if !self.options.is_empty() {
            output.push('\n');
            for (i, opt) in self.options.iter().enumerate() {
                let marker = if Some(opt.value.clone()) == self.default {
                    "*"
                } else {
                    " "
                };
                output.push_str(&format!("{} {}. {}", marker, i + 1, opt.format()));
            }
        }

        // Default hint
        if let Some(def) = &self.default {
            output.push_str(&format!("\n[default: {}]", def));
        }

        output
    }
}
