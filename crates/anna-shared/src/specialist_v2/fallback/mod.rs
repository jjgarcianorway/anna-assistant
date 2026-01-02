//! Fallback engine for specialist failures (v0.0.421).
//!
//! Provides deterministic fallback answers when:
//! - JSON parsing fails
//! - LLM response is invalid
//! - Timeout occurs
//!
//! Covers common question types:
//! - Memory usage
//! - Failed services
//! - Disk usage
//! - Network interfaces
//! - Swap status

mod engine;
mod handlers;
mod parsers;

#[cfg(test)]
mod tests;

pub use engine::{FallbackEngine, FallbackResult};
