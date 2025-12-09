//! User profile system for personalized Anna experience (v0.0.238).
//!
//! Tracks user preferences, patterns, and interaction history.
//!
//! v0.0.217: Modularized into domain-focused submodules.
//! v0.0.236: Added pattern history for trend detection.
//! v0.0.238: Added session history for "since last time" summaries.
//!
//! Storage: ~/.anna/profile.json (per-user)

mod greeting;
pub mod patterns;
mod profile;
pub mod session;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use greeting::GreetingContext;
pub use patterns::{EditorTrendInsight, PatternHistory, TopicTrendInsight};
pub use session::{SessionHistory, SessionSummary};
pub use types::{PersonalityTraits, UserPreferences, UserProfile};
