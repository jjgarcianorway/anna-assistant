//! ClaimGate - Blocks user-visible factual claims unless backed by evidence.
//!
//! This is a code-level enforcement mechanism, not just a prompt.
//! Claims must be backed by:
//! - Probe results (structured command output)
//! - Trusted doc citations (Arch Wiki, man pages, --help)
//! - Validated skill artifacts with evidence chains
//!
//! If evidence is missing, ClaimGate switches to Investigator mode.

mod config;
mod gate;
#[cfg(test)]
mod tests;
mod types;
mod verifier;

// Re-exports
pub use config::ClaimGateConfig;
pub use gate::ClaimGate;
pub use types::{Claim, ClaimCategory, EvidenceType, GateResult, SentenceType, TrustedDocSource};
pub use verifier::{ClaimVerifier, VerifiedResponse};
