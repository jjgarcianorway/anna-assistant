//! Recipe store with persistence (v0.0.232).

use super::types::Recipe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Recipe store with persistence
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStore {
    /// Version for migration
    pub version: u32,
    /// Recipes by ID
    pub recipes: HashMap<String, Recipe>,
    /// Index: query_class -> recipe IDs
    #[serde(default)]
    pub trigger_index: HashMap<String, Vec<String>>,
}

impl RecipeStore {
    pub fn new() -> Self {
        Self {
            version: 1,
            recipes: HashMap::new(),
            trigger_index: HashMap::new(),
        }
    }

    /// Default path
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/recipes_v2.json")
    }

    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let store: Self = serde_json::from_reader(reader)?;
        Ok(store)
    }

    /// Save to file
    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();

        // Ensure parent exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write via temp file
        let temp_path = path.with_extension("json.tmp");
        {
            let file = File::create(&temp_path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, self)?;
        }
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Add a recipe
    pub fn add(&mut self, recipe: Recipe) {
        // Update trigger index
        for trigger in &recipe.triggers {
            self.trigger_index
                .entry(trigger.clone())
                .or_default()
                .push(recipe.id.clone());
        }

        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Find matching recipes for a query
    pub fn find_matches(&self, query_class: &str, evidence: &[String]) -> Vec<&Recipe> {
        let recipe_ids = self.trigger_index.get(query_class);

        match recipe_ids {
            Some(ids) => ids
                .iter()
                .filter_map(|id| self.recipes.get(id))
                .filter(|r| r.matches(query_class, evidence))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get recipe by ID
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Recipe> {
        self.recipes.get_mut(id)
    }

    /// Count recipes by category
    pub fn count_by_category(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for recipe in self.recipes.values() {
            *counts.entry(recipe.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Total recipe count
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}
