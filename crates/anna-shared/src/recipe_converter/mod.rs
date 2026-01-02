//! Recipe Converter - Ticket-to-Recipe conversion (v0.0.412).
//!
//! Converts successful ticket resolutions into reusable recipes.
//! Validates safety before creating recipes.

mod conversion;
mod types;
mod validation;

// Re-export public types
pub use types::{
    SpecialistRecipeCandidate, SpecialistStepCandidate, ValidationResult, MAX_STEPS,
    MIN_CONFIDENCE,
};

// Re-export public functions
pub use conversion::{
    auto_generate_recipe, convert_candidate, generate_recipe_id, parse_evidence_requirement,
    try_convert_ticket,
};
pub use validation::{is_eligible_for_recipe, is_safe_command, validate_candidate};

use crate::recipe_engine::Recipe;
use crate::recipe_store_v2::RecipeStoreV2;
use tracing::info;

/// Store a converted recipe
pub fn store_recipe(recipe: Recipe, store: &mut RecipeStoreV2) {
    info!("Storing new recipe: {} ({})", recipe.name, recipe.id);
    store.add(recipe);
    let _ = store.save();
}
