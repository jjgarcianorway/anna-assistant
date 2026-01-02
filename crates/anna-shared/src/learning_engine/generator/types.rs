//! Generator types (v0.0.427).

use crate::learning_engine::LearnedRecipe;

/// Generated recipe from a ticket
#[derive(Debug, Clone)]
pub struct GeneratedRecipe {
    /// The recipe
    pub recipe: LearnedRecipe,
    /// Confidence in the generation
    pub confidence: f32,
    /// Warnings about the generated recipe
    pub warnings: Vec<String>,
}
