//! Team-aware review protocol for Junior/Senior specialists (v0.0.222).
//!
//! ReviewArtifact is the unified schema for all team reviews.
//! Supports both deterministic gate reviews and LLM escalation reviews.
//!
//! Pinned ordering for deterministic serialization.
//!
//! v0.0.222: Modularized into domain-focused submodules.

mod artifact;
mod inputs;
mod issue;
mod types;

// Re-export for backwards compatibility
pub use artifact::ReviewArtifact;
pub use inputs::ReviewInputsSummary;
pub use issue::{ReviewIssue, ReviewRevision};
pub use types::{ReviewDecision, ReviewIssueKind, ReviewSeverity, ReviewerType};

// Tests: tests/review_tests.rs
