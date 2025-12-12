//! Network troubleshooting recipes (v0.0.462).
//!
//! Recipes for network diagnostics, connectivity testing, and troubleshooting.
//! Covers ping, DNS, traceroute, port scanning, firewall, WiFi, and VPN.
//!
//! v0.0.462: Initial implementation per ROADMAP.md Future section.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{NetworkFeature, NetworkRecipe};
