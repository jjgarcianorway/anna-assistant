//! Transcript event kind types (v0.0.178).

use serde::{Deserialize, Serialize};

use super::StageOutcome;

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
    RevisionApplied { changes_made: Vec<String> },

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
