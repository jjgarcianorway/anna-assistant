//! EventBus types - Event definitions and enums.

use serde::{Deserialize, Serialize};

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

    // === Ticket Events (Phase 10) ===
    /// Ticket lifecycle event for specialist dispatch
    TicketLifecycle(TicketEvent),
}

/// Ticket lifecycle events for fly-on-the-wall display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ticket_event", content = "data")]
pub enum TicketEvent {
    /// New ticket created.
    Created {
        ticket_id: String,
        department: String,
        question_summary: String,
    },
    /// Ticket assigned to specialist.
    Assigned {
        ticket_id: String,
        specialist_id: String,
        specialist_name: String,
        department: String,
    },
    /// Specialist working on ticket.
    Working {
        ticket_id: String,
        specialist_id: String,
        action: String,
    },
    /// Ticket escalated to senior.
    Escalated {
        ticket_id: String,
        from_specialist: String,
        to_specialist: String,
        reason: String,
    },
    /// Ticket resolved successfully.
    Resolved {
        ticket_id: String,
        specialist_id: String,
        specialist_name: String,
        confidence: f32,
        learned_recipe: bool,
    },
    /// Ticket failed.
    Failed {
        ticket_id: String,
        specialist_id: Option<String>,
        reason: String,
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
