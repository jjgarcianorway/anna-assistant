//! Evidence fallback handling (v0.0.433).
//!
//! When LLM fails but probes succeeded, provide minimal fallback answers.

mod extractors;
mod generator;
mod types;

pub use generator::FallbackGenerator;
pub use types::{FallbackAnswer, ProbeEvidence};
