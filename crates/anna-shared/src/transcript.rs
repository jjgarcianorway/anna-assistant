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
    Junior,     // Junior reviewer (v0.0.25 tickets)
    Senior,     // Senior reviewer (v0.0.25 tickets)
    Annad,      // Daemon for probe execution (v0.0.25 tickets)
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
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
            Self::Annad => write!(f, "annad"),
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
    /// Clarification required before proceeding (v0.45.5)
    /// Stage paused waiting for user to select from verified choices
    ClarificationRequired {
        /// The question prompt
        question: String,
        /// Available choices (verified against evidence)
        choices: Vec<String>,
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

    /// Create a ClarificationRequired outcome (v0.45.5).
    pub fn clarification_required(question: impl Into<String>, choices: Vec<String>) -> Self {
        Self::ClarificationRequired {
            question: question.into(),
            choices,
        }
    }

    /// Check if this outcome represents a budget exceeded condition.
    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded { .. })
    }

    /// Check if this outcome requires user clarification (v0.45.5).
    pub fn is_clarification_required(&self) -> bool {
        matches!(self, Self::ClarificationRequired { .. })
    }

    /// Check if this outcome allows the stage to proceed without user input.
    pub fn can_proceed(&self) -> bool {
        matches!(self, Self::Ok | Self::Deterministic)
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
            Self::ClarificationRequired { question, choices } => {
                write!(f, "clarification_required({}, {} choices)", question, choices.len())
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

    // === Ticket lifecycle events (v0.0.25) ===

    /// Ticket created from user request
    TicketCreated {
        ticket_id: String,
        domain: String,
        intent: String,
        evidence_required: bool,
    },
    /// Ticket status changed
    TicketStatusChanged {
        ticket_id: String,
        from_status: String,
        to_status: String,
    },
    /// Junior review result
    JuniorReview {
        attempt: u8,
        score: u8,
        verified: bool,
        issues: Vec<String>,
    },
    /// Senior escalation
    SeniorEscalation {
        successful: bool,
        reason: Option<String>,
    },
    /// Revision applied based on instruction
    RevisionApplied {
        changes_made: Vec<String>,
    },

    // === Review gate events (v0.0.26) ===

    /// Review gate decision
    ReviewGateDecision {
        /// Decision made by the gate
        decision: String,
        /// Reliability score used
        score: u8,
        /// Whether LLM review is required
        requires_llm: bool,
    },
    /// Team review exchange
    TeamReview {
        /// Team that performed the review
        team: String,
        /// Reviewer type ("junior", "senior", or "deterministic")
        reviewer: String,
        /// Decision made
        decision: String,
        /// Number of issues found
        issues_count: usize,
    },

    // === Clarification events (v0.0.31) ===

    /// Clarification question asked
    ClarificationAsked {
        /// Question ID
        question_id: String,
        /// The question prompt
        prompt: String,
        /// Available choices (if any)
        choices: Vec<String>,
        /// Reason clarification is needed
        reason: String,
    },
    /// User provided clarification answer
    ClarificationAnswered {
        /// Question ID
        question_id: String,
        /// User's answer
        answer: String,
    },
    /// Clarification verification result
    ClarificationVerified {
        /// Question ID
        question_id: String,
        /// Whether verification succeeded
        verified: bool,
        /// Verification source
        source: String,
        /// Alternative options (if verification failed)
        alternatives: Vec<String>,
    },
    /// Fact stored from verified clarification
    FactStored {
        /// Fact key
        key: String,
        /// Fact value
        value: String,
        /// How the fact was verified
        source: String,
    },

    // === Fast path events (v0.0.39) ===

    /// Fast path evaluation result
    FastPath {
        /// Whether fast path handled the query
        handled: bool,
        /// Fast path class (e.g., "system_health", "disk_usage")
        class: String,
        /// Reason for decision
        reason: String,
        /// Whether probes were needed
        probes_needed: bool,
    },

    // === Timeout fallback events (v0.0.41) ===

    /// LLM timeout triggered fallback (v0.0.41)
    LlmTimeoutFallback {
        /// Stage that timed out ("translator" or "specialist")
        stage: String,
        /// Timeout duration in seconds
        timeout_secs: u64,
        /// Actual elapsed time in seconds
        elapsed_secs: u64,
        /// Fallback action taken
        fallback_action: String,
    },
    /// Graceful degradation applied (v0.0.41)
    GracefulDegradation {
        /// Reason for degradation
        reason: String,
        /// Original intended response type
        original_type: String,
        /// Fallback response type
        fallback_type: String,
    },

    // === Service Desk Theatre events (v0.0.63) ===

    /// Evidence summary - what probes found without raw output (v0.0.63)
    /// Used in clean mode to show "Checking X data sources..." without leaking probe output
    EvidenceSummary {
        /// Types of evidence gathered (e.g., ["audio", "tool_exists"])
        evidence_kinds: Vec<String>,
        /// Number of probes executed
        probe_count: usize,
        /// Key findings in human-readable form (no raw output)
        key_findings: Vec<String>,
    },
    /// Deterministic route taken (v0.0.63)
    /// Shows which deterministic path was used to answer
    DeterministicPath {
        /// Route class (e.g., "hardware_audio", "configure_editor")
        route_class: String,
        /// Evidence kinds used for the answer
        evidence_used: Vec<String>,
    },
    /// Proposed action requiring user confirmation (v0.0.63)
    /// Used for privileged actions that need explicit approval
    ProposedAction {
        /// Unique action identifier
        action_id: String,
        /// Human-readable description of the action
        description: String,
        /// Risk level: "low", "medium", "high"
        risk_level: String,
        /// Whether rollback is available
        rollback_available: bool,
    },
    /// Confirmation request for proposed action (v0.0.63)
    ActionConfirmationRequest {
        /// Action ID this confirms
        action_id: String,
        /// Confirmation prompt
        prompt: String,
        /// Available options (e.g., ["yes", "no", "show diff"])
        options: Vec<String>,
    },

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
