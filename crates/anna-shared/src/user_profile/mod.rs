//! User profile system for personalized Anna experience (v0.0.236).
//!
//! Tracks user preferences, patterns, and interaction history.
//!
//! v0.0.217: Modularized into domain-focused submodules.
//! v0.0.236: Added pattern history for trend detection.
//!
//! Storage: ~/.anna/profile.json (per-user)

mod greeting;
pub mod patterns;
mod profile;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use greeting::GreetingContext;
pub use patterns::{EditorTrendInsight, PatternHistory, TopicTrendInsight};
pub use types::{PersonalityTraits, UserPreferences, UserProfile};
