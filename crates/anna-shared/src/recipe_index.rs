//! In-memory recipe index for RAG-lite retrieval (v0.0.41).
//!
//! Provides deterministic token-based recipe retrieval with zero LLM calls.
//! Built at daemon start from recipes on disk, stored in daemon state.
//!
//! Retrieval: tokenize user request (lowercase, alnum, split), score = overlap
//! count + boosted matches on targets, deterministic tie-breaker by recipe_id.

use crate::recipe::{recipe_dir, Recipe};
use std::collections::{BTreeMap, BTreeSet};

/// Token boost multipliers for scoring
const TARGET_BOOST: u32 = 3;
const INTENT_TAG_BOOST: u32 = 2;
const BASE_MATCH: u32 = 1;

/// In-memory token index for recipe retrieval (v0.0.41)
/// Uses BTreeMap for deterministic iteration order.
#[derive(Debug, Clone, Default)]
pub struct RecipeIndex {
    /// Token -> set of recipe IDs containing that token
    token_to_recipes: BTreeMap<String, BTreeSet<String>>,
    /// Recipe ID -> Recipe (cached)
    recipes: BTreeMap<String, Recipe>,
}

impl RecipeIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            token_to_recipes: BTreeMap::new(),
            recipes: BTreeMap::new(),
        }
    }

    /// Build index from all recipes on disk
    pub fn build_from_disk() -> Self {
        let mut index = Self::new();
        let dir = recipe_dir();

        if !dir.exists() {
            return index;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(json) = std::fs::read_to_string(entry.path()) {
                    if let Ok(recipe) = serde_json::from_str::<Recipe>(&json) {
                        index.add_recipe(recipe);
                    }
                }
            }
        }

        index
    }

    /// Add a recipe to the index
    pub fn add_recipe(&mut self, recipe: Recipe) {
        let id = recipe.id.clone();

        // Index intent_tags
        for tag in &recipe.intent_tags {
            let tokens = tokenize(tag);
            for token in tokens {
                self.token_to_recipes
                    .entry(token)
                    .or_default()
                    .insert(id.clone());
            }
        }

        // Index targets
        for target in &recipe.targets {
            let tokens = tokenize(target);
            for token in tokens {
                self.token_to_recipes
                    .entry(token)
                    .or_default()
                    .insert(id.clone());
            }
        }

        // Index signature query_pattern
        let query_tokens = tokenize(&recipe.signature.query_pattern);
        for token in query_tokens {
            self.token_to_recipes
                .entry(token)
                .or_default()
                .insert(id.clone());
        }

        // Index signature domain and route_class
        for token in tokenize(&recipe.signature.domain) {
            self.token_to_recipes
                .entry(token)
                .or_default()
                .insert(id.clone());
        }
        for token in tokenize(&recipe.signature.route_class) {
            self.token_to_recipes
                .entry(token)
                .or_default()
                .insert(id.clone());
        }

        self.recipes.insert(id, recipe);
    }

    /// Remove a recipe from the index
    pub fn remove_recipe(&mut self, recipe_id: &str) {
        if let Some(recipe) = self.recipes.remove(recipe_id) {
            // Remove from token index
            for tag in &recipe.intent_tags {
                for token in tokenize(tag) {
                    if let Some(ids) = self.token_to_recipes.get_mut(&token) {
                        ids.remove(recipe_id);
                    }
                }
            }
            for target in &recipe.targets {
                for token in tokenize(target) {
                    if let Some(ids) = self.token_to_recipes.get_mut(&token) {
                        ids.remove(recipe_id);
                    }
                }
            }
            for token in tokenize(&recipe.signature.query_pattern) {
                if let Some(ids) = self.token_to_recipes.get_mut(&token) {
                    ids.remove(recipe_id);
                }
            }
        }
    }

    /// Search for recipes matching a user query
    /// Returns matches sorted by score desc, then recipe_id asc for determinism.
    pub fn search(&self, query: &str, limit: usize) -> Vec<RecipeIndexMatch> {
        let query_tokens: BTreeSet<String> = tokenize(query).into_iter().collect();

        if query_tokens.is_empty() {
            return Vec::new();
        }

        // Collect candidate recipe IDs
        let mut candidates: BTreeSet<&str> = BTreeSet::new();
        for token in &query_tokens {
            if let Some(ids) = self.token_to_recipes.get(token) {
                for id in ids {
                    candidates.insert(id);
                }
            }
        }

        // Score each candidate
        let mut scored: Vec<RecipeIndexMatch> = candidates
            .into_iter()
            .filter_map(|id| {
                self.recipes.get(id).map(|recipe| {
                    let score = self.score_recipe(recipe, &query_tokens);
                    RecipeIndexMatch {
                        recipe_id: id.to_string(),
                        score,
                        matched_tokens: self.get_matched_tokens(recipe, &query_tokens),
                    }
                })
            })
            .filter(|m| m.score > 0)
            .collect();

        // Sort by score desc, then recipe_id asc for determinism
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.recipe_id.cmp(&b.recipe_id))
        });

        scored.truncate(limit);
        scored
    }

    /// Search and return full recipes
    pub fn search_recipes(&self, query: &str, limit: usize) -> Vec<(Recipe, u32)> {
        self.search(query, limit)
            .into_iter()
            .filter_map(|m| {
                self.recipes.get(&m.recipe_id).map(|r| (r.clone(), m.score))
            })
            .collect()
    }

    /// Check if query fully matches a recipe (for zero LLM path)
    /// Returns Some(recipe) if all query tokens match a single recipe's targets/intent_tags.
    pub fn exact_match(&self, query: &str) -> Option<&Recipe> {
        let query_tokens: BTreeSet<String> = tokenize(query).into_iter().collect();

        if query_tokens.is_empty() {
            return None;
        }

        // Find recipes where all query tokens match targets or intent_tags
        for recipe in self.recipes.values() {
            let recipe_tokens: BTreeSet<String> = recipe
                .targets
                .iter()
                .chain(recipe.intent_tags.iter())
                .flat_map(|s| tokenize(s))
                .collect();

            // Check if all query tokens are in recipe tokens
            if query_tokens.iter().all(|t| recipe_tokens.contains(t)) {
                return Some(recipe);
            }
        }

        None
    }

    /// Get recipe by ID
    pub fn get(&self, recipe_id: &str) -> Option<&Recipe> {
        self.recipes.get(recipe_id)
    }

    /// Count of indexed recipes
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Count unique tokens in index
    pub fn token_count(&self) -> usize {
        self.token_to_recipes.len()
    }

    /// Score a recipe against query tokens
    fn score_recipe(&self, recipe: &Recipe, query_tokens: &BTreeSet<String>) -> u32 {
        let mut score: u32 = 0;

        // Score target matches (boosted)
        let target_tokens: BTreeSet<String> = recipe
            .targets
            .iter()
            .flat_map(|s| tokenize(s))
            .collect();
        for token in query_tokens {
            if target_tokens.contains(token) {
                score += TARGET_BOOST;
            }
        }

        // Score intent_tag matches (boosted)
        let intent_tokens: BTreeSet<String> = recipe
            .intent_tags
            .iter()
            .flat_map(|s| tokenize(s))
            .collect();
        for token in query_tokens {
            if intent_tokens.contains(token) {
                score += INTENT_TAG_BOOST;
            }
        }

        // Score query_pattern matches (base)
        let pattern_tokens: BTreeSet<String> =
            tokenize(&recipe.signature.query_pattern).into_iter().collect();
        for token in query_tokens {
            if pattern_tokens.contains(token) {
                score += BASE_MATCH;
            }
        }

        // Bonus for maturity
        if recipe.is_mature() {
            score += 1;
        }

        // Bonus for high reliability
        score += (recipe.reliability_score / 25) as u32;

        score
    }

    /// Get tokens that matched between query and recipe
    fn get_matched_tokens(&self, recipe: &Recipe, query_tokens: &BTreeSet<String>) -> Vec<String> {
        let recipe_tokens: BTreeSet<String> = recipe
            .targets
            .iter()
            .chain(recipe.intent_tags.iter())
            .flat_map(|s| tokenize(s))
            .chain(tokenize(&recipe.signature.query_pattern))
            .collect();

        query_tokens
            .iter()
            .filter(|t| recipe_tokens.contains(*t))
            .cloned()
            .collect()
    }
}

/// Result of a recipe index search
#[derive(Debug, Clone)]
pub struct RecipeIndexMatch {
    pub recipe_id: String,
    pub score: u32,
    pub matched_tokens: Vec<String>,
}

/// Tokenize text for indexing/search (v0.0.41)
/// Lowercase, keep alphanumeric, split on whitespace and punctuation.
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() >= 2) // Skip single chars
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Enable syntax highlighting in Vim");
        assert!(tokens.contains(&"enable".to_string()));
        assert!(tokens.contains(&"syntax".to_string()));
        assert!(tokens.contains(&"highlighting".to_string()));
        assert!(tokens.contains(&"vim".to_string()));
        assert!(tokens.contains(&"in".to_string())); // 2+ chars are kept
    }

    #[test]
    fn test_tokenize_special_chars() {
        let tokens = tokenize("vim-syntax_highlighting.test");
        assert!(tokens.contains(&"vim".to_string()));
        assert!(tokens.contains(&"syntax".to_string()));
        assert!(tokens.contains(&"highlighting".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_empty_index() {
        let index = RecipeIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert_eq!(index.token_count(), 0);
    }

    #[test]
    fn test_search_determinism() {
        // Search results should be deterministic (same input = same output)
        let index = RecipeIndex::new();
        let results1 = index.search("enable syntax vim", 10);
        let results2 = index.search("enable syntax vim", 10);
        assert_eq!(results1.len(), results2.len());
    }
}
