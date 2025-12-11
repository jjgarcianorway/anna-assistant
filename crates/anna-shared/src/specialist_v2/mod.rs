//! Specialist V2 - Stable, Schema-Driven Responses (v0.0.421).
//!
//! This module provides a clean, reliable specialist system that eliminates
//! "Failed to parse specialist response" errors and guarantees tight,
//! schema-obedient answers.
//!
//! Design principles:
//! - Specialists are narrow, deterministic, schema-obedient
//! - Every response fits `SpecialistResponseV2` exactly
//! - Graceful fallbacks when LLM output is invalid
//! - Never expose parse errors to users

pub mod answer;
pub mod call;
pub mod fallback;
pub mod prompt;
pub mod renderer;
pub mod schema;
pub mod validate;

// Re-export main types
pub use answer::{AnswerType, DirectAnswer, KeyFinding, RecommendedAction};
pub use call::{SpecialistCall, SpecialistCallResult};
pub use fallback::{FallbackEngine, FallbackResult};
pub use prompt::{build_specialist_prompt, SpecialistPromptConfig};
pub use renderer::{render_response, RenderedAnswer};
pub use schema::{SpecialistResponseV2, SpecialistStatus};
pub use validate::{validate_response, ValidationResult};

/// Maximum time for specialist LLM call (ms)
pub const SPECIALIST_TIMEOUT_MS: u64 = 5000;

/// Maximum tokens for specialist response
pub const SPECIALIST_MAX_TOKENS: u32 = 1024;

/// Default confidence for fallback answers
pub const FALLBACK_CONFIDENCE: f32 = 0.6;

/// Minimum confidence to consider response useful
pub const MIN_USEFUL_CONFIDENCE: f32 = 0.4;
