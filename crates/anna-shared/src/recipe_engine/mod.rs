//! Recipe Engine - Self-learning recipe system (v0.0.412).
//!
//! Core types for Anna's learning system:
//! - Recipe: Learned, replayable solution pattern
//! - RecipeStep: Individual action in a recipe
//! - EvidenceRequirement: What data a recipe needs
//! - RecipeStore: Persistent storage with matching
//!
//! Design goals:
//! - Minimize hardcoding, maximize learning
//! - All recipes are deterministic and auditable
//! - Generic recipes with parameters, not one per question

mod recipe;
mod step;
mod types;

// Re-export all public types to preserve the API
pub use recipe::Recipe;
pub use step::RecipeStep;
pub use types::{EvidenceRequirement, RecipeKind, RecipeParameter, RecipeStepType, RiskLevel};
