//! SSH key and configuration recipes (v0.0.196).
//!
//! Recipes for SSH key generation, copying, and config management.
//! These are common tasks that Anna can help with using deterministic recipes.
//!
//! v0.0.196: Modularized into domain-focused submodules.

mod matcher;
mod paths;
mod recipes;
mod tests;
mod types;

// Re-export all types and functions
pub use matcher::match_query;
pub use paths::{ssh_config_path, ssh_dir};
pub use recipes::builtin_recipes;
pub use types::{SshFeature, SshKeyType, SshRecipe, SshStep};
