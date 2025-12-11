//! Recipe store with persistence and indexing (v0.0.423).
//!
//! Stores recipes in JSON files with indexes by:
//! - ID (unique lookup)
//! - Domain (category filtering)
//! - Tags (search)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{RecipeV3, RecipeDomain, RecipeOrigin};

/// Recipe store with file-based persistence
pub struct RecipeStore {
    /// Base directory for recipes
    base_dir: PathBuf,
    /// Global recipes directory
    global_dir: PathBuf,
    /// User recipes directory
    user_dir: PathBuf,
    /// In-memory cache of recipes
    recipes: HashMap<String, RecipeV3>,
    /// Index by domain
    by_domain: HashMap<RecipeDomain, Vec<String>>,
    /// Index by tag
    by_tag: HashMap<String, Vec<String>>,
    /// Whether store has been loaded
    loaded: bool,
}

impl Default for RecipeStore {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn load_dir(&mut self, dir: &Path) -> Result<usize, StoreError> {
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
    fn load_recipe_file(&self, path: &Path) -> Result<RecipeV3, StoreError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| StoreError::IoError(format!("Failed to read {}: {}", path.display(), e)))?;

        serde_json::from_str(&content)
            .map_err(|e| StoreError::ParseError(format!("Failed to parse {}: {}", path.display(), e)))
    }

    /// Index a recipe
    fn index_recipe(&mut self, recipe: &RecipeV3) {
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
    fn unindex_recipe(&mut self, recipe: &RecipeV3) {
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
        std::fs::write(&path, content)
            .map_err(|e| StoreError::IoError(format!("Failed to write {}: {}", path.display(), e)))?;

        // Update indexes - remove old recipe from indexes first
        let recipe_id = recipe.id.clone();
        if let Some(old) = self.recipes.remove(&recipe_id) {
            self.unindex_recipe(&old);
        }
        self.index_recipe(&recipe);
        self.recipes.insert(recipe_id, recipe);

        Ok(())
    }

    /// Get recipe by ID
    pub fn get(&self, id: &str) -> Option<&RecipeV3> {
        self.recipes.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut RecipeV3> {
        self.recipes.get_mut(id)
    }

    /// Get all recipes
    pub fn all(&self) -> Vec<&RecipeV3> {
        self.recipes.values().collect()
    }

    /// Get enabled recipes
    pub fn enabled(&self) -> Vec<&RecipeV3> {
        self.recipes.values().filter(|r| r.enabled).collect()
    }

    /// Get recipes by domain
    pub fn by_domain(&self, domain: RecipeDomain) -> Vec<&RecipeV3> {
        self.by_domain
            .get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipes by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&RecipeV3> {
        self.by_tag
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Search recipes by query
    pub fn search(&self, query: &str) -> Vec<&RecipeV3> {
        let q = query.to_lowercase();
        self.recipes
            .values()
            .filter(|r| {
                r.title.to_lowercase().contains(&q)
                    || r.description.to_lowercase().contains(&q)
                    || r.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    || r.matcher.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .collect()
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

    /// Get count of recipes
    pub fn count(&self) -> usize {
        self.recipes.len()
    }

    /// Check if store is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

/// Store errors
#[derive(Debug, Clone)]
pub enum StoreError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    NotFound(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_recipe(id: &str) -> RecipeV3 {
        RecipeV3::new(id, &format!("Test Recipe {}", id))
            .with_description("A test recipe")
            .with_tag("test")
    }

    #[test]
    fn test_store_init() {
        let tmp = TempDir::new().unwrap();
        let store = RecipeStore::with_base_dir(tmp.path());
        assert!(store.init().is_ok());
        assert!(tmp.path().join("global").exists());
        assert!(tmp.path().join("user").exists());
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        let recipe = test_recipe("test-1");
        store.save(recipe).unwrap();

        // Reload
        let mut store2 = RecipeStore::with_base_dir(tmp.path());
        let count = store2.load().unwrap();
        assert_eq!(count, 1);
        assert!(store2.get("test-1").is_some());
    }

    #[test]
    fn test_index_by_domain() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        let r1 = RecipeV3::new("r1", "Test 1")
            .with_matcher(super::super::RecipeMatcher::new(RecipeDomain::Systemd));
        let r2 = RecipeV3::new("r2", "Test 2")
            .with_matcher(super::super::RecipeMatcher::new(RecipeDomain::Package));

        store.save(r1).unwrap();
        store.save(r2).unwrap();

        let systemd = store.by_domain(RecipeDomain::Systemd);
        assert_eq!(systemd.len(), 1);
        assert_eq!(systemd[0].id, "r1");
    }

    #[test]
    fn test_search() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        store.save(test_recipe("nginx-restart")).unwrap();
        store.save(test_recipe("vim-config")).unwrap();

        let results = store.search("nginx");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_delete() {
        let tmp = TempDir::new().unwrap();
        let mut store = RecipeStore::with_base_dir(tmp.path());
        store.init().unwrap();

        store.save(test_recipe("to-delete")).unwrap();
        assert!(store.get("to-delete").is_some());

        store.delete("to-delete").unwrap();
        assert!(store.get("to-delete").is_none());
    }
}
