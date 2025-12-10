//! Transcript event struct and constructors (v0.0.178).

use serde::{Deserialize, Serialize};

use super::{Actor, StageOutcome, TranscriptEventKind};

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

    /// Create a fast path event (v0.0.39)
    pub fn fast_path(
        elapsed_ms: u64,
        handled: bool,
        class: impl Into<String>,
        reason: impl Into<String>,
        probes_needed: bool,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::FastPath {
                handled,
                class: class.into(),
                reason: reason.into(),
                probes_needed,
            },
        }
    }

    // Ticket and review helpers in transcript_ext.rs (v0.0.25/v0.0.26)

    /// Create LLM timeout fallback event (v0.0.41)
    pub fn llm_timeout_fallback(
        elapsed_ms: u64,
        stage: impl Into<String>,
        timeout_secs: u64,
        elapsed_secs: u64,
        fallback_action: impl Into<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::LlmTimeoutFallback {
                stage: stage.into(),
                timeout_secs,
                elapsed_secs,
                fallback_action: fallback_action.into(),
            },
        }
    }

    /// Create graceful degradation event (v0.0.41)
    pub fn graceful_degradation(
        elapsed_ms: u64,
        reason: impl Into<String>,
        original_type: impl Into<String>,
        fallback_type: impl Into<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::GracefulDegradation {
                reason: reason.into(),
                original_type: original_type.into(),
                fallback_type: fallback_type.into(),
            },
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

    /// Create evidence summary event (v0.0.63)
    /// Used in clean mode to show "Checking X data sources..."
    pub fn evidence_summary(
        elapsed_ms: u64,
        evidence_kinds: Vec<String>,
        probe_count: usize,
        key_findings: Vec<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::EvidenceSummary {
                evidence_kinds,
                probe_count,
                key_findings,
            },
        }
    }

    /// Create deterministic path event (v0.0.63)
    pub fn deterministic_path(
        elapsed_ms: u64,
        route_class: impl Into<String>,
        evidence_used: Vec<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::DeterministicPath {
                route_class: route_class.into(),
                evidence_used,
            },
        }
    }

    /// Create proposed action event (v0.0.63)
    pub fn proposed_action(
        elapsed_ms: u64,
        action_id: impl Into<String>,
        description: impl Into<String>,
        risk_level: impl Into<String>,
        rollback_available: bool,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::Anna,
            to: Some(Actor::You),
            kind: TranscriptEventKind::ProposedAction {
                action_id: action_id.into(),
                description: description.into(),
                risk_level: risk_level.into(),
                rollback_available,
            },
        }
    }

    /// Create action confirmation request event (v0.0.63)
    pub fn action_confirmation_request(
        elapsed_ms: u64,
        action_id: impl Into<String>,
        prompt: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::Anna,
            to: Some(Actor::You),
            kind: TranscriptEventKind::ActionConfirmationRequest {
                action_id: action_id.into(),
                prompt: prompt.into(),
                options,
            },
        }
    }

    /// v0.0.302: Create LLM call event (debug mode only)
    pub fn llm_call(
        elapsed_ms: u64,
        stage: impl Into<String>,
        model: impl Into<String>,
        prompt: impl Into<String>,
        response: impl Into<String>,
        duration_ms: u64,
        tokens: Option<u32>,
    ) -> Self {
        Self {
            elapsed_ms,
            from: Actor::System,
            to: None,
            kind: TranscriptEventKind::LlmCall {
                stage: stage.into(),
                model: model.into(),
                prompt: prompt.into(),
                response: response.into(),
                duration_ms,
                tokens,
            },
        }
    }
}
