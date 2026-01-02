//! Recipe store query operations (v0.0.423).

use super::{RecipeDomain, RecipeV3};
use super::store_types::RecipeStore;

impl RecipeStore {
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
                    || r.matcher
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&q))
            })
            .collect()
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
