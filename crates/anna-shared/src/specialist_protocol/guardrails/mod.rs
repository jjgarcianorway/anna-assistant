//! Translator guardrails for intent validation (v0.0.428).
//!
//! Enforces:
//! - Correct intent classification (state vs how-to)
//! - Strict response validation
//! - No auto-invention of facts when specialist fails

pub mod context;
pub mod core;
pub mod intent;
pub mod response_type;
pub mod violations;

pub use context::*;
pub use core::*;
pub use intent::*;
pub use response_type::*;
pub use violations::*;
