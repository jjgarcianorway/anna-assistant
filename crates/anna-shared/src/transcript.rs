//! Transcript event model for consistent pipeline visibility.
//!
//! Single source of truth for rendering request/response conversations.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageOutcome {
    Ok,
    Timeout,
    Error,
    Skipped,
    Deterministic, // Used when deterministic router answered
}

impl std::fmt::Display for StageOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Timeout => write!(f, "timeout"),
            Self::Error => write!(f, "error"),
            Self::Skipped => write!(f, "skipped"),
            Self::Deterministic => write!(f, "deterministic"),
        }
    }
}

/// Kind of transcript event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TranscriptEventKind {
    /// A message from one actor to another
    Message { text: String },
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
    /// Create a message event
    pub fn message(elapsed_ms: u64, from: Actor, to: Actor, text: impl Into<String>) -> Self {
        Self {
            elapsed_ms,
            from,
            to: Some(to),
            kind: TranscriptEventKind::Message { text: text.into() },
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
}

impl Transcript {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: TranscriptEvent) {
        self.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_display() {
        assert_eq!(format!("{}", Actor::You), "you");
        assert_eq!(format!("{}", Actor::Anna), "anna");
        assert_eq!(format!("{}", Actor::System), "system");
    }

    #[test]
    fn test_transcript_event_creation() {
        let event = TranscriptEvent::message(100, Actor::You, Actor::Anna, "test query");
        assert_eq!(event.elapsed_ms, 100);
        assert_eq!(event.from, Actor::You);
        assert_eq!(event.to, Some(Actor::Anna));
    }

    #[test]
    fn test_is_debug_only() {
        let note = TranscriptEvent::note(0, "debug info");
        assert!(note.is_debug_only());

        let message = TranscriptEvent::message(0, Actor::Anna, Actor::You, "answer");
        assert!(!message.is_debug_only());
    }

    #[test]
    fn test_transcript_push_and_len() {
        let mut transcript = Transcript::new();
        assert!(transcript.is_empty());

        transcript.push(TranscriptEvent::message(
            0,
            Actor::You,
            Actor::Anna,
            "hello",
        ));
        assert_eq!(transcript.len(), 1);

        transcript.push(TranscriptEvent::stage_start(100, "translator"));
        assert_eq!(transcript.len(), 2);
    }

    #[test]
    fn test_transcript_serialization() {
        let mut transcript = Transcript::new();
        transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "test"));
        transcript.push(TranscriptEvent::stage_end(
            100,
            "translator",
            StageOutcome::Ok,
        ));

        let json = serde_json::to_string(&transcript).unwrap();
        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("\"type\":\"stage_end\""));

        let parsed: Transcript = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_probe_events() {
        let start = TranscriptEvent::probe_start(50, "top_mem", "ps aux --sort=-%mem");
        let end = TranscriptEvent::probe_end(100, "top_mem", 0, 50, Some("first line".to_string()));

        if let TranscriptEventKind::ProbeStart { probe_id, command } = &start.kind {
            assert_eq!(probe_id, "top_mem");
            assert_eq!(command, "ps aux --sort=-%mem");
        } else {
            panic!("Expected ProbeStart");
        }

        if let TranscriptEventKind::ProbeEnd {
            exit_code,
            timing_ms,
            ..
        } = &end.kind
        {
            assert_eq!(*exit_code, 0);
            assert_eq!(*timing_ms, 50);
        } else {
            panic!("Expected ProbeEnd");
        }
    }

    #[test]
    fn test_stage_outcome_display() {
        assert_eq!(format!("{}", StageOutcome::Ok), "ok");
        assert_eq!(format!("{}", StageOutcome::Timeout), "timeout");
        assert_eq!(format!("{}", StageOutcome::Error), "error");
        assert_eq!(format!("{}", StageOutcome::Skipped), "skipped");
        assert_eq!(format!("{}", StageOutcome::Deterministic), "deterministic");
    }
}
