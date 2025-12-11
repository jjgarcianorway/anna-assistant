//! Recipe V2 - Learning Engine (v0.0.420).
//!
//! A clean, transparent recipe system for self-improving Anna:
//! - Learn from successful tickets
//! - Store reusable patterns as recipes
//! - Match future queries to recipes before calling specialists
//! - Prefer general rules over one-off hacks
//!
//! Design principles:
//! - All learning is grounded in real probe data and successful outcomes
//! - Recipes are small, composable, and inspectable
//! - Global recipes (shipped with Anna) vs user learned recipes
//! - Conservative learning: only from high-confidence verified tickets

pub mod dispatcher;
pub mod domain;
pub mod fact;
pub mod learner;
pub mod matcher;
pub mod seed;
pub mod stats;
pub mod step;
pub mod storage;
pub mod trigger;
pub mod types;

// Re-export main types
pub use dispatcher::{RecipeDispatcher, RecipeQuery, RecipeResult};
pub use domain::RecipeDomain;
pub use fact::{FactOp, FactRequirement};
pub use learner::{RecipeLearner, TicketObservation};
pub use matcher::{find_best_recipe, has_high_confidence_match, MatchResult, RecipeMatcherV2};
pub use seed::{get_seed_recipes, SEED_RECIPES};
pub use stats::RecipeStats;
pub use step::{RecipeStepAction, RecipeStepKind, RecipeStepV2};
pub use storage::{load_all_recipes, load_global_recipes, load_user_recipes, RecipeStorageV2};
pub use trigger::TriggerPattern;
pub use types::RecipeV2;

/// Minimum confidence for auto-apply (without specialist)
pub const AUTO_APPLY_THRESHOLD: f32 = 0.75;

/// Minimum confidence for learning from a ticket
pub const LEARNING_THRESHOLD: f32 = 0.90;

/// Minimum success rate for recipe to be considered reliable
pub const MIN_SUCCESS_RATE: f32 = 0.80;
