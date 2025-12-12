//! Recipe Store v2 - Persistent storage for learned recipes (v0.0.412).
//!
//! Stores recipes in a JSON file with indexing for fast lookup.
//! Supports matching, promotion, deprecation, and cleanup.

use crate::recipe_engine::{Recipe, RecipeKind, RiskLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Recipe store with indexing
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RecipeStoreV2 {
    /// All recipes by ID
    pub recipes: HashMap<String, Recipe>,
    /// Index: tag -> recipe IDs
    #[serde(default)]
    pub tag_index: HashMap<String, Vec<String>>,
    /// Index: domain -> recipe IDs
    #[serde(default)]
    pub domain_index: HashMap<String, Vec<String>>,
    /// Index: kind -> recipe IDs
    #[serde(default)]
    pub kind_index: HashMap<String, Vec<String>>,
    /// Store metadata
    #[serde(default)]
    pub metadata: StoreMetadata,
}

/// Store metadata
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StoreMetadata {
    pub version: u32,
    pub last_gc_at: u64,
    pub total_recipes_created: u32,
    pub total_recipes_deprecated: u32,
}

/// Match result for a recipe
#[derive(Debug, Clone)]
pub struct RecipeMatch {
    pub recipe_id: String,
    pub score: f32,
    pub match_type: MatchType,
}

/// How the recipe matched
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchType {
    /// Exact trigger pattern match
    ExactTrigger,
    /// High tag overlap
    TagMatch,
    /// Domain + intent match
    DomainIntent,
    /// Partial match
    Partial,
}

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

    /// Find recipes matching a query
    pub fn find_matches(&self, query: &str, domain: Option<&str>) -> Vec<RecipeMatch> {
        let query_lower = query.to_lowercase();
        let keywords = extract_keywords(&query_lower);
        let mut matches = vec![];

        for recipe in self.recipes.values() {
            if recipe.deprecated || !recipe.is_active() {
                continue;
            }

            // Domain filter
            if let Some(d) = domain {
                if recipe.domain.to_lowercase() != d.to_lowercase() {
                    continue;
                }
            }

            let score = self.compute_match_score(recipe, &query_lower, &keywords);
            if score > 0.3 {
                let match_type = if score >= 0.9 {
                    MatchType::ExactTrigger
                } else if score >= 0.7 {
                    MatchType::TagMatch
                } else if score >= 0.5 {
                    MatchType::DomainIntent
                } else {
                    MatchType::Partial
                };

                matches.push(RecipeMatch {
                    recipe_id: recipe.id.clone(),
                    score,
                    match_type,
                });
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    /// Compute match score for a recipe
    fn compute_match_score(&self, recipe: &Recipe, query: &str, keywords: &[String]) -> f32 {
        let mut score = 0.0f32;

        // Exact trigger match (highest weight)
        for trigger in &recipe.trigger_patterns {
            if query.contains(&trigger.to_lowercase()) {
                score = score.max(0.95);
            } else if trigger
                .to_lowercase()
                .split_whitespace()
                .all(|w| query.contains(w))
            {
                score = score.max(0.85);
            }
        }

        // Tag overlap
        let tag_matches: usize = keywords
            .iter()
            .filter(|k| {
                recipe
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&k.to_lowercase()))
            })
            .count();
        if !keywords.is_empty() && !recipe.tags.is_empty() {
            let tag_score = tag_matches as f32 / keywords.len().min(recipe.tags.len()) as f32;
            score = score.max(tag_score * 0.8);
        }

        // Intent pattern similarity
        let intent_lower = recipe.intent_pattern.to_lowercase();
        let intent_keywords = extract_keywords(&intent_lower);
        let intent_overlap: usize = keywords
            .iter()
            .filter(|k| intent_keywords.contains(k))
            .count();
        if !intent_keywords.is_empty() {
            let intent_score = intent_overlap as f32 / keywords.len().max(1) as f32;
            score = score.max(intent_score * 0.7);
        }

        // Adjust by recipe confidence
        score * recipe.confidence_baseline
    }

    /// Get best match above threshold
    pub fn best_match(
        &self,
        query: &str,
        domain: Option<&str>,
        threshold: f32,
    ) -> Option<RecipeMatch> {
        self.find_matches(query, domain)
            .into_iter()
            .find(|m| m.score >= threshold)
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

/// Recipe store statistics
#[derive(Debug, Clone)]
pub struct RecipeStoreStats {
    pub total_recipes: usize,
    pub active_recipes: usize,
    pub deprecated_recipes: usize,
    pub total_uses: u32,
    pub total_successes: u32,
    pub overall_success_rate: f32,
}

impl std::fmt::Display for RecipeStoreStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[recipe store]")?;
        writeln!(f, "  total_recipes     {}", self.total_recipes)?;
        writeln!(f, "  active            {}", self.active_recipes)?;
        writeln!(f, "  deprecated        {}", self.deprecated_recipes)?;
        writeln!(f, "  total_uses        {}", self.total_uses)?;
        writeln!(
            f,
            "  success_rate      {:.1}%",
            self.overall_success_rate * 100.0
        )?;
        Ok(())
    }
}

/// Extract keywords from query
fn extract_keywords(query: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "what", "why", "how", "when",
        "where", "which", "who", "my", "your", "i", "me", "you", "it", "this", "that", "do",
        "does", "did", "can", "could", "would", "should", "to", "of", "in", "on", "at", "for",
        "with", "by",
    ];

    query
        .split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|w| w.len() >= 3 && !stop_words.contains(&w.as_str()))
        .collect()
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
    fn test_add_and_find() {
        let mut store = RecipeStoreV2::default();
        let recipe = Recipe::new(
            "disk-usage",
            "Check Disk Usage",
            RecipeKind::Inspect,
            "storage",
        )
        .with_tags(vec!["disk", "space", "usage", "df"])
        .with_triggers(vec!["disk usage", "disk space", "what's using space"]);

        store.add(recipe);

        let matches = store.find_matches("how much disk space do I have", Some("storage"));
        assert!(!matches.is_empty());
    }

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
