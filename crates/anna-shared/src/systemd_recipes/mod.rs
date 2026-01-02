//! Systemd unit file recipes (v0.0.233).
//!
//! Recipes for creating and managing systemd services, timers, and units.
//! Covers common tasks like creating services, debugging, and hardening.

mod matcher;
pub mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{RestartPolicy, SystemdFeature, SystemdRecipe, UnitType};
