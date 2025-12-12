//! Clarification protocol for specialists (v0.0.432).
//!
//! When a specialist needs more information to answer correctly,
//! they can request clarification through this protocol.

use serde::{Deserialize, Serialize};

/// Types of clarification requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarificationType {
    /// Need to know which option the user prefers.
    Choice,
    /// Need a specific value or parameter.
    Value,
    /// Need confirmation before proceeding.
    Confirmation,
    /// Need more context about the situation.
    Context,
    /// Need to know the scope/target.
    Scope,
}

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

/// An option in a clarification request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    /// The value to use if selected.
    pub value: String,
    /// Human-readable label.
    pub label: String,
    /// Optional description.
    pub description: Option<String>,
}

impl ClarificationOption {
    /// Create a new option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            description: None,
        }
    }

    /// Create a simple option (value = label).
    pub fn simple(value: &str) -> Self {
        Self {
            value: value.to_string(),
            label: value.to_string(),
            description: None,
        }
    }

    /// Create an option with description.
    pub fn with_desc(value: &str, label: &str, description: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            description: Some(description.to_string()),
        }
    }

    /// Format for display.
    pub fn format(&self) -> String {
        match &self.description {
            Some(desc) => format!("{} - {}", self.label, desc),
            None => self.label.clone(),
        }
    }
}

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

/// Clarification protocol manager.
pub struct ClarificationProtocol {
    /// Pending clarifications.
    pending: Vec<ClarificationRequest>,
    /// History of clarifications.
    history: Vec<(ClarificationRequest, ClarificationResponse)>,
}

impl ClarificationProtocol {
    /// Create a new protocol instance.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Request clarification.
    pub fn request(&mut self, req: ClarificationRequest) {
        self.pending.push(req);
    }

    /// Check if there are pending clarifications.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Get next pending clarification.
    pub fn next_pending(&self) -> Option<&ClarificationRequest> {
        self.pending.first()
    }

    /// Resolve the current pending clarification.
    pub fn resolve(&mut self, response: ClarificationResponse) -> Option<ClarificationRequest> {
        if let Some(req) = self.pending.first().cloned() {
            self.pending.remove(0);
            self.history.push((req.clone(), response));
            Some(req)
        } else {
            None
        }
    }

    /// Skip the current pending clarification (use default if available).
    pub fn skip(&mut self) -> Option<ClarificationResponse> {
        if let Some(req) = self.pending.first() {
            if let Some(default) = &req.default {
                let response = ClarificationResponse::default_value(default);
                self.resolve(response.clone());
                return Some(response);
            } else if !req.required {
                self.pending.remove(0);
                return Some(ClarificationResponse::default_value(""));
            }
        }
        None
    }

    /// Get clarification history.
    pub fn history(&self) -> &[(ClarificationRequest, ClarificationResponse)] {
        &self.history
    }

    /// Clear all pending clarifications.
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    /// Get count of pending clarifications.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for ClarificationProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_clarification() {
        let req =
            ClarificationRequest::choice("Which package manager?", vec!["pacman", "yay", "paru"])
                .with_default("pacman");

        assert_eq!(req.options.len(), 3);
        assert_eq!(req.default, Some("pacman".to_string()));

        let formatted = req.format();
        assert!(formatted.contains("Which package manager?"));
        assert!(formatted.contains("pacman"));
    }

    #[test]
    fn test_confirmation() {
        let req = ClarificationRequest::confirmation("Proceed with installation?")
            .with_reason("This will modify system packages");

        assert!(req.question.contains("Proceed"));
        assert_eq!(req.options.len(), 2);
    }

    #[test]
    fn test_protocol_flow() {
        let mut protocol = ClarificationProtocol::new();

        assert!(!protocol.has_pending());

        protocol.request(ClarificationRequest::value("Enter package name"));
        assert!(protocol.has_pending());
        assert_eq!(protocol.pending_count(), 1);

        let response = ClarificationResponse::with_value("firefox");
        protocol.resolve(response);

        assert!(!protocol.has_pending());
        assert_eq!(protocol.history().len(), 1);
    }

    #[test]
    fn test_skip_with_default() {
        let mut protocol = ClarificationProtocol::new();

        protocol
            .request(ClarificationRequest::choice("Pick one", vec!["a", "b"]).with_default("a"));

        let skipped = protocol.skip();
        assert!(skipped.is_some());
        assert_eq!(skipped.unwrap().value, "a");
        assert!(!protocol.has_pending());
    }
}
