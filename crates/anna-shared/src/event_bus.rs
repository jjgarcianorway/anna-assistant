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

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event types emitted by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum Event {
    // === Step Events ===
    /// A processing step has started
    StepStarted {
        step_id: String,
        step_type: StepType,
        description: String,
    },
    /// A processing step has finished
    StepFinished {
        step_id: String,
        step_type: StepType,
        duration_ms: u64,
        success: bool,
    },

    // === Probe Events ===
    /// A system probe is starting
    ProbeStarted {
        probe_id: String,
        command: String,
        /// Redacted command for display (hides sensitive args)
        display_command: String,
    },
    /// A system probe has finished
    ProbeFinished {
        probe_id: String,
        exit_code: i32,
        /// Redacted output summary
        output_summary: String,
        duration_ms: u64,
    },

    // === LLM Events ===
    /// LLM request is starting
    LlmStarted {
        request_id: String,
        purpose: LlmPurpose,
        model: String,
    },
    /// A token has been received from the LLM (streaming)
    LlmToken {
        request_id: String,
        token: String,
    },
    /// LLM request has finished
    LlmFinished {
        request_id: String,
        duration_ms: u64,
        tokens_used: Option<u32>,
        success: bool,
    },

    // === Skill Events ===
    /// A skill candidate has been created
    SkillCandidateCreated {
        skill_id: String,
        name: String,
        description: String,
    },
    /// A skill has been validated
    SkillValidated {
        skill_id: String,
        tests_passed: u32,
        tests_total: u32,
    },
    /// A skill has been promoted to trusted
    SkillPromoted {
        skill_id: String,
        tier: String,
    },

    // === Status Events ===
    /// Warning condition detected
    Warning {
        code: String,
        message: String,
        source: Option<String>,
    },
    /// Error condition detected
    Error {
        code: String,
        message: String,
        source: Option<String>,
        recoverable: bool,
    },

    // === Progress Events ===
    /// Generic progress update
    Progress {
        operation: String,
        current: u64,
        total: Option<u64>,
        message: Option<String>,
    },

    // === Answer Events ===
    /// Final answer is ready
    AnswerReady {
        answer: String,
        confidence: f32,
        citations: Vec<String>,
    },
    /// Answer requires investigation
    InvestigationNeeded {
        reason: String,
        suggested_probes: Vec<String>,
    },
}

/// Types of processing steps
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    /// Intent classification
    IntentClassification,
    /// Question decomposition
    QuestionDecomposition,
    /// Knowledge retrieval
    KnowledgeRetrieval,
    /// Wiki search
    WikiSearch,
    /// Command generation
    CommandGeneration,
    /// Command execution
    CommandExecution,
    /// Output validation
    OutputValidation,
    /// Answer generation
    AnswerGeneration,
    /// Claim verification
    ClaimVerification,
    /// Memory update
    MemoryUpdate,
}

impl StepType {
    pub fn display_name(&self) -> &'static str {
        match self {
            StepType::IntentClassification => "understanding question",
            StepType::QuestionDecomposition => "breaking down question",
            StepType::KnowledgeRetrieval => "searching knowledge",
            StepType::WikiSearch => "checking documentation",
            StepType::CommandGeneration => "planning commands",
            StepType::CommandExecution => "running commands",
            StepType::OutputValidation => "validating output",
            StepType::AnswerGeneration => "generating answer",
            StepType::ClaimVerification => "verifying claims",
            StepType::MemoryUpdate => "updating memory",
        }
    }
}

/// Purpose of LLM request
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LlmPurpose {
    /// Intent classification
    IntentClassification,
    /// Command selection
    CommandSelection,
    /// Output validation
    Validation,
    /// Answer generation
    AnswerGeneration,
    /// Clarification
    Clarification,
    /// General
    General,
}

impl LlmPurpose {
    pub fn display_name(&self) -> &'static str {
        match self {
            LlmPurpose::IntentClassification => "understanding",
            LlmPurpose::CommandSelection => "planning",
            LlmPurpose::Validation => "validating",
            LlmPurpose::AnswerGeneration => "answering",
            LlmPurpose::Clarification => "clarifying",
            LlmPurpose::General => "thinking",
        }
    }
}

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

    // === Convenience methods ===

    /// Emit step started event
    pub fn step_started(&self, step_type: StepType, description: &str) -> String {
        let step_id = uuid::Uuid::new_v4().to_string();
        self.emit(Event::StepStarted {
            step_id: step_id.clone(),
            step_type,
            description: description.to_string(),
        });
        step_id
    }

    /// Emit step finished event
    pub fn step_finished(&self, step_id: &str, step_type: StepType, duration_ms: u64, success: bool) {
        self.emit(Event::StepFinished {
            step_id: step_id.to_string(),
            step_type,
            duration_ms,
            success,
        });
    }

    /// Emit probe started event
    pub fn probe_started(&self, command: &str) -> String {
        let probe_id = uuid::Uuid::new_v4().to_string();
        let display_command = redact_command(command);
        self.emit(Event::ProbeStarted {
            probe_id: probe_id.clone(),
            command: command.to_string(),
            display_command,
        });
        probe_id
    }

    /// Emit probe finished event
    pub fn probe_finished(&self, probe_id: &str, exit_code: i32, output: &str, duration_ms: u64) {
        let output_summary = redact_output(output);
        self.emit(Event::ProbeFinished {
            probe_id: probe_id.to_string(),
            exit_code,
            output_summary,
            duration_ms,
        });
    }

    /// Emit LLM started event
    pub fn llm_started(&self, purpose: LlmPurpose, model: &str) -> String {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.emit(Event::LlmStarted {
            request_id: request_id.clone(),
            purpose,
            model: model.to_string(),
        });
        request_id
    }

    /// Emit LLM token event
    pub fn llm_token(&self, request_id: &str, token: &str) {
        self.emit(Event::LlmToken {
            request_id: request_id.to_string(),
            token: token.to_string(),
        });
    }

    /// Emit LLM finished event
    pub fn llm_finished(&self, request_id: &str, duration_ms: u64, tokens_used: Option<u32>, success: bool) {
        self.emit(Event::LlmFinished {
            request_id: request_id.to_string(),
            duration_ms,
            tokens_used,
            success,
        });
    }

    /// Emit warning event
    pub fn warning(&self, code: &str, message: &str, source: Option<&str>) {
        self.emit(Event::Warning {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
        });
    }

    /// Emit error event
    pub fn error(&self, code: &str, message: &str, source: Option<&str>, recoverable: bool) {
        self.emit(Event::Error {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            recoverable,
        });
    }

    /// Emit progress event
    pub fn progress(&self, operation: &str, current: u64, total: Option<u64>, message: Option<&str>) {
        self.emit(Event::Progress {
            operation: operation.to_string(),
            current,
            total,
            message: message.map(|s| s.to_string()),
        });
    }

    /// Emit answer ready event
    pub fn answer_ready(&self, answer: &str, confidence: f32, citations: Vec<String>) {
        self.emit(Event::AnswerReady {
            answer: answer.to_string(),
            confidence,
            citations,
        });
    }

    /// Emit investigation needed event
    pub fn investigation_needed(&self, reason: &str, suggested_probes: Vec<String>) {
        self.emit(Event::InvestigationNeeded {
            reason: reason.to_string(),
            suggested_probes,
        });
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
