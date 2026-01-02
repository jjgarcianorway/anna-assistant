//! Core RecipeStoreV2 implementation - persistence, CRUD, and indexing

use crate::recipe_engine::Recipe;
use super::types::{RecipeStoreV2, RecipeStoreStats};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

impl RecipeStoreV2 {
    /// Load from disk
    pub fn load() -> Self {
        let path = Self::store_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(store) => {
                        debug!("Loaded {} recipes from store", store_count(&store));
                        return store;
                    }
                    Err(e) => warn!("Failed to parse recipe store: {}", e),
                },
                Err(e) => warn!("Failed to read recipe store: {}", e),
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        debug!("Saved {} recipes to store", self.recipes.len());
        Ok(())
    }

    /// Get store path
    fn store_path() -> PathBuf {
        let base = std::env::var("ANNA_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".anna")
            });
        base.join("recipes_v2.json")
    }

    /// Add a recipe to the store
    pub fn add(&mut self, recipe: Recipe) {
        let id = recipe.id.clone();

        // Update indexes
        for tag in &recipe.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(id.clone());
        }
        self.domain_index
            .entry(recipe.domain.clone())
            .or_default()
            .push(id.clone());
        self.kind_index
            .entry(recipe.kind.to_string())
            .or_default()
            .push(id.clone());

        self.recipes.insert(id.clone(), recipe);
        self.metadata.total_recipes_created += 1;
        info!("Added recipe: {}", id);
    }

    /// Get a recipe by ID
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Recipe> {
        self.recipes.get_mut(id)
    }

    /// Deprecate a recipe
    pub fn deprecate(&mut self, id: &str) {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.deprecated = true;
            self.metadata.total_recipes_deprecated += 1;
            info!("Deprecated recipe: {}", id);
        }
    }

    /// Run garbage collection
    pub fn gc(&mut self) {
        let now = current_secs();
        let old_threshold = now.saturating_sub(30 * 24 * 3600); // 30 days

        let to_remove: Vec<_> = self
            .recipes
            .iter()
            .filter(|(_, r)| r.deprecated && r.last_used_at < old_threshold && r.use_count < 5)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &to_remove {
            self.recipes.remove(id);
            info!("GC removed recipe: {}", id);
        }

        // Rebuild indexes
        self.rebuild_indexes();
        self.metadata.last_gc_at = now;
    }

    /// Rebuild all indexes
    fn rebuild_indexes(&mut self) {
        self.tag_index.clear();
        self.domain_index.clear();
        self.kind_index.clear();

        for (id, recipe) in &self.recipes {
            for tag in &recipe.tags {
                self.tag_index
                    .entry(tag.clone())
                    .or_default()
                    .push(id.clone());
            }
            self.domain_index
                .entry(recipe.domain.clone())
                .or_default()
                .push(id.clone());
            self.kind_index
                .entry(recipe.kind.to_string())
                .or_default()
                .push(id.clone());
        }
    }

    /// Get all active recipes
    pub fn active_recipes(&self) -> Vec<&Recipe> {
        self.recipes.values().filter(|r| r.is_active()).collect()
    }

    /// Get recipes by domain
    pub fn by_domain(&self, domain: &str) -> Vec<&Recipe> {
        self.domain_index
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipe count
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Get stats summary
    pub fn stats(&self) -> RecipeStoreStats {
        let active = self.recipes.values().filter(|r| r.is_active()).count();
        let deprecated = self.recipes.values().filter(|r| r.deprecated).count();
        let total_uses: u32 = self.recipes.values().map(|r| r.use_count).sum();
        let total_successes: u32 = self.recipes.values().map(|r| r.success_count).sum();

        RecipeStoreStats {
            total_recipes: self.recipes.len(),
            active_recipes: active,
            deprecated_recipes: deprecated,
            total_uses,
            total_successes,
            overall_success_rate: if total_uses > 0 {
                total_successes as f32 / total_uses as f32
            } else {
                1.0
            },
        }
    }
}

fn store_count(store: &RecipeStoreV2) -> usize {
    store.recipes.len()
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_engine::RecipeKind;

    #[test]
    fn test_stats() {
        let mut store = RecipeStoreV2::default();
        let mut recipe = Recipe::new("test", "Test", RecipeKind::ProbeOnly, "system");
        recipe.use_count = 10;
        recipe.success_count = 8;
        store.add(recipe);

        let stats = store.stats();
        assert_eq!(stats.total_recipes, 1);
        assert_eq!(stats.total_uses, 10);
    }
}
