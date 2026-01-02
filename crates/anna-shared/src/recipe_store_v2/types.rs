//! Types for Recipe Store v2

use crate::recipe_engine::Recipe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recipe store with indexing
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecipeStoreV2 {
    /// All recipes by ID
    pub recipes: HashMap<String, Recipe>,
    /// Index: tag -> recipe IDs
    #[serde(default)]
    pub tag_index: HashMap<String, Vec<String>>,
    /// Index: domain -> recipe IDs
    #[serde(default)]
    pub domain_index: HashMap<String, Vec<String>>,
    /// Index: kind -> recipe IDs
    #[serde(default)]
    pub kind_index: HashMap<String, Vec<String>>,
    /// Store metadata
    #[serde(default)]
    pub metadata: StoreMetadata,
}

/// Store metadata
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StoreMetadata {
    pub version: u32,
    pub last_gc_at: u64,
    pub total_recipes_created: u32,
    pub total_recipes_deprecated: u32,
}

/// Match result for a recipe
#[derive(Debug, Clone)]
pub struct RecipeMatch {
    pub recipe_id: String,
    pub score: f32,
    pub match_type: MatchType,
}

/// How the recipe matched
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
    /// Exact trigger pattern match
    ExactTrigger,
    /// High tag overlap
    TagMatch,
    /// Domain + intent match
    DomainIntent,
    /// Partial match
    Partial,
}

/// Recipe store statistics
#[derive(Debug, Clone)]
pub struct RecipeStoreStats {
    pub total_recipes: usize,
    pub active_recipes: usize,
    pub deprecated_recipes: usize,
    pub total_uses: u32,
    pub total_successes: u32,
    pub overall_success_rate: f32,
}

impl std::fmt::Display for RecipeStoreStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[recipe store]")?;
        writeln!(f, "  total_recipes     {}", self.total_recipes)?;
        writeln!(f, "  active            {}", self.active_recipes)?;
        writeln!(f, "  deprecated        {}", self.deprecated_recipes)?;
        writeln!(f, "  total_uses        {}", self.total_uses)?;
        writeln!(
            f,
            "  success_rate      {:.1}%",
            self.overall_success_rate * 100.0
        )?;
        Ok(())
    }
}
