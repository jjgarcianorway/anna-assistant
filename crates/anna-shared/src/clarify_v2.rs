//! Clarification engine v2 (v0.0.44).
//!
//! Clean request/response flow with verification integration.
//! Auto-select when only one option. Installed-only menus.

use crate::facts::{FactKey, FactSource, FactValue, FactsStore};
use crate::inventory::InventoryCache;
use crate::verify::{run_verification, VerificationStep, VerifyExpectation};
use serde::{Deserialize, Serialize};

/// Reserved numeric keys for escape options
pub const KEY_CANCEL: u8 = 0;
pub const KEY_OTHER: u8 = 9;

/// Clarification request (v0.0.44)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyRequest {
    /// Unique identifier for this request
    pub id: &'static str,
    /// The question to ask
    pub question: String,
    /// Available options (only installed tools)
    pub options: Vec<ClarifyOption>,
    /// Allow cancel option
    pub allow_cancel: bool,
    /// Reason for asking
    pub reason: Option<String>,
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
        Self { key, label: label.into(), value: value.into(), verify: None }
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
        Self { selected: Some(key), free_text: None, cancelled: false }
    }

    pub fn other(text: impl Into<String>) -> Self {
        Self { selected: None, free_text: Some(text.into()), cancelled: false }
    }

    pub fn cancel() -> Self {
        Self { selected: None, free_text: None, cancelled: true }
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
            if num == KEY_OTHER {
                return Self::other("");
            }
            if prompt.options.iter().any(|o| o.key == num) {
                return Self::selected(num);
            }
        }

        // Treat as free text (other)
        Self::other(trimmed)
    }
}

impl ClarifyRequest {
    /// Create new request
    pub fn new(id: &'static str, question: impl Into<String>) -> Self {
        Self {
            id,
            question: question.into(),
            options: Vec::new(),
            allow_cancel: true,
            reason: None,
        }
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
        lines.push(format!("  [{}] Something else (type it)", KEY_OTHER));

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

/// Result of processing a clarification (v0.0.44)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ClarifyResult {
    /// User selected a verified option
    Verified { value: String, fact_key: Option<String> },
    /// Auto-selected (only one option)
    AutoSelected { value: String },
    /// User provided other input (needs verification)
    NeedsVerification { value: String },
    /// Verification failed (offer alternatives)
    VerificationFailed { value: String, error: String, alternatives: Vec<String> },
    /// User cancelled
    Cancelled,
}

impl ClarifyResult {
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Verified { value, .. } |
            Self::AutoSelected { value } |
            Self::NeedsVerification { value } => Some(value),
            _ => None,
        }
    }

    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. } | Self::AutoSelected { .. })
    }
}

/// Process clarification response with verification
pub fn process_response(
    request: &ClarifyRequest,
    response: &ClarifyResponse,
    cache: &InventoryCache,
) -> ClarifyResult {
    if response.cancelled {
        return ClarifyResult::Cancelled;
    }

    // Handle free text (other)
    if let Some(text) = &response.free_text {
        if text.is_empty() {
            return ClarifyResult::Cancelled;
        }

        // Verify the free text input
        let step = VerificationStep::editor_installed(text);
        let result = run_verification(&step);

        if result.passed {
            return ClarifyResult::Verified {
                value: text.clone(),
                fact_key: Some("preferred_editor".to_string()),
            };
        } else {
            let alts = find_installed_alternatives(text, cache);
            return ClarifyResult::VerificationFailed {
                value: text.clone(),
                error: result.error.unwrap_or_else(|| "not installed".to_string()),
                alternatives: alts,
            };
        }
    }

    // Handle numeric selection
    if let Some(key) = response.selected {
        if let Some(opt) = request.get_option(key) {
            if let Some(verify_exp) = &opt.verify {
                let step = VerificationStep::new(
                    format!("verify_{}", opt.value),
                    format!("Verify {}", opt.label),
                    verify_exp.clone(),
                );
                let result = run_verification(&step);

                if result.passed {
                    return ClarifyResult::Verified {
                        value: opt.value.clone(),
                        fact_key: Some("preferred_editor".to_string()),
                    };
                } else {
                    let alts = find_installed_alternatives(&opt.value, cache);
                    return ClarifyResult::VerificationFailed {
                        value: opt.value.clone(),
                        error: result.error.unwrap_or_else(|| "failed".to_string()),
                        alternatives: alts,
                    };
                }
            } else {
                return ClarifyResult::Verified {
                    value: opt.value.clone(),
                    fact_key: None,
                };
            }
        }
    }

    ClarifyResult::Cancelled
}

/// Find installed alternatives for a tool
pub fn find_installed_alternatives(tool: &str, cache: &InventoryCache) -> Vec<String> {
    let alt_map: &[(&str, &[&str])] = &[
        ("vim", &["nvim", "vi", "nano", "micro"]),
        ("nvim", &["vim", "vi", "nano", "micro"]),
        ("emacs", &["vim", "nano", "code", "nvim"]),
        ("code", &["vim", "nano", "nvim", "emacs"]),
        ("nano", &["vim", "micro", "vi", "nvim"]),
        ("vi", &["vim", "nano", "nvim", "micro"]),
        ("micro", &["nano", "vim", "nvim", "vi"]),
    ];

    let mut alts = Vec::new();
    for (t, alternatives) in alt_map {
        if *t == tool {
            for alt in *alternatives {
                if cache.is_installed(alt).unwrap_or(false) {
                    alts.push(alt.to_string());
                }
            }
            break;
        }
    }
    alts
}

/// Generate installed-only editor request (v0.45.x: shows only installed editors)
pub fn editor_request(cache: &InventoryCache) -> ClarifyRequest {
    let editors = [
        ("vim", "Vim"), ("nvim", "Neovim"), ("nano", "Nano"),
        ("emacs", "Emacs"), ("code", "VS Code"), ("micro", "Micro"), ("vi", "Vi"),
    ];

    let mut opts = Vec::new();
    let mut key: u8 = 1;

    for (cmd, label) in &editors {
        if cache.is_installed(cmd).unwrap_or(false) && key < KEY_OTHER {
            // Use friendly label for display, command for value
            opts.push(ClarifyOption::new(key, *label, *cmd)
                .with_verify(VerifyExpectation::CommandExists { name: cmd.to_string() }));
            key += 1;
        }
    }

    ClarifyRequest::new("editor_select", "Which editor do you prefer?")
        .with_options(opts)
        .with_reason("Options shown are installed on your system")
}

/// Check if clarification can be skipped
/// - Skip if fact is fresh and verified
/// - Skip if only one option (auto-select)
pub fn should_skip(
    fact_key: &FactKey,
    facts: &FactsStore,
    cache: &InventoryCache,
) -> Option<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    match fact_key {
        FactKey::PreferredEditor => {
            // Check if we have a verified editor fact
            if let Some(fact) = facts.get_fresh(&FactKey::PreferredEditor, now) {
                // Verify it's still installed
                if cache.is_installed(&fact.value).unwrap_or(false) {
                    return Some(fact.value.clone());
                }
            }

            // Check if only one editor installed (auto-select)
            let installed = cache.installed_editors();
            if installed.len() == 1 {
                return Some(installed[0].to_string());
            }
        }
        _ => {}
    }

    None
}

/// Store verified fact from clarification
pub fn store_fact(
    fact_key: FactKey,
    value: &str,
    facts: &mut FactsStore,
    transcript_id: &str,
) {
    facts.upsert_verified(
        fact_key,
        FactValue::String(value.to_string()),
        FactSource::UserConfirmed { transcript_id: transcript_id.to_string() },
        90, // User-confirmed confidence
    );
}

/// Invalidate fact when tool is uninstalled
pub fn invalidate_on_uninstall(
    tool: &str,
    facts: &mut FactsStore,
    cache: &InventoryCache,
) -> bool {
    // Check if tool still exists
    if cache.is_installed(tool).unwrap_or(false) {
        return false; // Still installed
    }

    // Mark related facts as stale
    if let Some(value) = facts.get_verified(&FactKey::PreferredEditor) {
        if value == tool {
            facts.invalidate(&FactKey::PreferredEditor);
            return true;
        }
    }

    false
}

/// Track verification failures for a fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyFailureTracker {
    /// Map of fact key string to failure count
    failures: std::collections::HashMap<String, u8>,
}

impl VerifyFailureTracker {
    pub fn new() -> Self { Self::default() }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarify_request_format() {
        let req = ClarifyRequest::new("test", "Which editor?")
            .add_option(ClarifyOption::tool(1, "vim"))
            .add_option(ClarifyOption::tool(2, "nano"));

        let menu = req.format_menu();
        assert!(menu.contains("[1] vim"));
        assert!(menu.contains("[2] nano"));
        assert!(menu.contains("[0] Cancel"));
        assert!(menu.contains("[9] Something else"));
    }

    #[test]
    fn test_parse_numeric_response() {
        let req = ClarifyRequest::new("test", "Which?")
            .add_option(ClarifyOption::tool(1, "vim"));

        let resp = ClarifyResponse::parse("1", &req);
        assert_eq!(resp.selected, Some(1));
        assert!(!resp.cancelled);
    }

    #[test]
    fn test_parse_cancel() {
        let req = ClarifyRequest::new("test", "Which?");

        let resp = ClarifyResponse::parse("0", &req);
        assert!(resp.cancelled);

        let resp = ClarifyResponse::parse("cancel", &req);
        assert!(resp.cancelled);
    }

    #[test]
    fn test_parse_free_text() {
        let req = ClarifyRequest::new("test", "Which?");

        let resp = ClarifyResponse::parse("emacs", &req);
        assert_eq!(resp.free_text, Some("emacs".to_string()));
    }

    #[test]
    fn test_single_option() {
        let req = ClarifyRequest::new("test", "Which?")
            .add_option(ClarifyOption::tool(1, "vim"));

        assert!(req.is_single_option());
        assert_eq!(req.single_option_value(), Some("vim"));
    }

    #[test]
    fn test_verify_failure_tracker() {
        let mut tracker = VerifyFailureTracker::new();

        assert!(!tracker.should_reclarify("editor"));

        tracker.record_failure("editor");
        assert!(!tracker.should_reclarify("editor"));

        tracker.record_failure("editor");
        assert!(tracker.should_reclarify("editor"));

        tracker.clear("editor");
        assert!(!tracker.should_reclarify("editor"));
    }
}
