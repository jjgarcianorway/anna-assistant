//! Shell configuration recipes for .bashrc, .zshrc, etc. (v0.0.231).
//!
//! v0.0.100: Learned from specialists, reusable by translator.
//!
//! Key principle: Recipes are LEARNED from specialist responses and
//! can be applied to SIMILAR queries by the translator without LLM.
//!
//! v0.0.231: Modularized into domain-focused submodules.

mod catalog;
mod search;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use catalog::builtin_recipes;
pub use search::{detect_feature, find_recipe, find_recipes_by_keywords};
pub use types::{Shell, ShellFeature, ShellRecipe};
