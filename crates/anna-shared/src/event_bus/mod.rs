//! EventBus - Single source of truth for progress events.
//!
//! All progress updates flow through this bus. annactl renders these
//! consistently across one-shot, REPL, status, stats, reset, etc.
//!
//! Events:
//! - step_started/step_finished
//! - probe_started/probe_finished (with redaction)
//! - llm_started/llm_token/llm_finished
//! - skill_candidate_created/validated/promoted
//! - warning/error

mod handlers;
mod types;

pub use types::{Event, LlmPurpose, StepType, TicketEvent};

use std::sync::Arc;
use tokio::sync::broadcast;

/// The EventBus - broadcasts events to all listeners
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new EventBus
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Emit an event
    pub fn emit(&self, event: Event) {
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Redact sensitive information from commands
fn redact_command(command: &str) -> String {
    // Redact patterns that might contain sensitive info
    let patterns = [
        // Passwords in URLs
        (r"://[^:]+:[^@]+@", "://<redacted>@"),
        // API keys
        (r"(?i)(api[_-]?key|token|secret|password)=\S+", "$1=<redacted>"),
        // Private key files
        (r"/\.ssh/\S+", "/ssh/<redacted>"),
        // Home directory paths
        (r"/home/[^/\s]+", "/home/<user>"),
    ];

    let mut result = command.to_string();
    for (pattern, replacement) in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    result
}

/// Truncate string for display.
fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Redact sensitive information from output
fn redact_output(output: &str) -> String {
    // Truncate long output
    let max_len = 200;
    let truncated = if output.len() > max_len {
        format!("{}... ({} chars)", &output[..max_len], output.len())
    } else {
        output.to_string()
    };

    // Apply same redaction patterns
    redact_command(&truncated)
}

/// Shared event bus type for passing between components
pub type SharedEventBus = Arc<EventBus>;

/// Create a shared event bus
pub fn create_event_bus() -> SharedEventBus {
    Arc::new(EventBus::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_basic() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.emit(Event::Warning {
            code: "TEST".to_string(),
            message: "test warning".to_string(),
            source: None,
        });

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, Event::Warning { .. }));
    }

    #[test]
    fn test_redact_command() {
        let cmd = "curl https://user:password@example.com";
        let redacted = redact_command(cmd);
        assert!(!redacted.contains("password"));
        assert!(redacted.contains("<redacted>"));
    }

    #[test]
    fn test_step_type_names() {
        assert_eq!(StepType::CommandExecution.display_name(), "running commands");
    }
}
