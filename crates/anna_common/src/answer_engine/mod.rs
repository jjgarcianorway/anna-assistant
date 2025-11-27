//! Answer Engine v0.10.0
//!
//! Disciplined, auditable answer engine with:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised second-pass auditor (LLM-B)
//! - Transparent reliability scoring
//! - Automatic back-and-forth until score is high enough
//! - Clean refusal when it cannot answer safely

pub mod evidence;
pub mod protocol;
pub mod scoring;

pub use evidence::*;
pub use protocol::*;
pub use scoring::*;
