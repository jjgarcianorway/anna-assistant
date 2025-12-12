//! Web server recipes (v0.0.460).
//!
//! Recipes for Nginx and Apache configuration.
//! Covers virtual hosts, SSL/TLS, reverse proxy, load balancing, and optimization.
//!
//! v0.0.460: Initial implementation per ROADMAP.md Future section.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{WebServerFeature, WebServerRecipe, WebServerType};
