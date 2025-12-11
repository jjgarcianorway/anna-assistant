//! Recipe matching engine (v0.0.420).
//!
//! Matches incoming queries to recipes based on:
//! - Intent match (exact or prefix)
//! - Keyword overlap (Jaccard similarity)
//! - Fact requirements satisfaction

use std::collections::HashMap;

use super::{fact::check_requirements, RecipeV2, AUTO_APPLY_THRESHOLD};

/// Result of matching a recipe
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched recipe
    pub recipe: RecipeV2,
    /// Match score (0.0 - 1.0)
    pub score: f32,
    /// Adjusted score after maturity/reliability factors
    pub adjusted_score: f32,
    /// Which intent matched
    pub matched_intent: String,
    /// Whether this is a high-confidence match
    pub high_confidence: bool,
    /// Facts that were missing (if any)
    pub missing_facts: Vec<String>,
}

impl MatchResult {
    /// Check if this match should be auto-applied (skip specialist)
    pub fn should_auto_apply(&self) -> bool {
        self.high_confidence && self.missing_facts.is_empty()
    }
}

/// Recipe matcher
pub struct RecipeMatcherV2 {
    /// Global threshold for auto-apply
    auto_apply_threshold: f32,
    /// Whether to include disabled recipes in results (for debugging)
    include_disabled: bool,
}

impl Default for RecipeMatcherV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeMatcherV2 {
    /// Create a new matcher
    pub fn new() -> Self {
        Self {
            auto_apply_threshold: AUTO_APPLY_THRESHOLD,
            include_disabled: false,
        }
    }

    /// Set auto-apply threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.auto_apply_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Include disabled recipes (for debugging)
    pub fn with_disabled(mut self) -> Self {
        self.include_disabled = true;
        self
    }

    /// Find the best matching recipe
    pub fn find_best(
        &self,
        recipes: &[&RecipeV2],
        intent: &str,
        keywords: &[String],
        facts: &HashMap<String, String>,
    ) -> Option<MatchResult> {
        let mut results = self.find_all(recipes, intent, keywords, facts);
        results.sort_by(|a, b| {
            b.adjusted_score
                .partial_cmp(&a.adjusted_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.into_iter().next()
    }

    /// Find all matching recipes
    pub fn find_all(
        &self,
        recipes: &[&RecipeV2],
        intent: &str,
        keywords: &[String],
        facts: &HashMap<String, String>,
    ) -> Vec<MatchResult> {
        recipes
            .iter()
            .filter_map(|r| self.match_recipe(r, intent, keywords, facts))
            .collect()
    }

    /// Match a single recipe
    pub fn match_recipe(
        &self,
        recipe: &RecipeV2,
        intent: &str,
        keywords: &[String],
        facts: &HashMap<String, String>,
    ) -> Option<MatchResult> {
        // Skip disabled/unavailable recipes unless debugging
        if !self.include_disabled && !recipe.is_available() {
            return None;
        }

        // Get best trigger score
        let score = recipe.best_trigger_score(intent, keywords);
        if score < 0.1 {
            return None; // No meaningful match
        }

        // Find matched intent
        let matched_intent = recipe
            .trigger_patterns
            .iter()
            .find(|t| t.match_score(intent, keywords) >= score)
            .map(|t| t.intent.clone())
            .unwrap_or_default();

        // Check fact requirements
        let missing = super::fact::missing_facts(&recipe.required_facts, facts);
        let facts_satisfied = missing.is_empty();

        // Apply maturity multiplier
        let maturity_mult = recipe.stats.maturity_multiplier();
        let adjusted_score = score * maturity_mult;

        // Determine if high confidence
        let recipe_min_conf = recipe.min_confidence();
        let high_confidence = adjusted_score >= self.auto_apply_threshold
            && adjusted_score >= recipe_min_conf
            && facts_satisfied
            && recipe.stats.is_reliable();

        Some(MatchResult {
            recipe: recipe.clone(),
            score,
            adjusted_score,
            matched_intent,
            high_confidence,
            missing_facts: missing,
        })
    }
}

/// Quick function to find best matching recipe
pub fn find_best_recipe(
    recipes: &[&RecipeV2],
    intent: &str,
    keywords: &[String],
    facts: &HashMap<String, String>,
) -> Option<MatchResult> {
    RecipeMatcherV2::new().find_best(recipes, intent, keywords, facts)
}

/// Quick function to check if any recipe matches with high confidence
pub fn has_high_confidence_match(
    recipes: &[&RecipeV2],
    intent: &str,
    keywords: &[String],
    facts: &HashMap<String, String>,
) -> bool {
    find_best_recipe(recipes, intent, keywords, facts)
        .map(|r| r.high_confidence)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_v2::{RecipeDomain, TriggerPattern};

    fn make_test_recipe(id: &str, intent: &str, keywords: Vec<&str>) -> RecipeV2 {
        RecipeV2::new(id, id, RecipeDomain::Generic)
            .with_trigger(TriggerPattern::new(intent, keywords))
    }

    #[test]
    fn test_exact_intent_match() {
        let recipe = make_test_recipe("test", "show_memory", vec!["memory", "free", "ram"]);
        let recipes: Vec<&RecipeV2> = vec![&recipe];
        let facts = HashMap::new();

        let result = find_best_recipe(
            &recipes,
            "show_memory",
            &["memory".into(), "free".into()],
            &facts,
        );

        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.score > 0.8); // High score for exact match
        assert_eq!(r.matched_intent, "show_memory");
    }

    #[test]
    fn test_partial_match() {
        let recipe = make_test_recipe("test", "enable_vim_syntax", vec!["vim", "syntax", "highlight"]);
        let recipes: Vec<&RecipeV2> = vec![&recipe];
        let facts = HashMap::new();

        let result = find_best_recipe(
            &recipes,
            "enable_vim",
            &["vim".into()],
            &facts,
        );

        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.score > 0.3); // Partial match has lower score
    }

    #[test]
    fn test_no_match() {
        let recipe = make_test_recipe("test", "show_memory", vec!["memory", "free"]);
        let recipes: Vec<&RecipeV2> = vec![&recipe];
        let facts = HashMap::new();

        let result = find_best_recipe(
            &recipes,
            "configure_vim",
            &["vim".into(), "config".into()],
            &facts,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_missing_facts() {
        use crate::recipe_v2::FactRequirement;

        let recipe = make_test_recipe("test", "enable_vim_syntax", vec!["vim", "syntax"])
            .with_fact(FactRequirement::eq("editor", "vim"));
        let recipes: Vec<&RecipeV2> = vec![&recipe];

        // Without facts - should match but note missing
        let facts = HashMap::new();
        let result = find_best_recipe(
            &recipes,
            "enable_vim_syntax",
            &["vim".into(), "syntax".into()],
            &facts,
        );

        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.missing_facts.is_empty());
        assert!(!r.high_confidence); // Can't be high confidence with missing facts

        // With facts - should be high confidence
        let mut facts = HashMap::new();
        facts.insert("editor".to_string(), "vim".to_string());

        let result = find_best_recipe(
            &recipes,
            "enable_vim_syntax",
            &["vim".into(), "syntax".into()],
            &facts,
        );

        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.missing_facts.is_empty());
    }
}
