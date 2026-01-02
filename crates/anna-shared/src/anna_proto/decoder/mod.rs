//! Two-Stage Decoder (Part B) - v0.0.436.
//!
//! Decoder pipeline that never blocks:
//! 1. Fast path: Extract framed JSON
//! 2. Recovery path: Tolerant JSON scanning and repair
//!
//! Parsing itself NEVER times out - only model calls can timeout.

mod core;
mod helpers;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use core::ProtoDecoder;
pub use types::{DecodeError, DecodeResult};
