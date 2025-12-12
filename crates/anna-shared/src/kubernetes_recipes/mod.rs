//! Kubernetes recipes (v0.0.459).
//!
//! Recipes for managing Kubernetes clusters, pods, deployments, and services.
//! Covers common kubectl operations, debugging, and resource management.
//!
//! v0.0.459: Initial implementation per ROADMAP.md Future section.

mod matcher;
mod recipes;
#[cfg(test)]
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::{detect_feature, match_query};
pub use recipes::builtin_recipes;
pub use types::{K8sFeature, K8sRecipe};
