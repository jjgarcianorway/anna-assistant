//! Recipe Store v2 - Persistent storage for learned recipes (v0.0.412).
//!
//! Stores recipes in a JSON file with indexing for fast lookup.
//! Supports matching, promotion, deprecation, and cleanup.

mod types;
mod store;
mod matching;

// Re-export public types
pub use types::{
    RecipeStoreV2,
    StoreMetadata,
    RecipeMatch,
    MatchType,
    RecipeStoreStats,
};
