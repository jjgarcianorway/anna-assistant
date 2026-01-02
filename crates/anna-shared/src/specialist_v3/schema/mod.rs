//! Strict specialist response schema (v0.0.425).
//!
//! This is THE SINGLE CANONICAL SCHEMA for all specialist responses.
//! No freeform prose - JSON only.

mod impl_response;
mod tests;
mod types;

// Re-export all public types
pub use types::*;
