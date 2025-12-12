//! Team-tagged recipes for learning from successful patterns (v0.0.177).
//!
//! Recipes capture successful ticket resolutions for future reference.
//! Only persisted when: Ticket status = Verified, reliability score >= 80.
//!
//! Storage: ~/.anna/recipes/{recipe_id}.json
//!
//! v0.0.27: Extended with RecipeKind for config edits and change actions.
//! v0.0.177: Modularized into domain-focused submodules.

mod core;
mod search;
mod signature;
mod slot;
mod storage;
mod target;
mod types;

// Re-export all types
pub use core::{compute_recipe_id, Recipe};
pub use search::{find_config_edit_recipes, search_recipes_by_keywords, RecipeMatch};
pub use signature::RecipeSignature;
pub use slot::{ClarifyPrereq, RecipeSlot};
pub use storage::{
    clear_all_recipes, recipe_count, recipe_dir, recipe_filename, should_persist_recipe,
    RECIPE_PERSIST_THRESHOLD,
};
pub use target::{RecipeTarget, RollbackInfo};
pub use types::{RecipeAction, RecipeKind};
