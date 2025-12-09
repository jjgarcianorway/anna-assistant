//! Revision instructions for junior/senior feedback (v0.0.208).
//!
//! Junior and Senior reviewers return revision instructions (not direct answers)
//! to keep Anna as the visible owner of the response.
//!
//! Re-exports team-aware ReviewArtifact from the review module for gradual migration.
//!
//! v0.0.208: Modularized into domain-focused submodules.

mod conversion;
mod junior;
mod senior;
mod tests;
mod types;

// Re-export all types and functions
pub use conversion::{junior_to_review_artifact, senior_to_review_artifact};
pub use junior::JuniorVerification;
pub use senior::SeniorEscalation;
pub use types::{RevisionInstruction, RevisionIssue};

// Re-export new team-aware review types
pub use crate::review::{
    ReviewArtifact, ReviewIssue, ReviewIssueKind, ReviewRevision, ReviewSeverity,
};
