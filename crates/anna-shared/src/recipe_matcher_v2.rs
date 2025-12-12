//! Recipe matcher for Anna's learning system.
//! v0.0.418: Finds best matching recipe for a query at runtime.
//!
//! Scoring based on:
//! - Required keyword matches (must all match)
//! - Optional keyword matches (boost score)
//! - Negative keyword presence (disqualify)
//! - Intent match (exact or similar)
//! - Slot compatibility

use crate::recipe_schema::{Recipe, RecipeStatus};
use crate::recipe_storage::RecipeStorage;
use std::collections::HashMap;

/// Result of recipe matching.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched recipe
    pub recipe: Recipe,
    /// Match confidence (0.0-1.0)
    pub confidence: f32,
    /// Breakdown of how the score was calculated
    pub score_breakdown: ScoreBreakdown,
}

/// Breakdown of match score.
#[derive(Debug, Clone, Default)]
pub struct ScoreBreakdown {
    /// Score from required keyword matches
    pub required_score: f32,
    /// Score from optional keyword matches
    pub optional_score: f32,
    /// Score from intent match
    pub intent_score: f32,
    /// Score from slot compatibility
    pub slot_score: f32,
    /// Penalties applied
    pub penalties: f32,
}

/// Query context for matching.
#[derive(Debug, Clone)]
pub struct MatchQuery {
    /// User's original query text
    pub query_text: String,
    /// Detected intent from translator
    pub intent: Option<String>,
    /// Detected domain
    pub domain: Option<String>,
    /// Extracted slots from translator
    pub slots: HashMap<String, String>,
}

impl MatchQuery {
    /// Create a simple query from text only
    pub fn from_text(text: &str) -> Self {
        Self {
            query_text: text.to_string(),
            intent: None,
            domain: None,
            slots: HashMap::new(),
        }
    }

    /// Create a full query with all context
    pub fn new(
        query_text: String,
        intent: Option<String>,
        domain: Option<String>,
        slots: HashMap<String, String>,
    ) -> Self {
        Self {
            query_text,
            intent,
            domain,
            slots,
        }
    }
}

/// Recipe matcher that finds best matching recipes.
pub struct RecipeMatcher<'a> {
    storage: &'a RecipeStorage,
}

impl<'a> RecipeMatcher<'a> {
    pub fn new(storage: &'a RecipeStorage) -> Self {
        Self { storage }
    }

    /// Find the best matching recipe for a query.
    pub fn find_best(&self, query: &MatchQuery) -> Option<MatchResult> {
        let candidates = self.find_candidates(query);
        candidates.into_iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Find all candidate recipes above threshold.
    pub fn find_candidates(&self, query: &MatchQuery) -> Vec<MatchResult> {
        let mut results = Vec::new();

        // Get recipes to consider (filter by domain if specified)
        let recipes: Vec<&Recipe> = if let Some(domain) = &query.domain {
            self.storage.get_by_domain(domain)
        } else {
            self.storage.get_active()
        };

        for recipe in recipes {
            if recipe.status != RecipeStatus::Active {
                continue;
            }

            if let Some(result) = self.score_recipe(recipe, query) {
                if result.confidence >= recipe.matcher.min_confidence {
                    results.push(result);
                }
            }
        }

        // Sort by confidence descending
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Score a recipe against a query.
    fn score_recipe(&self, recipe: &Recipe, query: &MatchQuery) -> Option<MatchResult> {
        let query_lower = query.query_text.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        // Check negative keywords first (disqualify)
        for neg in &recipe.matcher.negative_keywords {
            if query_lower.contains(&neg.to_lowercase()) {
                return None;
            }
        }

        let mut breakdown = ScoreBreakdown::default();

        // Required keywords must ALL match
        let required_matches = recipe
            .matcher
            .required_keywords
            .iter()
            .filter(|kw| query_lower.contains(&kw.to_lowercase()))
            .count();

        if required_matches < recipe.matcher.required_keywords.len() {
            // Not all required keywords present
            return None;
        }

        // Required score: 1.0 if all match
        breakdown.required_score = 1.0;

        // Optional keywords boost score
        if !recipe.matcher.optional_keywords.is_empty() {
            let optional_matches = recipe
                .matcher
                .optional_keywords
                .iter()
                .filter(|kw| query_lower.contains(&kw.to_lowercase()))
                .count();
            breakdown.optional_score =
                optional_matches as f32 / recipe.matcher.optional_keywords.len() as f32;
        }

        // Intent match
        if let (Some(query_intent), Some(recipe_intent)) =
            (&query.intent, &recipe.matcher.exact_intent)
        {
            if query_intent == recipe_intent {
                breakdown.intent_score = 1.0;
            } else if query_intent.contains(recipe_intent) || recipe_intent.contains(query_intent) {
                breakdown.intent_score = 0.5;
            }
        } else if query.intent.is_some() {
            // Query has intent but recipe doesn't require exact match
            breakdown.intent_score = 0.3;
        }

        // Slot compatibility
        if !recipe.pattern.slots.is_empty() {
            let matching_slots = recipe
                .pattern
                .slots
                .iter()
                .filter(|(k, v)| query.slots.get(*k).map(|qv| qv == *v).unwrap_or(false))
                .count();
            breakdown.slot_score = matching_slots as f32 / recipe.pattern.slots.len() as f32;
        }

        // Apply penalties
        // Penalty for low usage (new recipes need to prove themselves)
        if recipe.metrics.times_used < 3 {
            breakdown.penalties += 0.05; // Small penalty for new recipes
        }
        // Penalty for recent failures
        if let Some(rate) = recipe.metrics.recent_success_rate {
            if rate < 0.8 {
                breakdown.penalties += (1.0 - rate) * 0.2;
            }
        }

        // Calculate final confidence
        // Required keywords are most important (0.6), then optional (0.15), intent (0.15), slots (0.1)
        let base_score = breakdown.required_score * 0.6
            + breakdown.optional_score * 0.15
            + breakdown.intent_score * 0.15
            + breakdown.slot_score * 0.1;

        let confidence = (base_score - breakdown.penalties).max(0.0).min(1.0);

        Some(MatchResult {
            recipe: recipe.clone(),
            confidence,
            score_breakdown: breakdown,
        })
    }
}

/// Standalone function to match a recipe.
pub fn match_recipe(
    storage: &RecipeStorage,
    query_text: &str,
    intent: Option<&str>,
    domain: Option<&str>,
    slots: HashMap<String, String>,
) -> Option<MatchResult> {
    let query = MatchQuery {
        query_text: query_text.to_string(),
        intent: intent.map(String::from),
        domain: domain.map(String::from),
        slots,
    };
    let matcher = RecipeMatcher::new(storage);
    matcher.find_best(&query)
}

/// Quick match from text only (no translator context).
pub fn quick_match(storage: &RecipeStorage, query_text: &str) -> Option<MatchResult> {
    let query = MatchQuery::from_text(query_text);
    let matcher = RecipeMatcher::new(storage);
    matcher.find_best(&query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_schema::{PlanStep, RecipeMatcher as SchemaMatcher, RecipePattern};
    use tempfile::tempdir;

    fn make_test_recipe(
        id: &str,
        required: Vec<&str>,
        optional: Vec<&str>,
        negative: Vec<&str>,
    ) -> Recipe {
        let mut recipe = Recipe::new(
            id.into(),
            "desktop".into(),
            "configure_editor".into(),
            RecipePattern {
                user_goal: "test".into(),
                slots: HashMap::new(),
            },
            SchemaMatcher {
                required_keywords: required.into_iter().map(String::from).collect(),
                optional_keywords: optional.into_iter().map(String::from).collect(),
                negative_keywords: negative.into_iter().map(String::from).collect(),
                min_confidence: 0.5, // Lower threshold for tests
                exact_intent: None,
            },
            vec![PlanStep::Explain {
                message: "Test".into(),
            }],
        );
        // Simulate some usage to avoid new-recipe penalty
        recipe.metrics.times_used = 5;
        recipe
    }

    #[test]
    fn test_basic_matching() {
        let dir = tempdir().unwrap();
        let mut storage = RecipeStorage::with_dirs(dir.path().join("user"), dir.path().join("sys"));

        let recipe = make_test_recipe(
            "vim_syntax",
            vec!["vim", "syntax"],
            vec!["highlight", "color"],
            vec!["neovim", "nvim"],
        );
        storage.save(&recipe).unwrap();

        // Should match
        let result = quick_match(&storage, "enable syntax highlighting in vim");
        assert!(result.is_some());
        assert!(result.as_ref().unwrap().confidence >= 0.5);

        // Should NOT match (negative keyword)
        let result = quick_match(&storage, "enable syntax highlighting in neovim");
        assert!(result.is_none());

        // Should NOT match (missing required keyword)
        let result = quick_match(&storage, "enable syntax highlighting");
        assert!(result.is_none());
    }

    #[test]
    fn test_optional_keywords_boost() {
        let dir = tempdir().unwrap();
        let mut storage = RecipeStorage::with_dirs(dir.path().join("user"), dir.path().join("sys"));

        let recipe = make_test_recipe(
            "vim_syntax",
            vec!["vim", "syntax"],
            vec!["highlight", "color", "enable"],
            vec![],
        );
        storage.save(&recipe).unwrap();

        // Query with optional keywords should score higher
        let result1 = quick_match(&storage, "vim syntax").unwrap();
        let result2 = quick_match(&storage, "vim syntax highlight color enable").unwrap();

        assert!(result2.confidence > result1.confidence);
    }

    #[test]
    fn test_intent_matching() {
        let dir = tempdir().unwrap();
        let mut storage = RecipeStorage::with_dirs(dir.path().join("user"), dir.path().join("sys"));

        let mut recipe = make_test_recipe("vim_syntax", vec!["vim", "syntax"], vec![], vec![]);
        recipe.matcher.exact_intent = Some("configure_editor".into());
        storage.save(&recipe).unwrap();

        let query = MatchQuery::new(
            "vim syntax".into(),
            Some("configure_editor".into()),
            Some("desktop".into()),
            HashMap::new(),
        );

        let matcher = RecipeMatcher::new(&storage);
        let result = matcher.find_best(&query).unwrap();

        assert!(result.score_breakdown.intent_score > 0.0);
    }
}
