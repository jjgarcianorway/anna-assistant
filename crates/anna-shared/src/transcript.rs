//! Transcript event model for consistent pipeline visibility.
//!
//! Single source of truth for rendering request/response conversations.
//! Enforces size cap with diagnostic surfacing (COST phase).

use crate::resource_limits::{ResourceDiagnostic, MAX_TRANSCRIPT_EVENTS};
use serde::{Deserialize, Serialize};

/// Actor in the transcript (who is speaking/acting)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    You,        // The user
    Anna,       // Anna's final response
    Translator, // LLM translator stage
    Dispatcher, // Probe dispatcher
    Probe,      // System probe execution
    Specialist, // Domain specialist LLM
    Supervisor, // Quality/reliability validator
    System,     // System messages (errors, timeouts, etc.)
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::You => write!(f, "you"),
            Self::Anna => write!(f, "anna"),
            Self::Translator => write!(f, "translator"),
            Self::Dispatcher => write!(f, "dispatcher"),
            Self::Probe => write!(f, "probe"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
            Self::System => write!(f, "system"),
        }
    }
}

/// Stage outcome for StageEnd events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageOutcome {
    Ok,
    Timeout,
    Error,
    Skipped,
    Deterministic, // Used when deterministic router answered
    /// Stage budget exceeded (METER phase)
    /// Distinct from Timeout: budget is stage-level, timeout is operation-level
    BudgetExceeded {
        /// Which stage exceeded its budget
        stage: String,
        /// Budget in milliseconds
        budget_ms: u64,
        /// Actual elapsed time in milliseconds
        elapsed_ms: u64,
    },
}

impl StageOutcome {
    /// Create a BudgetExceeded outcome.
    pub fn budget_exceeded(stage: impl Into<String>, budget_ms: u64, elapsed_ms: u64) -> Self {
        Self::BudgetExceeded {
            stage: stage.into(),
            budget_ms,
            elapsed_ms,
        }
    }

    /// Check if this outcome represents a budget exceeded condition.
    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded { .. })
    }
}

impl std::fmt::Display for StageOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Timeout => write!(f, "timeout"),
            Self::Error => write!(f, "error"),
            Self::Skipped => write!(f, "skipped"),
            Self::Deterministic => write!(f, "deterministic"),
            Self::BudgetExceeded { stage, budget_ms, elapsed_ms } => {
                write!(f, "budget_exceeded({}: {}ms > {}ms)", stage, elapsed_ms, budget_ms)
            }
        }
    }
}

/// Kind of transcript event
///
/// WIRE COMPATIBILITY: The `Unknown` variant with `#[serde(other)]` ensures
/// older clients can deserialize transcripts containing new event kinds
/// without crashing. New kinds should be added BEFORE `Unknown`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TranscriptEventKind {
    /// A message from one actor to another (general conversation)
    Message { text: String },
    /// The final answer to the user's query (THE discriminator for answer source)
    /// This is the authoritative "Anna's response" - not Message, not Note.
    FinalAnswer { text: String },
    /// Stage starting
    StageStart { stage: String },
    /// Stage ending with outcome
    StageEnd {
        stage: String,
        outcome: StageOutcome,
    },
    /// Probe execution starting
    ProbeStart { probe_id: String, command: String },
    /// Probe execution ending
    ProbeEnd {
        probe_id: String,
        exit_code: i32,
        timing_ms: u64,
        stdout_preview: Option<String>,
    },
    /// Metadata note (debug mode only)
    Note { text: String },
    /// Unknown event kind (forward compatibility)
    /// Deserializes any unrecognized "type" value - old clients won't crash on new kinds.
    #[serde(other)]
    Unknown,
}

/// A single transcript event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    /// Elapsed time since request started (ms)
    pub elapsed_ms: u64,
    /// Who is acting/speaking
    pub from: Actor,
    /// Who they are addressing (optional for broadcasts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Actor>,
    /// The event content
    pub kind: TranscriptEventKind,
}

impl TranscriptEvent {
    /// Create a message event (general conversation, NOT the final answer)
    pub fn message(elapsed_ms: u64, from: Actor, to: Actor, text: impl Into<String>) -> Self {
        Self {
            elapsed_ms,
            from,
            to: Some(to),
            kind: TranscriptEventKind::Message { text: text.into() },
        }
    }

    /// Create the final answer event (THE authoritative Anna response)
    /// This is the discriminator for answer source - use this, not message(), for Anna's answer.
    pub fn final_answer(elapsed_ms: u64, text: impl Into<String>) -> Self {
        Self {
            elapsed_ms,
            from: Actor::Anna,
            to: Some(Actor::You),
            kind: TranscriptEventKind::FinalAnswer { text: text.into() },
        }
    }

    /// Create a stage start event
    pub fn stage_start(elapsed_ms: u64, stage: impl Into<String>) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::StageStart {
                stage: stage.into(),
            },
        }
    }

    /// Create a stage end event
    pub fn stage_end(elapsed_ms: u64, stage: impl Into<String>, outcome: StageOutcome) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::StageEnd {
                stage: stage.into(),
                outcome,
            },
        }
    }

    /// Create a probe start event
    pub fn probe_start(
        elapsed_ms: u64,
        probe_id: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::Dispatcher,
            to: Some(Actor::Probe),
            kind: TranscriptEventKind::ProbeStart {
                probe_id: probe_id.into(),
                command: command.into(),
            },
        }
    }

    /// Create a probe end event
    pub fn probe_end(
        elapsed_ms: u64,
        probe_id: impl Into<String>,
        exit_code: i32,
        timing_ms: u64,
        stdout_preview: Option<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::Probe,
            to: Some(Actor::Dispatcher),
            kind: TranscriptEventKind::ProbeEnd {
                probe_id: probe_id.into(),
                exit_code,
                timing_ms,
                stdout_preview,
            },
        }
    }

    /// Create a note event (debug only)
    pub fn note(elapsed_ms: u64, text: impl Into<String>) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::Note { text: text.into() },
        }
    }

    /// Check if this is a debug-only event
    pub fn is_debug_only(&self) -> bool {
        matches!(
            self.kind,
            TranscriptEventKind::Note { .. }
                | TranscriptEventKind::StageStart { .. }
                | TranscriptEventKind::StageEnd { .. }
        )
    }
}

/// Full transcript for a request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transcript {
    /// All events in chronological order
    pub events: Vec<TranscriptEvent>,
    /// Number of events dropped due to cap (not serialized for wire compat)
    #[serde(skip)]
    dropped_events: usize,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            dropped_events: 0,
        }
    }

    /// Push event, enforcing cap. Returns true if event was added.
    /// COST: Never silently truncate - track dropped count for diagnostic.
    pub fn push(&mut self, event: TranscriptEvent) -> bool {
        if self.events.len() >= MAX_TRANSCRIPT_EVENTS {
            self.dropped_events += 1;
            false
        } else {
            self.events.push(event);
            true
        }
    }

    /// Check if transcript was capped (events were dropped)
    pub fn was_capped(&self) -> bool {
        self.dropped_events > 0
    }

    /// Get number of dropped events
    pub fn dropped_count(&self) -> usize {
        self.dropped_events
    }

    /// Get resource diagnostic if capped
    pub fn diagnostic(&self) -> Option<ResourceDiagnostic> {
        if self.dropped_events > 0 {
            Some(ResourceDiagnostic::transcript_capped(self.dropped_events))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}
// Tests are in tests/transcript_tests.rs
