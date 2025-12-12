//! Specialist Response Contract v1 - v0.0.440.
//!
//! Eliminates "Failed to parse specialist response" errors with:
//! - Strict JSON-only schema (Part A)
//! - Validation before accepting (Part B)
//! - Retry strategy with repair prompts (Part C)
//! - Fallback summarizer from evidence only (Part D)
//! - Ticket states and stats integrity (Part E)
//! - Clean UX messages (Part F)

pub mod fallback;
pub mod retry;
pub mod schema;
pub mod ticket_state;
pub mod ux;
pub mod validator;

// Re-export main types for convenience
pub use fallback::{FallbackContext, FallbackReason, FallbackResponse, FallbackSummarizer, ProbeEvidence};
pub use retry::{
    build_repair_prompt, AttemptResult, RetryAttempt, RetryConfig, RetryDecision, RetryState,
    RetrySummary, BACKOFF_1_MS, BACKOFF_2_MS, MAX_RETRIES, REPAIR_PROMPT_1, REPAIR_PROMPT_2,
    SPECIALIST_TIMEOUT_MS,
};
pub use schema::{
    SpecialistResponseV1, SrcAction, SrcActionType, SrcAssessment, SrcCitation, SrcCitationSource,
    SrcDepartment, SrcRisk, MAX_ACTIONS, MAX_CITATIONS, MAX_SNIPPET_CHARS, MAX_SUMMARY_CHARS,
};
pub use ticket_state::{
    ResolutionCriteria, StatsSummary, StateTransition, TicketState, TicketStateMachine,
    TicketStats, MIN_CONFIDENCE_FOR_RESOLVED,
};
pub use ux::{
    fallback_message, retry_message, state_message, success_message, ProgressIndicator,
    UxMessage, UxSeverity,
};
pub use validator::{
    BatchValidator, SrcValidator, ValidationError, ValidationResult, MAX_RESPONSE_CHARS,
    MAX_RESPONSE_TOKENS,
};
