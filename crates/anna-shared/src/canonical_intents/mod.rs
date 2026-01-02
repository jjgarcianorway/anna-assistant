//! Canonical Intents and Topics (v0.0.416).
//!
//! The stable API surface for intent-based routing.
//! NO HARDCODED NATURAL LANGUAGE - only concepts.
//!
//! Intents: What the user wants to do (check_ram, diagnose_boot)
//! Topics: Knowledge domains to search (ram_usage, systemd_analyze)
//!
//! Router maps: Intent → Probes + Topics
//! Specialists receive: Probes output + Knowledge hits

mod types;
mod intent_impl;
mod mapping;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{CanonicalIntent, Topic};
pub use mapping::{intent_to_topics, translator_to_canonical};
