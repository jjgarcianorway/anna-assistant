//! Pending clarification persistence for REPL continuity (v0.0.36).
//!
//! When Anna needs clarification, the pending question is saved so the user
//! can resume in a new REPL session without losing context.

use crate::clarify::{ClarifyKind, ClarifyOption};
use crate::facts::FactKey;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    AlternativeFound { requested: String, available: String },
    /// Answer could not be verified
    NotVerified { value: String, reason: String },
}

/// Verify a clarification answer (e.g., check if editor is installed)
pub fn verify_answer(answer: &str, verify_cmd: Option<&str>) -> VerifyResult {
    let Some(cmd_template) = verify_cmd else {
        // No verification needed
        return VerifyResult::Verified {
            value: answer.to_string(),
        };
    };

    let cmd = cmd_template.replace("{}", answer);
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.is_empty() {
        return VerifyResult::Verified {
            value: answer.to_string(),
        };
    }

    // Run verification command
    let output = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .output();

    match output {
        Ok(out) if out.status.success() => VerifyResult::Verified {
            value: answer.to_string(),
        },
        _ => {
            // Check for common alternatives
            if answer == "vim" {
                // Check if "vi" exists instead
                if let Ok(out) = std::process::Command::new("which").arg("vi").output() {
                    if out.status.success() {
                        return VerifyResult::AlternativeFound {
                            requested: "vim".to_string(),
                            available: "vi".to_string(),
                        };
                    }
                }
            }
            VerifyResult::NotVerified {
                value: answer.to_string(),
                reason: format!("{} not found", answer),
            }
        }
    }
}

// === Persistence ===

/// Get pending clarification file path
pub fn pending_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("pending.json")
}

/// Load pending clarification (if any)
pub fn load_pending() -> Option<PendingClarification> {
    let path = pending_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    let pending: PendingClarification = serde_json::from_str(&data).ok()?;

    // Check if stale
    if pending.is_stale() {
        let _ = clear_pending();
        return None;
    }

    Some(pending)
}

/// Save pending clarification
pub fn save_pending(pending: &PendingClarification) -> std::io::Result<()> {
    let path = pending_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(pending)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear pending clarification
pub fn clear_pending() -> std::io::Result<()> {
    let path = pending_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Check if there's a pending clarification
pub fn has_pending() -> bool {
    pending_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_options() -> Vec<ClarifyOption> {
        vec![
            ClarifyOption::new("vim", "Vim"),
            ClarifyOption::new("nano", "Nano"),
            ClarifyOption::new("__other__", "Other"),
            ClarifyOption::new("__cancel__", "Cancel"),
        ]
    }

    #[test]
    fn test_pending_creation() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor do you prefer?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "edit the config",
        );

        assert_eq!(pending.request_id, "req-123");
        assert_eq!(pending.options.len(), 4);
    }

    #[test]
    fn test_parse_number_input() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(pending.parse_input("1"), ParseResult::Selected("vim".to_string()));
        assert_eq!(pending.parse_input("2"), ParseResult::Selected("nano".to_string()));
    }

    #[test]
    fn test_parse_name_input() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(pending.parse_input("vim"), ParseResult::Selected("vim".to_string()));
        assert_eq!(pending.parse_input("VIM"), ParseResult::Selected("vim".to_string()));
    }

    #[test]
    fn test_parse_cancel() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(pending.parse_input("cancel"), ParseResult::Cancelled);
        assert_eq!(pending.parse_input("c"), ParseResult::Cancelled);
        assert_eq!(pending.parse_input("0"), ParseResult::Cancelled);
    }

    #[test]
    fn test_parse_custom() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert_eq!(pending.parse_input("emacs"), ParseResult::Custom("emacs".to_string()));
    }

    #[test]
    fn test_format_prompt() {
        let pending = PendingClarification::new(
            "req-123",
            "Which editor do you prefer?",
            sample_options(),
            ClarifyKind::PreferredEditor,
            "test",
        );

        let prompt = pending.format_prompt();
        assert!(prompt.contains("Which editor"));
        assert!(prompt.contains("1) Vim"));
        assert!(prompt.contains("2) Nano"));
    }

    #[test]
    fn test_staleness() {
        let mut pending = PendingClarification::new(
            "req-123",
            "test",
            vec![],
            ClarifyKind::PreferredEditor,
            "test",
        );

        assert!(!pending.is_stale()); // Just created

        // Simulate old timestamp
        pending.created_at = 0;
        assert!(pending.is_stale());
    }
}
