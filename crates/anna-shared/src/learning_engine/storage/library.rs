//! Recipe library core data structure and operations.

use super::now_epoch;
use crate::learning_engine::{EvidenceCache, LearnedRecipe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recipe library with persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeLibrary {
    /// All recipes by ID
    recipes: HashMap<String, LearnedRecipe>,
    /// Index by domain
    #[serde(skip)]
    domain_index: HashMap<String, Vec<String>>,
    /// Index by intent
    #[serde(skip)]
    intent_index: HashMap<String, Vec<String>>,
    /// Evidence cache (separate file)
    #[serde(skip)]
    evidence_cache: Option<EvidenceCache>,
    /// Last modification timestamp
    pub last_modified: u64,
    /// Library version
    pub version: u32,
}

impl Default for RecipeLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
            domain_index: HashMap::new(),
            intent_index: HashMap::new(),
            evidence_cache: None,
            last_modified: now_epoch(),
            version: 1,
        }
    }

    /// Add a recipe to the library
    pub fn add(&mut self, recipe: LearnedRecipe) -> Result<(), String> {
        if recipe.id.is_empty() {
            return Err("Recipe ID cannot be empty".to_string());
        }

        // Check for duplicate
        if self.recipes.contains_key(&recipe.id) {
            return Err(format!("Recipe '{}' already exists", recipe.id));
        }

        let id = recipe.id.clone();
        let domain = recipe.domain.clone();
        let intent = recipe.pattern.intent.clone();

        self.recipes.insert(id.clone(), recipe);

        // Update indexes
        self.domain_index
            .entry(domain)
            .or_default()
            .push(id.clone());
        if !intent.is_empty() {
            self.intent_index.entry(intent).or_default().push(id);
        }

        self.last_modified = now_epoch();
        Ok(())
    }

    /// Update an existing recipe
    pub fn update(&mut self, recipe: LearnedRecipe) -> Result<(), String> {
        if !self.recipes.contains_key(&recipe.id) {
            return Err(format!("Recipe '{}' not found", recipe.id));
        }

        // Increment version
        let mut updated = recipe;
        updated.version += 1;

        self.recipes.insert(updated.id.clone(), updated);
        self.last_modified = now_epoch();
        self.rebuild_indexes();
        Ok(())
    }

    /// Get a recipe by ID
    pub fn get(&self, id: &str) -> Option<&LearnedRecipe> {
        self.recipes.get(id)
    }

    /// Get mutable recipe by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LearnedRecipe> {
        self.recipes.get_mut(id)
    }

    /// Get recipes by domain
    pub fn by_domain(&self, domain: &str) -> Vec<&LearnedRecipe> {
        self.domain_index
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get recipes by intent
    pub fn by_intent(&self, intent: &str) -> Vec<&LearnedRecipe> {
        self.intent_index
            .get(intent)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all enabled recipes
    pub fn enabled(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| r.enabled).collect()
    }

    /// Get all recipes
    pub fn all(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().collect()
    }

    /// Get recipe count
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Remove a recipe
    pub fn remove(&mut self, id: &str) -> Option<LearnedRecipe> {
        let recipe = self.recipes.remove(id)?;
        self.rebuild_indexes();
        self.last_modified = now_epoch();
        Some(recipe)
    }

    /// Disable a recipe
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.enabled = false;
            self.last_modified = now_epoch();
            true
        } else {
            false
        }
    }

    /// Enable a recipe
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.enabled = true;
            self.last_modified = now_epoch();
            true
        } else {
            false
        }
    }

    /// Record recipe usage (success)
    pub fn record_success(&mut self, id: &str) {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.stats.record_success();
            self.last_modified = now_epoch();
        }
    }

    /// Record recipe usage (failure)
    pub fn record_failure(&mut self, id: &str) {
        if let Some(recipe) = self.recipes.get_mut(id) {
            recipe.stats.record_failure();
            self.last_modified = now_epoch();
        }
    }

    /// Rebuild indexes after modifications
    pub(crate) fn rebuild_indexes(&mut self) {
        self.domain_index.clear();
        self.intent_index.clear();

        for (id, recipe) in &self.recipes {
            self.domain_index
                .entry(recipe.domain.clone())
                .or_default()
                .push(id.clone());
            if !recipe.pattern.intent.is_empty() {
                self.intent_index
                    .entry(recipe.pattern.intent.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }

    /// Get evidence cache (lazy load)
    pub fn evidence_cache(&mut self) -> &mut EvidenceCache {
        if self.evidence_cache.is_none() {
            self.evidence_cache = Some(EvidenceCache::default());
        }
        self.evidence_cache.as_mut().unwrap()
    }

    /// Set evidence cache
    pub fn set_evidence_cache(&mut self, cache: EvidenceCache) {
        self.evidence_cache = Some(cache);
    }

    /// Get reference to evidence cache if it exists
    pub(crate) fn evidence_cache_ref(&self) -> Option<&EvidenceCache> {
        self.evidence_cache.as_ref()
    }

    /// Get seed recipes
    pub fn seeds(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| r.origin.is_seed).collect()
    }

    /// Get learned recipes (non-seed)
    pub fn learned(&self) -> Vec<&LearnedRecipe> {
        self.recipes
            .values()
            .filter(|r| !r.origin.is_seed)
            .collect()
    }

    /// Get recipes used in last N days
    pub fn recent(&self, days: u32) -> Vec<&LearnedRecipe> {
        let cutoff = now_epoch() - (days as u64 * 24 * 60 * 60);
        self.recipes
            .values()
            .filter(|r| {
                r.stats
                    .last_used_at
                    .as_ref()
                    .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
                    .map(|dt| dt.timestamp() as u64 >= cutoff)
                    .unwrap_or(false)
            })
            .collect()
    }
}
