//! Internal communications helper for Service Desk Theatre (v0.0.254).
//!
//! Generates IT department chatter for fly-on-wall experience.
//! Uses roster and dialogue systems to create authentic messages.
//!
//! v0.0.152: Added more variety, team-specific flavor, and probe result commentary.
//! v0.0.192: Modularized into domain-focused submodules.
//! v0.0.254: Added LLM-powered dialogue generation for natural variety.

pub mod dialogue_gen;
mod generator;
mod messages;
mod routing;
mod tests;

// Re-export main types and functions
pub use generator::CommsGenerator;
pub use routing::{team_from_domain, team_from_query_class};
