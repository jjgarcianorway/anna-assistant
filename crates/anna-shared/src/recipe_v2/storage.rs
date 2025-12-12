//! Recipe storage with global vs user recipes (v0.0.420).
//!
//! Storage locations:
//! - Global recipes: /etc/anna/recipes/{domain}/*.json
//! - User recipes: ~/.anna/recipes_v2/{domain}/*.json
//!
//! Precedence: User recipes override global recipes by id.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{RecipeDomain, RecipeV2};

/// Global recipes directory
pub fn global_recipe_dir() -> PathBuf {
    PathBuf::from("/etc/anna/recipes")
}

/// User recipes directory
pub fn user_recipe_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("recipes_v2")
}

/// Recipe storage manager
#[derive(Debug, Default)]
pub struct RecipeStorageV2 {
    /// All loaded recipes by id
    recipes: HashMap<String, RecipeV2>,
    /// Whether learning is enabled
    learning_enabled: bool,
}

impl RecipeStorageV2 {
    /// Create new storage
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
            learning_enabled: true,
        }
    }

    /// Load all recipes from both global and user directories
    pub fn load_all(&mut self) -> Result<(), String> {
        // Load global recipes first
        let global = load_global_recipes()?;
        for recipe in global {
            self.recipes.insert(recipe.id.clone(), recipe);
        }

        // Load user recipes (override global)
        let user = load_user_recipes()?;
        for recipe in user {
            self.recipes.insert(recipe.id.clone(), recipe);
        }

        Ok(())
    }

    /// Get a recipe by id
    pub fn get(&self, id: &str) -> Option<&RecipeV2> {
        self.recipes.get(id)
    }

    /// Get a mutable recipe by id
    pub fn get_mut(&mut self, id: &str) -> Option<&mut RecipeV2> {
        self.recipes.get_mut(id)
    }

    /// Get all recipes for a domain
    pub fn get_by_domain(&self, domain: RecipeDomain) -> Vec<&RecipeV2> {
        self.recipes
            .values()
            .filter(|r| r.domain == domain && r.is_available())
            .collect()
    }

    /// Get all available recipes
    pub fn get_available(&self) -> Vec<&RecipeV2> {
        self.recipes.values().filter(|r| r.is_available()).collect()
    }

    /// Add or update a recipe (saves to user directory)
    pub fn upsert(&mut self, mut recipe: RecipeV2) -> Result<(), String> {
        if !self.learning_enabled && !recipe.is_global {
            return Err("Learning is disabled".to_string());
        }

        // Mark as not global (user recipe)
        recipe.is_global = false;

        // Save to disk
        save_user_recipe(&recipe)?;

        // Update in-memory
        self.recipes.insert(recipe.id.clone(), recipe);

        Ok(())
    }

    /// Remove a recipe (only user recipes can be removed)
    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        if let Some(recipe) = self.recipes.get(id) {
            if recipe.is_global {
                return Err("Cannot remove global recipes".to_string());
            }

            let path = recipe.file_path(&user_recipe_dir());
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
            }
        }

        self.recipes.remove(id);
        Ok(())
    }

    /// Enable/disable learning
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
    }

    /// Check if learning is enabled
    pub fn is_learning_enabled(&self) -> bool {
        self.learning_enabled
    }

    /// Get total recipe count
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Get count of global recipes
    pub fn global_count(&self) -> usize {
        self.recipes.values().filter(|r| r.is_global).count()
    }

    /// Get count of user recipes
    pub fn user_count(&self) -> usize {
        self.recipes.values().filter(|r| !r.is_global).count()
    }

    /// Clear all user recipes
    pub fn clear_user_recipes(&mut self) -> Result<(), String> {
        let dir = user_recipe_dir();
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed to clear recipes: {}", e))?;
        }

        // Remove user recipes from memory
        self.recipes.retain(|_, r| r.is_global);
        Ok(())
    }
}

/// Load all global recipes
pub fn load_global_recipes() -> Result<Vec<RecipeV2>, String> {
    load_recipes_from_dir(&global_recipe_dir(), true)
}

/// Load all user recipes
pub fn load_user_recipes() -> Result<Vec<RecipeV2>, String> {
    load_recipes_from_dir(&user_recipe_dir(), false)
}

/// Load all recipes (global + user with precedence)
pub fn load_all_recipes() -> Result<HashMap<String, RecipeV2>, String> {
    let mut recipes = HashMap::new();

    // Load global first
    for recipe in load_global_recipes()? {
        recipes.insert(recipe.id.clone(), recipe);
    }

    // User recipes override
    for recipe in load_user_recipes()? {
        recipes.insert(recipe.id.clone(), recipe);
    }

    Ok(recipes)
}

/// Load recipes from a directory
fn load_recipes_from_dir(base_dir: &Path, is_global: bool) -> Result<Vec<RecipeV2>, String> {
    let mut recipes = Vec::new();

    if !base_dir.exists() {
        return Ok(recipes);
    }

    // Walk domain subdirectories
    for domain in RecipeDomain::all() {
        let domain_dir = base_dir.join(domain.subdir());
        if !domain_dir.exists() {
            continue;
        }

        let entries = std::fs::read_dir(&domain_dir)
            .map_err(|e| format!("Failed to read {}: {}", domain_dir.display(), e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match load_recipe_file(&path, is_global) {
                    Ok(recipe) => recipes.push(recipe),
                    Err(e) => {
                        tracing::warn!("Failed to load recipe {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(recipes)
}

/// Load a single recipe file
fn load_recipe_file(path: &Path, is_global: bool) -> Result<RecipeV2, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;

    let mut recipe: RecipeV2 =
        serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;

    recipe.is_global = is_global;
    Ok(recipe)
}

/// Save a recipe to user directory
pub fn save_user_recipe(recipe: &RecipeV2) -> Result<(), String> {
    let base_dir = user_recipe_dir();
    let domain_dir = base_dir.join(recipe.domain.subdir());

    // Create directory if needed
    std::fs::create_dir_all(&domain_dir)
        .map_err(|e| format!("Failed to create {}: {}", domain_dir.display(), e))?;

    let path = domain_dir.join(format!("{}.json", recipe.id));
    let content =
        serde_json::to_string_pretty(recipe).map_err(|e| format!("Serialize error: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_paths() {
        let global = global_recipe_dir();
        assert!(global.to_string_lossy().contains("/etc/anna"));

        let user = user_recipe_dir();
        assert!(user.to_string_lossy().contains(".anna"));
    }
}
