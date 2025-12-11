//! Recipe file loader (v0.0.406).
//!
//! Loads TOML recipes from:
//! - /etc/anna/recipes/*.toml (system-wide)
//! - ~/.anna/recipes/authored/*.toml (user-authored)

use super::format::FileRecipe;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Get recipe directory paths
pub fn recipe_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];

    // System-wide recipes
    dirs.push(PathBuf::from("/etc/anna/recipes"));

    // User-authored recipes
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".anna").join("recipes").join("authored"));
    }

    dirs
}

/// Load a single recipe from a file
pub fn load_recipe_from_file(path: &PathBuf) -> Result<FileRecipe, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Load all recipes from all directories
/// Returns a map of full_id -> Recipe
pub fn load_all_recipes() -> HashMap<String, FileRecipe> {
    let mut recipes = HashMap::new();
    let mut loaded = 0;
    let mut errors = 0;

    for dir in recipe_dirs() {
        if !dir.exists() {
            debug!("Recipe dir does not exist: {}", dir.display());
            continue;
        }

        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "toml").unwrap_or(false) {
                        match load_recipe_from_file(&path) {
                            Ok(recipe) => {
                                let full_id = recipe.full_id();
                                debug!("Loaded recipe: {} from {}", full_id, path.display());
                                recipes.insert(full_id, recipe);
                                loaded += 1;
                            }
                            Err(e) => {
                                warn!("Failed to load recipe: {}", e);
                                errors += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Cannot read recipe dir {}: {}", dir.display(), e);
            }
        }
    }

    if loaded > 0 || errors > 0 {
        info!("Loaded {} recipes ({} errors)", loaded, errors);
    }

    recipes
}

/// Recipe registry with caching
#[derive(Debug, Default)]
pub struct RecipeRegistry {
    recipes: HashMap<String, FileRecipe>,
    loaded_at: Option<std::time::Instant>,
}

impl RecipeRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all recipes (with cache)
    pub fn load(&mut self) -> &HashMap<String, FileRecipe> {
        // Reload if stale (older than 60 seconds)
        let should_reload = self.loaded_at
            .map(|t| t.elapsed().as_secs() > 60)
            .unwrap_or(true);

        if should_reload {
            self.recipes = load_all_recipes();
            self.loaded_at = Some(std::time::Instant::now());
        }

        &self.recipes
    }

    /// Force reload
    pub fn reload(&mut self) {
        self.recipes = load_all_recipes();
        self.loaded_at = Some(std::time::Instant::now());
    }

    /// Get recipe by full ID
    pub fn get(&self, full_id: &str) -> Option<&FileRecipe> {
        self.recipes.get(full_id)
    }

    /// Get all recipes for a domain
    pub fn by_domain(&self, domain: &str) -> Vec<&FileRecipe> {
        self.recipes
            .values()
            .filter(|r| r.id.domain == domain)
            .collect()
    }

    /// Get recipe count
    pub fn count(&self) -> usize {
        self.recipes.len()
    }
}

/// Global recipe registry (lazy-loaded)
static REGISTRY: std::sync::OnceLock<std::sync::Mutex<RecipeRegistry>> = std::sync::OnceLock::new();

/// Get the global recipe registry
pub fn registry() -> std::sync::MutexGuard<'static, RecipeRegistry> {
    REGISTRY
        .get_or_init(|| std::sync::Mutex::new(RecipeRegistry::new()))
        .lock()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_dirs() {
        let dirs = recipe_dirs();
        assert!(!dirs.is_empty());
        // Should include /etc/anna/recipes
        assert!(dirs.iter().any(|d| d.to_str().unwrap().contains("/etc/anna")));
    }

    #[test]
    fn test_registry_cache() {
        let mut reg = RecipeRegistry::new();
        // First load
        let _ = reg.load();
        let loaded_at = reg.loaded_at;
        // Second load should use cache
        let _ = reg.load();
        assert_eq!(reg.loaded_at, loaded_at);
    }
}
