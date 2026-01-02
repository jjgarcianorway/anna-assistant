//! Fallback Summarizer (Part D) - v0.0.440.
//!
//! If specialist fails after retries:
//! - Translator model produces a minimal answer from evidence only
//! - No speculation
//! - Prevents garbage answers (e.g., "CPU model" for "top CPU service")

mod extractors;
mod summarizer;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use summarizer::{AnswerTemplate, FallbackSummarizer};
pub use types::{FallbackContext, FallbackReason, FallbackResponse, ProbeEvidence};
