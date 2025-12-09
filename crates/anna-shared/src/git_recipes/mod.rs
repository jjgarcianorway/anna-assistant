//! Git configuration recipes for .gitconfig (v0.0.224).
//!
//! v0.0.100: Learned from specialists, reusable by translator.
//! v0.0.224: Modularized into domain-focused submodules.
//!
//! These recipes configure git via `git config` commands or direct
//! .gitconfig edits.

mod catalog;
mod recipe;
mod search;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use catalog::builtin_recipes;
pub use recipe::{GitParameter, GitRecipe};
pub use search::{detect_feature, find_recipe, find_recipes_by_keywords};
pub use types::{GitFeature, GitScope};
