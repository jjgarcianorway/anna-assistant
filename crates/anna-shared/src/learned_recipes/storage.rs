//! Recipe storage and persistence.

use super::types::{LearnedRecipe, RecipeStoreSummary};
use crate::canonical_intents::CanonicalIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Recipe store - persistent storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStore {
    /// Recipes by ID
    pub recipes: HashMap<String, LearnedRecipe>,
    /// Index by intent
    pub by_intent: HashMap<String, Vec<String>>,
    /// Last save time
    pub last_saved: u64,
}

impl RecipeStore {
    /// Load from disk
    pub fn load() -> Self {
        let path = store_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&mut self) -> Result<(), String> {
        self.last_saved = current_secs();
        let path = store_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Add or update recipe
    pub fn upsert(&mut self, recipe: LearnedRecipe) {
        let intent_key = format!("{:?}", recipe.intent);

        // Update index
        let ids = self.by_intent.entry(intent_key).or_default();
        if !ids.contains(&recipe.id) {
            ids.push(recipe.id.clone());
        }

        // Store recipe
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Find recipe for intent
    pub fn find_for_intent(&self, intent: CanonicalIntent) -> Option<&LearnedRecipe> {
        let intent_key = format!("{:?}", intent);
        let ids = self.by_intent.get(&intent_key)?;

        // Find best active recipe
        ids.iter()
            .filter_map(|id| self.recipes.get(id))
            .filter(|r| !r.deprecated && r.stats.success_rate() >= 0.5)
            .max_by(|a, b| {
                a.stats
                    .success_rate()
                    .partial_cmp(&b.stats.success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get mutable recipe
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LearnedRecipe> {
        self.recipes.get_mut(id)
    }

    /// List all active recipes
    pub fn active_recipes(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| !r.deprecated).collect()
    }

    /// Get stats summary
    pub fn stats_summary(&self) -> RecipeStoreSummary {
        let active = self.recipes.values().filter(|r| !r.deprecated).count();
        let deprecated = self.recipes.values().filter(|r| r.deprecated).count();
        let total_uses: u32 = self.recipes.values().map(|r| r.stats.uses).sum();
        let total_successes: u32 = self.recipes.values().map(|r| r.stats.successes).sum();

        RecipeStoreSummary {
            total: self.recipes.len(),
            active,
            deprecated,
            total_uses,
            success_rate: if total_uses > 0 {
                total_successes as f32 / total_uses as f32
            } else {
                1.0
            },
        }
    }
}

fn store_path() -> PathBuf {
    let base = std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
    PathBuf::from(base).join("learned_recipes.json")
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
