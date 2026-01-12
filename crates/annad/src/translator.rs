//! Translator LLM - Lightweight model for fast input/output transformation.
//!
//! The Translator is the key component that converts:
//! - User natural language -> Actionable intents/commands
//! - Internal JSON/events -> Natural language dialogue for user
//!
//! Uses a very fast, lightweight model (llama3.2:1b or similar) to ensure
//! minimal latency while maintaining conversation quality.
//!
//! v0.1.0: Initial implementation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::{debug, info};

use crate::ollama;

/// Default translator model - very lightweight for speed
const DEFAULT_TRANSLATOR_MODEL: &str = "llama3.2:1b";

/// Cached translator model name
static TRANSLATOR_MODEL: OnceLock<String> = OnceLock::new();

/// Get the translator model (cached)
fn get_translator_model() -> &'static str {
    TRANSLATOR_MODEL.get_or_init(|| {
        // Check if the light model is available, fallback to main model
        std::env::var("ANNA_TRANSLATOR_MODEL")
            .unwrap_or_else(|_| DEFAULT_TRANSLATOR_MODEL.to_string())
    })
}

/// User intent extracted from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntent {
    /// What the user wants to do (query, configure, execute, etc.)
    pub action: IntentAction,
    /// The main subject/topic
    pub subject: String,
    /// Specific parameters or details
    pub details: Vec<String>,
    /// Confidence in the interpretation (0.0 - 1.0)
    pub confidence: f32,
    /// Whether this needs confirmation before executing
    pub needs_confirmation: bool,
}

/// Types of actions the user might want
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntentAction {
    /// Query information about the system
    Query,
    /// Configure or change settings
    Configure,
    /// Execute a command or task
    Execute,
    /// Install/remove packages
    Package,
    /// Troubleshoot a problem
    Troubleshoot,
    /// Get help or explanation
    Help,
    /// Change Anna's configuration
    AnnaConfig,
    /// Unknown/needs clarification
    Unknown,
}

impl Default for UserIntent {
    fn default() -> Self {
        Self {
            action: IntentAction::Unknown,
            subject: String::new(),
            details: vec![],
            confidence: 0.0,
            needs_confirmation: false,
        }
    }
}

/// Internal event types that need to be translated to natural language
#[derive(Debug, Clone)]
pub enum InternalEvent {
    /// Ticket created with ID and department
    TicketCreated { id: String, department: String },
    /// Team member assigned
    TeamAssigned { name: String, role: String, message: String },
    /// Team dialogue between members
    TeamDialogue { from: String, to: String, message: String },
    /// Escalation to senior
    Escalation { from: String, to: String, reason: String },
    /// Command being executed
    CommandExec { command: String, purpose: String },
    /// Command result
    CommandResult { command: String, output: String, success: bool },
    /// Wiki search
    WikiSearch { query: String },
    /// Wiki results found
    WikiFound { articles: Vec<String> },
    /// Asking for clarification
    Clarification { question: String, options: Vec<String> },
    /// Error occurred
    Error { message: String, recoverable: bool },
    /// Resolution found
    Resolution { summary: String, commands: Vec<String> },
}

/// Translate user input to intent (fast, uses light model)
pub async fn understand_user_input(input: &str) -> Result<UserIntent> {
    let model = get_translator_model();

    // First, try simple pattern matching for common intents
    if let Some(intent) = quick_intent_match(input) {
        debug!("Translator: quick match for '{}'", input);
        return Ok(intent);
    }

    // Use LLM for complex inputs
    let prompt = format!(
        r#"Analyze this user request and extract the intent. Reply ONLY with a JSON object.

User request: "{}"

Reply format (JSON only, no explanation):
{{
  "action": "query|configure|execute|package|troubleshoot|help|anna_config|unknown",
  "subject": "main topic",
  "details": ["specific", "details"],
  "confidence": 0.0-1.0,
  "needs_confirmation": true/false
}}

JSON:"#,
        input
    );

    // Use shorter timeout for translator (it's a light model)
    let response = ollama::chat_with_timeout(model, &prompt, 10).await?;

    // Parse the JSON response
    parse_intent_response(&response)
}

/// Quick pattern matching for common intents (no LLM needed)
fn quick_intent_match(input: &str) -> Option<UserIntent> {
    let lower = input.to_lowercase();

    // Anna config changes
    if lower.contains("anna") && (lower.contains("debug") || lower.contains("verbose") || lower.contains("quiet")) {
        return Some(UserIntent {
            action: IntentAction::AnnaConfig,
            subject: "debug_mode".to_string(),
            details: vec![input.to_string()],
            confidence: 0.9,
            needs_confirmation: false,
        });
    }

    // Help patterns - check FIRST since "how do I enable" should be Help, not Configure
    if lower.starts_with("how do i") || lower.starts_with("how to") || lower.contains("help") {
        return Some(UserIntent {
            action: IntentAction::Help,
            subject: "howto".to_string(),
            details: vec![input.to_string()],
            confidence: 0.85,
            needs_confirmation: false,
        });
    }

    // Query patterns
    let query_patterns = [
        ("how much", "quantity"),
        ("what is", "info"),
        ("show", "display"),
        ("list", "enumerate"),
        ("check", "status"),
        ("status", "status"),
        ("version", "version"),
    ];

    for (pattern, subject_type) in query_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent {
                action: IntentAction::Query,
                subject: subject_type.to_string(),
                details: vec![input.to_string()],
                confidence: 0.85,
                needs_confirmation: false,
            });
        }
    }

    // Execute patterns (need confirmation)
    let exec_patterns = [
        ("install", IntentAction::Package),
        ("remove", IntentAction::Package),
        ("uninstall", IntentAction::Package),
        ("restart", IntentAction::Execute),
        ("start", IntentAction::Execute),
        ("stop", IntentAction::Execute),
        ("enable", IntentAction::Configure),
        ("disable", IntentAction::Configure),
    ];

    for (pattern, action) in exec_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent {
                action,
                subject: pattern.to_string(),
                details: vec![input.to_string()],
                confidence: 0.85,
                needs_confirmation: true, // These need confirmation
            });
        }
    }

    // Troubleshoot patterns
    let trouble_patterns = ["not working", "broken", "error", "failed", "can't", "cannot", "won't", "doesn't"];
    for pattern in trouble_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent {
                action: IntentAction::Troubleshoot,
                subject: "problem".to_string(),
                details: vec![input.to_string()],
                confidence: 0.8,
                needs_confirmation: false,
            });
        }
    }

    None
}

/// Parse LLM response into UserIntent
fn parse_intent_response(response: &str) -> Result<UserIntent> {
    // Try to extract JSON from response
    let json_start = response.find('{');
    let json_end = response.rfind('}');

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response[start..=end];
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
            let action = match value.get("action").and_then(|a| a.as_str()).unwrap_or("unknown") {
                "query" => IntentAction::Query,
                "configure" => IntentAction::Configure,
                "execute" => IntentAction::Execute,
                "package" => IntentAction::Package,
                "troubleshoot" => IntentAction::Troubleshoot,
                "help" => IntentAction::Help,
                "anna_config" => IntentAction::AnnaConfig,
                _ => IntentAction::Unknown,
            };

            return Ok(UserIntent {
                action,
                subject: value.get("subject").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                details: value.get("details")
                    .and_then(|d| d.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                confidence: value.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5) as f32,
                needs_confirmation: value.get("needs_confirmation").and_then(|n| n.as_bool()).unwrap_or(false),
            });
        }
    }

    // Fallback if parsing fails
    Ok(UserIntent::default())
}

/// Translate internal event to natural language for "fly on the wall" display
pub fn narrate_event(event: &InternalEvent, debug_mode: bool) -> String {
    match event {
        InternalEvent::TicketCreated { id, department } => {
            if debug_mode {
                format!("Ticket {} created, assigned to {} department", id, department)
            } else {
                format!("{} opened -> {} department", id, department)
            }
        }

        InternalEvent::TeamAssigned { name, role, message } => {
            if debug_mode {
                format!("[{}:{}] {}", role, name, message)
            } else {
                format!("Hey {}! {}", name, message)
            }
        }

        InternalEvent::TeamDialogue { from, to, message } => {
            if debug_mode {
                format!("{} -> {}: {}", from, to, message)
            } else {
                format!("{}: {}", from, message)
            }
        }

        InternalEvent::Escalation { from, to, reason } => {
            if debug_mode {
                format!("ESCALATION: {} -> {} ({})", from, to, reason)
            } else {
                format!("I need to check with {}... {}", to, reason)
            }
        }

        InternalEvent::CommandExec { command, purpose } => {
            if debug_mode {
                format!("EXEC: {} ({})", command, purpose)
            } else {
                format!("Let me check... running: {}", command)
            }
        }

        InternalEvent::CommandResult { command: _, output, success } => {
            if debug_mode {
                format!("OUTPUT [{}]: {}", if *success { "OK" } else { "FAIL" }, output)
            } else if *success {
                "Got the information I needed.".to_string()
            } else {
                "Hmm, that didn't work as expected...".to_string()
            }
        }

        InternalEvent::WikiSearch { query } => {
            if debug_mode {
                format!("WIKI SEARCH: {}", query)
            } else {
                "Checking the Arch Wiki...".to_string()
            }
        }

        InternalEvent::WikiFound { articles } => {
            if debug_mode {
                format!("WIKI FOUND: {:?}", articles)
            } else if articles.is_empty() {
                "No relevant wiki articles found.".to_string()
            } else {
                format!("Found some helpful articles: {}", articles.join(", "))
            }
        }

        InternalEvent::Clarification { question, options } => {
            if debug_mode {
                format!("CLARIFY: {} [options: {:?}]", question, options)
            } else {
                question.clone()
            }
        }

        InternalEvent::Error { message, recoverable } => {
            if debug_mode {
                format!("ERROR [recoverable={}]: {}", recoverable, message)
            } else if *recoverable {
                format!("Hit a small snag: {}. Let me try another approach.", message)
            } else {
                format!("I encountered an issue: {}", message)
            }
        }

        InternalEvent::Resolution { summary, commands } => {
            if debug_mode {
                format!("RESOLVED: {} (commands: {:?})", summary, commands)
            } else {
                summary.clone()
            }
        }
    }
}

/// Generate a natural greeting based on context
pub fn generate_greeting(username: &str, last_seen_hours: Option<u64>) -> String {
    let time_greeting = match last_seen_hours {
        Some(h) if h < 1 => "Welcome back!",
        Some(h) if h < 24 => "Hey there!",
        Some(h) if h < 168 => "It's been a while!",
        Some(_) => "Long time no see!",
        None => "Nice to meet you!",
    };

    format!("Hello {}, {}", username, time_greeting)
}

/// Check if translator model is available
pub async fn ensure_translator_model() -> Result<bool> {
    let model = get_translator_model();
    info!("Translator: checking for model '{}'", model);

    // Check if model is available
    let models = ollama::list_models().await?;

    if models.iter().any(|m| m.contains(model)) {
        info!("Translator model '{}' is available", model);
        Ok(true)
    } else {
        info!("Translator model '{}' not found, will use main model", model);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_intent_query() {
        let intent = quick_intent_match("how much disk space do I have").unwrap();
        assert_eq!(intent.action, IntentAction::Query);
        assert!(!intent.needs_confirmation);
    }

    #[test]
    fn test_quick_intent_install() {
        let intent = quick_intent_match("install neovim").unwrap();
        assert_eq!(intent.action, IntentAction::Package);
        assert!(intent.needs_confirmation);
    }

    #[test]
    fn test_quick_intent_troubleshoot() {
        let intent = quick_intent_match("my wifi is not working").unwrap();
        assert_eq!(intent.action, IntentAction::Troubleshoot);
    }

    #[test]
    fn test_quick_intent_help() {
        let intent = quick_intent_match("how do I enable syntax highlighting").unwrap();
        assert_eq!(intent.action, IntentAction::Help);
    }

    #[test]
    fn test_narrate_ticket() {
        let event = InternalEvent::TicketCreated {
            id: "CN-0001".to_string(),
            department: "Desktop".to_string(),
        };
        let narrative = narrate_event(&event, false);
        assert!(narrative.contains("CN-0001"));
        assert!(narrative.contains("Desktop"));
    }
}
