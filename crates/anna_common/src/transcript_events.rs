//! Transcript Events v0.0.60 - 3-Tier Event Bus
//!
//! Core event system for all transcript rendering. Each step emits a structured
//! event once, then each mode (human/debug/test) renders differently.
//!
//! Events are:
//! - Always saved to debug log (full detail)
//! - Filtered and humanized for human mode
//! - Shown raw for debug/test mode

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Transcript Mode Configuration
// ============================================================================

/// Transcript rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptMode {
    /// Human-readable IT department dialogue (default)
    /// No tool names, evidence IDs, or raw prompts
    #[default]
    Human,
    /// Full debug output with all internal details
    Debug,
    /// Test mode - same as debug but for automated testing
    Test,
}

impl TranscriptMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptMode::Human => "human",
            TranscriptMode::Debug => "debug",
            TranscriptMode::Test => "test",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => TranscriptMode::Debug,
            "test" => TranscriptMode::Test,
            _ => TranscriptMode::Human,
        }
    }

    /// v0.0.72: Get mode from environment, with precedence
    /// Priority:
    ///   1. ANNA_DEBUG_TRANSCRIPT=1 env var (shorthand for debug mode)
    ///   2. ANNA_UI_TRANSCRIPT_MODE env var (human/debug/test)
    ///   3. Default: Human (config loaded separately by caller)
    pub fn resolve() -> Self {
        // 1. Check ANNA_DEBUG_TRANSCRIPT=1 shorthand (for tests)
        if let Ok(val) = std::env::var("ANNA_DEBUG_TRANSCRIPT") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                return TranscriptMode::Debug;
            }
        }

        // 2. Check ANNA_UI_TRANSCRIPT_MODE env var
        if let Ok(mode) = std::env::var("ANNA_UI_TRANSCRIPT_MODE") {
            return Self::from_str(&mode);
        }

        // 3. Default to human (config loaded separately by caller)
        TranscriptMode::Human
    }

    /// Whether to show internal details (tool names, evidence IDs, etc.)
    pub fn show_internals(&self) -> bool {
        matches!(self, TranscriptMode::Debug | TranscriptMode::Test)
    }
}

// ============================================================================
// Event Actors
// ============================================================================

/// Participants in the dialogue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventActor {
    You,
    Anna,
    Translator,
    Junior,
    Annad,
    Networking,
    Storage,
    Boot,
    Audio,
    Graphics,
    Security,
    Performance,
}

impl EventActor {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventActor::You => "you",
            EventActor::Anna => "anna",
            EventActor::Translator => "translator",
            EventActor::Junior => "junior",
            EventActor::Annad => "annad",
            EventActor::Networking => "networking",
            EventActor::Storage => "storage",
            EventActor::Boot => "boot",
            EventActor::Audio => "audio",
            EventActor::Graphics => "graphics",
            EventActor::Security => "security",
            EventActor::Performance => "performance",
        }
    }
}

impl std::fmt::Display for EventActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Event Kinds
// ============================================================================

/// Type of transcript event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Planning what to do next
    Planning,
    /// Starting a tool call
    ToolCall,
    /// Tool result received
    ToolResult,
    /// Decision made (agree/disagree, routing)
    Decision,
    /// Draft answer being prepared
    Draft,
    /// Final answer to user
    Final,
    /// Warning or issue
    Warning,
    /// Waiting for user confirmation
    Confirmation,
    /// Handoff to another actor/department
    Handoff,
    /// Phase separator (intake, triage, evidence, etc.)
    Phase,
    /// Working indicator (in progress)
    Working,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::Planning => "planning",
            EventKind::ToolCall => "tool_call",
            EventKind::ToolResult => "tool_result",
            EventKind::Decision => "decision",
            EventKind::Draft => "draft",
            EventKind::Final => "final",
            EventKind::Warning => "warning",
            EventKind::Confirmation => "confirmation",
            EventKind::Handoff => "handoff",
            EventKind::Phase => "phase",
            EventKind::Working => "working",
        }
    }
}

// ============================================================================
// Raw Event Data (Internal Details)
// ============================================================================

/// Raw internal data - always saved, only shown in debug/test
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawEventData {
    /// Tool name (e.g., "hw_snapshot_summary")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Evidence ID (e.g., "E1", "E2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,

    /// Raw prompt sent to LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Raw response from LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,

    /// Structured data (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Error message if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Parse warnings or fallback info
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,

    /// Additional key-value metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Human Summary Data
// ============================================================================

/// Human-readable summary - shown in human mode
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HumanSummary {
    /// Main message to display
    pub message: String,

    /// Human description of evidence (e.g., "hardware inventory snapshot")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_description: Option<String>,

    /// Human description of action being taken
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_description: Option<String>,

    /// Progress indicator text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
}

// ============================================================================
// Transcript Event
// ============================================================================

/// A single transcript event - the source of truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    /// Timestamp of event
    pub ts: DateTime<Utc>,

    /// Who generated this event
    pub actor: EventActor,

    /// Type of event
    pub kind: EventKind,

    /// Raw internal data (always saved, shown in debug/test)
    pub raw: RawEventData,

    /// Human summary (shown in human mode)
    pub summary: HumanSummary,
}

impl TranscriptEvent {
    /// Create a new event
    pub fn new(actor: EventActor, kind: EventKind) -> Self {
        Self {
            ts: Utc::now(),
            actor,
            kind,
            raw: RawEventData::default(),
            summary: HumanSummary::default(),
        }
    }

    /// Builder: set human message
    pub fn with_message(mut self, msg: &str) -> Self {
        self.summary.message = msg.to_string();
        self
    }

    /// Builder: set tool name (internal)
    pub fn with_tool(mut self, name: &str) -> Self {
        self.raw.tool_name = Some(name.to_string());
        self
    }

    /// Builder: set evidence ID (internal)
    pub fn with_evidence_id(mut self, id: &str) -> Self {
        self.raw.evidence_id = Some(id.to_string());
        self
    }

    /// Builder: set evidence description (human)
    pub fn with_evidence_desc(mut self, desc: &str) -> Self {
        self.summary.evidence_description = Some(desc.to_string());
        self
    }

    /// Builder: set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.raw.duration_ms = Some(ms);
        self
    }

    /// Builder: set error
    pub fn with_error(mut self, err: &str) -> Self {
        self.raw.error = Some(err.to_string());
        self
    }

    /// Builder: set prompt (internal)
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.raw.prompt = Some(prompt.to_string());
        self
    }

    /// Builder: set progress text
    pub fn with_progress(mut self, progress: &str) -> Self {
        self.summary.progress = Some(progress.to_string());
        self
    }

    /// Builder: add warning
    pub fn with_warning(mut self, warning: &str) -> Self {
        self.raw.warnings.push(warning.to_string());
        self
    }

    /// Builder: add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.raw.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Builder: set raw data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.raw.data = Some(data);
        self
    }
}

// ============================================================================
// Event Stream / Collector
// ============================================================================

/// Collects events during a case execution
#[derive(Debug, Clone, Default)]
pub struct TranscriptEventStream {
    events: Vec<TranscriptEvent>,
    case_id: Option<String>,
}

impl TranscriptEventStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_case_id(mut self, case_id: &str) -> Self {
        self.case_id = Some(case_id.to_string());
        self
    }

    /// Add an event to the stream
    pub fn push(&mut self, event: TranscriptEvent) {
        self.events.push(event);
    }

    /// Get all events
    pub fn events(&self) -> &[TranscriptEvent] {
        &self.events
    }

    /// Get case ID if set
    pub fn case_id(&self) -> Option<&str> {
        self.case_id.as_deref()
    }

    /// Number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_mode_from_env() {
        // Clean up any existing env vars first
        std::env::remove_var("ANNA_DEBUG_TRANSCRIPT");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");

        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "debug");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "test");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Test);

        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "human");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Human);

        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
    }

    #[test]
    fn test_transcript_mode_debug_shorthand() {
        // Clean up
        std::env::remove_var("ANNA_DEBUG_TRANSCRIPT");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");

        // ANNA_DEBUG_TRANSCRIPT=1 should enable debug mode
        std::env::set_var("ANNA_DEBUG_TRANSCRIPT", "1");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        // ANNA_DEBUG_TRANSCRIPT=true should also work
        std::env::set_var("ANNA_DEBUG_TRANSCRIPT", "true");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        // ANNA_DEBUG_TRANSCRIPT takes precedence over ANNA_UI_TRANSCRIPT_MODE
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "human");
        std::env::set_var("ANNA_DEBUG_TRANSCRIPT", "1");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        // Clean up
        std::env::remove_var("ANNA_DEBUG_TRANSCRIPT");
        std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
    }

    #[test]
    fn test_event_builder() {
        let event = TranscriptEvent::new(EventActor::Anna, EventKind::ToolCall)
            .with_message("Checking hardware inventory")
            .with_tool("hw_snapshot_summary")
            .with_evidence_id("E1")
            .with_evidence_desc("hardware inventory snapshot");

        assert_eq!(event.actor, EventActor::Anna);
        assert_eq!(event.kind, EventKind::ToolCall);
        assert_eq!(event.summary.message, "Checking hardware inventory");
        assert_eq!(event.raw.tool_name, Some("hw_snapshot_summary".to_string()));
        assert_eq!(event.raw.evidence_id, Some("E1".to_string()));
    }

    #[test]
    fn test_show_internals() {
        assert!(!TranscriptMode::Human.show_internals());
        assert!(TranscriptMode::Debug.show_internals());
        assert!(TranscriptMode::Test.show_internals());
    }
}
