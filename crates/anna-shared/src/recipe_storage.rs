//! Recipe storage and indexing for Anna's learning system.
//! v0.0.418: File-based storage with in-memory index for fast lookup.
//!
//! Recipes are stored in:
//! - ~/.anna/recipes/{domain}/{id}.json (user-learned recipes)
//! - /var/lib/anna/recipes/{domain}/{id}.json (system/seed recipes)

use crate::recipe_schema::{Recipe, RecipeStatus};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Recipe storage with file persistence and in-memory index.
pub struct RecipeStorage {
    /// User recipes directory (~/.anna/recipes)
    user_dir: PathBuf,
    /// System recipes directory (/var/lib/anna/recipes)
    system_dir: PathBuf,
    /// In-memory index: domain -> intent -> recipe_id -> Recipe
    index: HashMap<String, HashMap<String, HashMap<String, Recipe>>>,
    /// Quick lookup by ID
    by_id: HashMap<String, Recipe>,
}

impl RecipeStorage {
    /// Create new storage with default directories
    pub fn new() -> Self {
        let user_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".anna")
            .join("recipes");
        let system_dir = PathBuf::from(crate::state_dir()).join("recipes");
        Self {
            user_dir,
            system_dir,
            index: HashMap::new(),
            by_id: HashMap::new(),
        }
    }

    /// Create storage with custom directories
    pub fn with_dirs(user_dir: PathBuf, system_dir: PathBuf) -> Self {
        Self {
            user_dir,
            system_dir,
            index: HashMap::new(),
            by_id: HashMap::new(),
        }
    }

    /// Load all recipes from disk
    pub fn load(&mut self) -> Result<()> {
        self.index.clear();
        self.by_id.clear();

        // Load system recipes first (can be overridden by user)
        if self.system_dir.exists() {
            self.load_from_dir(&self.system_dir.clone())?;
        }

        // Load user recipes (override system recipes with same ID)
        if self.user_dir.exists() {
            self.load_from_dir(&self.user_dir.clone())?;
        }

        Ok(())
    }

    fn load_from_dir(&mut self, base: &Path) -> Result<()> {
        if !base.exists() {
            return Ok(());
        }

        // Iterate domain directories
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.load_domain_dir(&path)?;
            }
        }
        Ok(())
    }

    fn load_domain_dir(&mut self, domain_dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(domain_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(recipe) = self.load_recipe_file(&path) {
                    self.index_recipe(recipe);
                }
            }
        }
        Ok(())
    }

    fn load_recipe_file(&self, path: &Path) -> Result<Recipe> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read recipe: {:?}", path))?;
        let recipe: Recipe = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse recipe: {:?}", path))?;
        Ok(recipe)
    }

    fn index_recipe(&mut self, recipe: Recipe) {
        let domain = recipe.domain.clone();
        let intent = recipe.intent.clone();
        let id = recipe.id.clone();

        self.by_id.insert(id.clone(), recipe.clone());

        self.index
            .entry(domain)
            .or_default()
            .entry(intent)
            .or_default()
            .insert(id, recipe);
    }

    /// Save a recipe to disk (user directory)
    pub fn save(&mut self, recipe: &Recipe) -> Result<()> {
        let domain_dir = self.user_dir.join(&recipe.domain);
        std::fs::create_dir_all(&domain_dir)?;

        let path = domain_dir.join(format!("{}.json", recipe.id));
        let content = serde_json::to_string_pretty(recipe)?;
        std::fs::write(&path, content)?;

        // Update index
        self.index_recipe(recipe.clone());
        Ok(())
    }

    /// Get recipe by ID
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.by_id.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Recipe> {
        self.by_id.get_mut(id)
    }

    /// Get all recipes for a domain
    pub fn get_by_domain(&self, domain: &str) -> Vec<&Recipe> {
        self.index
            .get(domain)
            .map(|intents| intents.values().flat_map(|r| r.values()).collect())
            .unwrap_or_default()
    }

    /// Get all recipes for a domain and intent
    pub fn get_by_intent(&self, domain: &str, intent: &str) -> Vec<&Recipe> {
        self.index
            .get(domain)
            .and_then(|intents| intents.get(intent))
            .map(|recipes| recipes.values().collect())
            .unwrap_or_default()
    }

    /// Get all active recipes
    pub fn get_active(&self) -> Vec<&Recipe> {
        self.by_id
            .values()
            .filter(|r| r.status == RecipeStatus::Active)
            .collect()
    }

    /// Get all recipes
    pub fn all(&self) -> Vec<&Recipe> {
        self.by_id.values().collect()
    }

    /// Count recipes by status
    pub fn count_by_status(&self) -> RecipeStatusCounts {
        let mut counts = RecipeStatusCounts::default();
        for recipe in self.by_id.values() {
            match recipe.status {
                RecipeStatus::Active => counts.active += 1,
                RecipeStatus::NeedsReview => counts.needs_review += 1,
                RecipeStatus::Disabled => counts.disabled += 1,
                RecipeStatus::Deprecated => counts.deprecated += 1,
            }
        }
        counts
    }

    /// Update recipe metrics and save
    pub fn update_metrics(&mut self, id: &str, success: bool) -> Result<()> {
        if let Some(recipe) = self.by_id.get_mut(id) {
            if success {
                recipe.record_success();
            } else {
                recipe.record_failure();
            }
            // Save updated recipe
            let recipe_clone = recipe.clone();
            self.save(&recipe_clone)?;
        }
        Ok(())
    }

    /// Check if recipe exists
    pub fn exists(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    /// Delete a recipe
    pub fn delete(&mut self, id: &str) -> Result<bool> {
        if let Some(recipe) = self.by_id.remove(id) {
            // Remove from index
            if let Some(intents) = self.index.get_mut(&recipe.domain) {
                if let Some(recipes) = intents.get_mut(&recipe.intent) {
                    recipes.remove(id);
                }
            }
            // Remove file
            let path = self
                .user_dir
                .join(&recipe.domain)
                .join(format!("{}.json", id));
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get total recipe count
    pub fn count(&self) -> usize {
        self.by_id.len()
    }

    /// Get domains with recipes
    pub fn domains(&self) -> Vec<&str> {
        self.index.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for RecipeStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Recipe counts by status.
#[derive(Debug, Clone, Default)]
pub struct RecipeStatusCounts {
    pub active: usize,
    pub needs_review: usize,
    pub disabled: usize,
    pub deprecated: usize,
}

impl RecipeStatusCounts {
    pub fn total(&self) -> usize {
        self.active + self.needs_review + self.disabled + self.deprecated
    }
}

/// Recipe storage statistics.
#[derive(Debug, Clone)]
pub struct RecipeStorageStats {
    pub total_recipes: usize,
    pub active_recipes: usize,
    pub disabled_recipes: usize,
    pub total_uses: u32,
    pub total_failures: u32,
    pub domains: Vec<String>,
}

impl RecipeStorage {
    /// Get storage statistics
    pub fn stats(&self) -> RecipeStorageStats {
        let counts = self.count_by_status();
        let (total_uses, total_failures) = self.by_id.values().fold((0u32, 0u32), |acc, r| {
            (acc.0 + r.metrics.times_used, acc.1 + r.metrics.times_failed)
        });

        RecipeStorageStats {
            total_recipes: counts.total(),
            active_recipes: counts.active,
            disabled_recipes: counts.disabled,
            total_uses,
            total_failures,
            domains: self.domains().into_iter().map(String::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_schema::{RecipeMatcher, RecipePattern};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn make_test_recipe(id: &str, domain: &str, intent: &str) -> Recipe {
        Recipe::new(
            id.into(),
            domain.into(),
            intent.into(),
            RecipePattern {
                user_goal: "test".into(),
                slots: HashMap::new(),
            },
            RecipeMatcher {
                required_keywords: vec![],
                optional_keywords: vec![],
                negative_keywords: vec![],
                min_confidence: 0.8,
                exact_intent: None,
            },
            vec![],
        )
    }

    #[test]
    fn test_storage_save_load() {
        let dir = tempdir().unwrap();
        let user_dir = dir.path().join("user");
        let sys_dir = dir.path().join("sys");

        let mut storage = RecipeStorage::with_dirs(user_dir.clone(), sys_dir);
        let recipe = make_test_recipe("test1", "desktop", "configure_editor");

        storage.save(&recipe).unwrap();
        assert!(storage.exists("test1"));

        // Reload
        let mut storage2 = RecipeStorage::with_dirs(user_dir, dir.path().join("sys2"));
        storage2.load().unwrap();
        assert!(storage2.exists("test1"));
    }

    #[test]
    fn test_get_by_domain() {
        let dir = tempdir().unwrap();
        let mut storage = RecipeStorage::with_dirs(dir.path().join("user"), dir.path().join("sys"));

        storage
            .save(&make_test_recipe("r1", "desktop", "intent1"))
            .unwrap();
        storage
            .save(&make_test_recipe("r2", "desktop", "intent2"))
            .unwrap();
        storage
            .save(&make_test_recipe("r3", "network", "intent1"))
            .unwrap();

        let desktop = storage.get_by_domain("desktop");
        assert_eq!(desktop.len(), 2);

        let network = storage.get_by_domain("network");
        assert_eq!(network.len(), 1);
    }
}
