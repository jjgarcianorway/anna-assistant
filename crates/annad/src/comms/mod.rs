//! Internal communications helper for Service Desk Theatre (v0.0.192).
//!
//! Generates IT department chatter for fly-on-wall experience.
//! Uses roster and dialogue systems to create authentic messages.
//!
//! v0.0.152: Added more variety, team-specific flavor, and probe result commentary.
//! v0.0.192: Modularized into domain-focused submodules.

mod generator;
mod messages;
mod routing;
mod tests;

// Re-export main types and functions
pub use generator::CommsGenerator;
pub use routing::team_from_domain;
