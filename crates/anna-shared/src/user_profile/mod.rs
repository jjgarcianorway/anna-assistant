//! User profile system for personalized Anna experience (v0.0.217).
//!
//! Tracks user preferences, patterns, and interaction history.
//!
//! v0.0.217: Modularized into domain-focused submodules.
//!
//! Storage: ~/.anna/profile.json (per-user)

mod greeting;
mod profile;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use greeting::GreetingContext;
pub use types::{PersonalityTraits, UserPreferences, UserProfile};
