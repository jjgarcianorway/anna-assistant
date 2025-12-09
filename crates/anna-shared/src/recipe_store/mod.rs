//! Recipe store for learned solutions (v0.0.232).
//!
//! Stores reusable recipes learned from high-reliability ticket resolutions.
//!
//! v0.0.75: Initial implementation.
//! v0.0.232: Modularized into domain-focused submodules.

mod learning;
mod store;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use learning::{should_learn_recipe, MIN_LEARN_RELIABILITY};
pub use store::RecipeStore;
pub use types::{Citation, Recipe, RecipeRisk, RecipeStep};
