//! Specialist Output Contract v2 (Part B) - v0.0.438.
//!
//! Every specialist must output ONLY this JSON, no prose, no markdown.
//! Hard limits:
//! - Max tokens: 220
//! - Max summary: 160 chars
//! - Max notes: 200 chars
//!
//! If exceeded: truncate server-side and set verdict="cannot_answer".

// Re-export all public types and constants from sibling modules
pub use super::specialist_v2_parser::{ParseResult, SpecialistParser};
pub use super::specialist_v2_types::{
    AnswerPayload, SpecialistOutputV2, ValidationResult, Verdict, MAX_ANSWER_FIELDS,
    MAX_NOTES_CHARS, MAX_SPECIALIST_TOKENS, MAX_SUMMARY_CHARS,
};
