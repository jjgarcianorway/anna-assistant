//! Recipe store implementation - loading, saving, and indexing (v0.0.423).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{RecipeOrigin, RecipeV3};
use super::store_types::{RecipeStore, StoreError};

impl RecipeStore {
    /// Create a new store with default paths
    pub fn new() -> Self {
        let base = PathBuf::from("/var/lib/anna/recipes_v3");
        Self::with_base_dir(&base)
    }

    /// Create store with custom base directory
    pub fn with_base_dir(base: &Path) -> Self {
        Self {
            base_dir: base.to_path_buf(),
            global_dir: base.join("global"),
            user_dir: base.join("user"),
            recipes: HashMap::new(),
            by_domain: HashMap::new(),
            by_tag: HashMap::new(),
            loaded: false,
        }
    }

    /// Initialize directories
    pub fn init(&self) -> Result<(), StoreError> {
        std::fs::create_dir_all(&self.global_dir)
            .map_err(|e| StoreError::IoError(format!("Failed to create global dir: {}", e)))?;
        std::fs::create_dir_all(&self.user_dir)
            .map_err(|e| StoreError::IoError(format!("Failed to create user dir: {}", e)))?;
        Ok(())
    }

    /// Load all recipes from disk
    pub fn load(&mut self) -> Result<usize, StoreError> {
        self.recipes.clear();
        self.by_domain.clear();
        self.by_tag.clear();

        let mut count = 0;

        // Load global recipes
        count += self.load_dir(&self.global_dir.clone())?;

        // Load user recipes
        count += self.load_dir(&self.user_dir.clone())?;

        self.loaded = true;
        Ok(count)
    }

    /// Load recipes from a directory
    pub(super) fn load_dir(&mut self, dir: &Path) -> Result<usize, StoreError> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut count = 0;

        let entries = std::fs::read_dir(dir)
            .map_err(|e| StoreError::IoError(format!("Failed to read dir: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(recipe) = self.load_recipe_file(&path) {
                    self.index_recipe(&recipe);
                    self.recipes.insert(recipe.id.clone(), recipe);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Load a single recipe file
    pub(super) fn load_recipe_file(&self, path: &Path) -> Result<RecipeV3, StoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            StoreError::IoError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            StoreError::ParseError(format!("Failed to parse {}: {}", path.display(), e))
        })
    }

    /// Index a recipe
    pub(super) fn index_recipe(&mut self, recipe: &RecipeV3) {
        // Index by domain
        self.by_domain
            .entry(recipe.matcher.domain)
            .or_default()
            .push(recipe.id.clone());

        // Index by tags
        for tag in &recipe.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .push(recipe.id.clone());
        }
    }

    /// Remove recipe from indexes
    pub(super) fn unindex_recipe(&mut self, recipe: &RecipeV3) {
        // Remove from domain index
        if let Some(ids) = self.by_domain.get_mut(&recipe.matcher.domain) {
            ids.retain(|id| id != &recipe.id);
        }

        // Remove from tag indexes
        for tag in &recipe.tags {
            if let Some(ids) = self.by_tag.get_mut(tag) {
                ids.retain(|id| id != &recipe.id);
            }
        }
    }

    /// Save a recipe
    pub fn save(&mut self, recipe: RecipeV3) -> Result<(), StoreError> {
        let _ = self.init();

        // Determine directory based on origin
        let dir = match recipe.origin {
            RecipeOrigin::BuiltIn => &self.global_dir,
            RecipeOrigin::LearnedFromTicket | RecipeOrigin::UserAuthored => &self.user_dir,
        };

        let path = dir.join(format!("{}.json", recipe.id));

        // Serialize
        let content = serde_json::to_string_pretty(&recipe)
            .map_err(|e| StoreError::SerializeError(format!("Failed to serialize: {}", e)))?;

        // Write file
        std::fs::write(&path, content).map_err(|e| {
            StoreError::IoError(format!("Failed to write {}: {}", path.display(), e))
        })?;

        // Update indexes - remove old recipe from indexes first
        let recipe_id = recipe.id.clone();
        if let Some(old) = self.recipes.remove(&recipe_id) {
            self.unindex_recipe(&old);
        }
        self.index_recipe(&recipe);
        self.recipes.insert(recipe_id, recipe);

        Ok(())
    }

    /// Delete a recipe
    pub fn delete(&mut self, id: &str) -> Result<bool, StoreError> {
        if let Some(recipe) = self.recipes.remove(id) {
            self.unindex_recipe(&recipe);

            // Remove file
            let dir = match recipe.origin {
                RecipeOrigin::BuiltIn => &self.global_dir,
                _ => &self.user_dir,
            };
            let path = dir.join(format!("{}.json", id));
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| StoreError::IoError(format!("Failed to delete: {}", e)))?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update recipe stats after execution
    pub fn record_execution(
        &mut self,
        id: &str,
        success: bool,
        duration_ms: u64,
    ) -> Result<(), StoreError> {
        if let Some(recipe) = self.recipes.get_mut(id) {
            if success {
                recipe.stats.record_success(duration_ms);
            } else {
                recipe.stats.record_failure();
            }

            // Persist updated stats
            let recipe_clone = recipe.clone();
            self.save(recipe_clone)?;
        }
        Ok(())
    }

    /// Record that recipe was matched but skipped
    pub fn record_skip(&mut self, id: &str) -> Result<(), StoreError> {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.stats.record_skip();
            let recipe_clone = recipe.clone();
            self.save(recipe_clone)?;
        }
        Ok(())
    }
}
