//! Recipe storage for learning engine (v0.0.427).
//!
//! Persistent storage for learned recipes with:
//! - JSON file storage
//! - Domain-based indexing
//! - Intent-based lookup
//! - Version tracking

use super::{EvidenceCache, LearnedRecipe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
            self.intent_index
                .entry(intent)
                .or_default()
                .push(id);
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
    fn rebuild_indexes(&mut self) {
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

    /// Get seed recipes
    pub fn seeds(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| r.origin.is_seed).collect()
    }

    /// Get learned recipes (non-seed)
    pub fn learned(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| !r.origin.is_seed).collect()
    }

    /// Get recipes used in last N days
    pub fn recent(&self, days: u32) -> Vec<&LearnedRecipe> {
        let cutoff = now_epoch() - (days as u64 * 24 * 60 * 60);
        self.recipes
            .values()
            .filter(|r| {
                r.stats.last_used_at
                    .as_ref()
                    .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
                    .map(|dt| dt.timestamp() as u64 >= cutoff)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Load library from file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read recipe library: {}", e))?;

        let mut library: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse recipe library: {}", e))?;

        library.rebuild_indexes();
        Ok(library)
    }

    /// Save library to file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize library: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write library: {}", e))?;

        Ok(())
    }

    /// Get default library path
    pub fn default_path() -> PathBuf {
        let state_dir = std::env::var("ANNA_STATE_DIR")
            .unwrap_or_else(|_| "/var/lib/anna".to_string());
        PathBuf::from(state_dir).join("recipes.json")
    }

    /// Get default evidence cache path
    pub fn evidence_cache_path() -> PathBuf {
        let state_dir = std::env::var("ANNA_STATE_DIR")
            .unwrap_or_else(|_| "/var/lib/anna".to_string());
        PathBuf::from(state_dir).join("evidence_cache.json")
    }

    /// Load evidence cache from file
    pub fn load_evidence_cache(&mut self) -> Result<(), String> {
        let path = Self::evidence_cache_path();
        if !path.exists() {
            self.evidence_cache = Some(EvidenceCache::default());
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read evidence cache: {}", e))?;

        let cache: EvidenceCache = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse evidence cache: {}", e))?;

        self.evidence_cache = Some(cache);
        Ok(())
    }

    /// Save evidence cache to file
    pub fn save_evidence_cache(&self) -> Result<(), String> {
        let Some(cache) = &self.evidence_cache else {
            return Ok(());
        };

        let path = Self::evidence_cache_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(cache)
            .map_err(|e| format!("Failed to serialize evidence cache: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write evidence cache: {}", e))?;

        Ok(())
    }
}

/// Get current Unix epoch seconds
fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_engine::{RecipeOrigin, RecipePattern};

    fn make_recipe(id: &str, domain: &str, intent: &str) -> LearnedRecipe {
        LearnedRecipe::new(id, domain)
            .with_pattern(RecipePattern::new(intent))
    }

    #[test]
    fn test_library_add() {
        let mut lib = RecipeLibrary::new();
        let recipe = make_recipe("test-1", "memory", "check_ram");

        lib.add(recipe).unwrap();
        assert_eq!(lib.len(), 1);
        assert!(lib.get("test-1").is_some());
    }

    #[test]
    fn test_library_duplicate() {
        let mut lib = RecipeLibrary::new();
        let recipe = make_recipe("test-1", "memory", "check_ram");

        lib.add(recipe.clone()).unwrap();
        let result = lib.add(recipe);
        assert!(result.is_err());
    }

    #[test]
    fn test_library_indexes() {
        let mut lib = RecipeLibrary::new();
        lib.add(make_recipe("mem-1", "memory", "check_ram")).unwrap();
        lib.add(make_recipe("mem-2", "memory", "check_swap")).unwrap();
        lib.add(make_recipe("disk-1", "disk", "check_disk")).unwrap();

        let memory_recipes = lib.by_domain("memory");
        assert_eq!(memory_recipes.len(), 2);

        let ram_recipes = lib.by_intent("check_ram");
        assert_eq!(ram_recipes.len(), 1);
    }

    #[test]
    fn test_library_disable() {
        let mut lib = RecipeLibrary::new();
        lib.add(make_recipe("test-1", "memory", "check_ram")).unwrap();

        assert!(lib.get("test-1").unwrap().enabled);
        lib.disable("test-1");
        assert!(!lib.get("test-1").unwrap().enabled);

        let enabled = lib.enabled();
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_library_stats() {
        let mut lib = RecipeLibrary::new();
        lib.add(make_recipe("test-1", "memory", "check_ram")).unwrap();

        lib.record_success("test-1");
        lib.record_success("test-1");
        lib.record_failure("test-1");

        let recipe = lib.get("test-1").unwrap();
        assert_eq!(recipe.stats.uses, 3);
        assert_eq!(recipe.stats.successes, 2);
    }
}
