//! Ticket Integrity Module - v0.0.442.
//!
//! Fix lying stats, clarification order, and package/system confusion.
//!
//! Key problems fixed:
//! 1. Stats showing 100% success with parse errors
//! 2. Clarification running AFTER probes instead of before
//! 3. "do I have swap?" returning "swap package not installed"
//!
//! Rules enforced:
//! - ONLY `Answered` counts as success
//! - Clarify BEFORE probes for config-like intents
//! - Separate system features from packages

pub mod clarification;
pub mod outcome;
pub mod package_system;

#[cfg(test)]
mod tests;

// Re-export main types
pub use clarification::{
    check_clarification_needed, is_clarification_required_intent, ClarificationDecision,
    ClarificationOption, ClarificationQuestion, ClarificationRequiredIntent, FactSource, KnownFact,
    KnownFacts,
};
pub use outcome::{
    is_parse_error, outcome_from_error, AnsweredConditions, HonestTicketStats, TicketOutcome,
    TicketOutcomeRecord, PARSE_ERROR_PATTERNS,
};
pub use package_system::{
    classify_question, PackageIntent, PackageStatus, QuestionClassification, SwapKind, SwapStatus,
    SystemIntent,
};
