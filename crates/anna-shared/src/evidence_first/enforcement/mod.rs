//! Citation Enforcement - No Citations, No Claims (v0.0.435).
//!
//! Every claim in Anna's response must be backed by evidence.
//! This module validates that claims have supporting citations.

pub mod claim;
pub mod extraction;
pub mod report;
pub mod validation;
pub mod validator;

#[cfg(test)]
mod tests;

pub use claim::{Claim, ClaimType, EvidenceType};
pub use extraction::extract_claims;
pub use report::{Strictness, ValidationReport};
pub use validation::{SupportedClaim, ValidationResult};
pub use validator::ClaimValidator;
