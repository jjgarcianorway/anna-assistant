//! Specialist Response Contract v1 (Part A) - v0.0.440.
//!
//! Every specialist MUST output ONLY this JSON, nothing else.
//! No markdown. No tables. No extra keys.
//!
//! Hard limits:
//! - summary: max 140 chars
//! - actions: max 5
//! - citations: max 5

pub mod action;
pub mod assessment;
pub mod citation;
pub mod constants;
pub mod response;
pub mod types;

// Re-export all public types for convenience
pub use action::SrcAction;
pub use assessment::SrcAssessment;
pub use citation::SrcCitation;
pub use constants::{
    truncate_str, MAX_ACTIONS, MAX_CITATIONS, MAX_SNIPPET_CHARS, MAX_SUMMARY_CHARS,
};
pub use response::SpecialistResponseV1;
pub use types::{SrcActionType, SrcCitationSource, SrcDepartment, SrcRisk};
