//! Recipe schema for learning engine (v0.0.427).
//!
//! Defines the core recipe structure with:
//! - Pattern matching (intent + keywords + required signals)
//! - Probe definitions
//! - Answer templates (short and detailed)
//! - Safety flags
//! - Origin tracking with citations
//! - Usage statistics

mod builder;
mod pattern;
mod template;
mod types;
mod utils;

// Re-export all public types
pub use pattern::RecipePattern;
pub use template::AnswerTemplate;
pub use types::{
    AnswerKind, ConditionalBranch, LearnedRecipe, LogicType, RecipeInputs, RecipeLogic,
    RecipeOrigin, RecipeProbe, RecipeSafety, RecipeUsageStats, RiskLevel,
};
