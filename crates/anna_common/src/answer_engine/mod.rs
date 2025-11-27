//! Answer Engine v0.12.0
//!
//! Disciplined, auditable answer engine with:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised second-pass auditor (LLM-B)
//! - Transparent reliability scoring
//! - Automatic back-and-forth until score is high enough
//! - Partial answer fallback instead of total refusal
//! - Strict probe catalog enforcement

pub mod evidence;
pub mod protocol;
pub mod scoring;

pub use evidence::*;
pub use protocol::*;
pub use scoring::*;
