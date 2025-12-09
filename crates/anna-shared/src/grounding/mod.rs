//! Claim verification against typed evidence (v0.0.195).
//!
//! Computes grounding ratio by verifying extracted claims against
//! ParsedProbeData from STRUCT-lite parsers. No fuzzy matching.
//!
//! # Verification Rules
//!
//! - **Numeric claims**: exact byte equality (no tolerance)
//! - **Percent claims**: exact u8 equality with DiskUsage.percent_used
//! - **Status claims**: exact enum equality with ServiceStatus.state
//! - **Missing evidence**: claim is unverifiable (not counted as verified)
//!
//! v0.0.195: Modularized into domain-focused submodules.

mod tests;
mod types;
mod verify;

// Re-export all types and functions
pub use types::{ClaimVerification, GroundingReport, ParsedEvidence, VerificationReason};
pub use verify::{compute_grounding, is_answer_grounded};
