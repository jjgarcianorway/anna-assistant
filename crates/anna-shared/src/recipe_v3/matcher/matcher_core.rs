//! Recipe matching engine core (v0.0.423).
//!
//! This module contains the RecipeMatcher struct and its implementation for
//! scoring and matching recipes against queries.

use std::collections::{HashMap, HashSet};

use crate::recipe_v3::{RecipeV3, MAX_RECIPES_TO_CHECK, MIN_MATCH_SCORE, MIN_SUCCESS_RATE};

use super::matcher_types::{MatchBreakdown, MatchQuery, MatchResult};

/// Recipe matcher
pub struct RecipeMatcher {
    /// Minimum score threshold
    min_score: f32,
    /// Maximum recipes to check
    max_check: usize,
    /// Whether to evaluate preconditions
    eval_preconditions: bool,
}

impl Default for RecipeMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeMatcher {
    /// Create new matcher with defaults
    pub fn new() -> Self {
        Self {
            min_score: MIN_MATCH_SCORE,
            max_check: MAX_RECIPES_TO_CHECK,
            eval_preconditions: true,
        }
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = score;
        self
    }

    /// Set whether to evaluate preconditions
    pub fn with_precondition_eval(mut self, eval: bool) -> Self {
        self.eval_preconditions = eval;
        self
    }

    /// Find matching recipes for a query
    pub fn find_matches(&self, query: &MatchQuery, recipes: &[RecipeV3]) -> Vec<MatchResult> {
        let mut results: Vec<MatchResult> = recipes
            .iter()
            .take(self.max_check)
            .filter(|r| r.enabled)
            .filter_map(|recipe| self.score_recipe(query, recipe))
            .filter(|r| r.score >= self.min_score)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Find best matching recipe
    pub fn find_best(&self, query: &MatchQuery, recipes: &[RecipeV3]) -> Option<MatchResult> {
        self.find_matches(query, recipes).into_iter().next()
    }

    /// Score a single recipe against query
    fn score_recipe(&self, query: &MatchQuery, recipe: &RecipeV3) -> Option<MatchResult> {
        let mut breakdown = MatchBreakdown::default();

        // Domain matching (15%)
        if let Some(ref query_domain) = query.domain {
            let recipe_domain = format!("{:?}", recipe.matcher.domain).to_lowercase();
            if recipe_domain == query_domain.to_lowercase() || recipe_domain == "general" {
                breakdown.domain_score = 1.0;
            }
        } else {
            // No domain specified, slight bonus for general recipes
            breakdown.domain_score = 0.5;
        }

        // Intent matching (35%)
        if let Some(ref query_intent) = query.intent {
            let intent_lower = query_intent.to_lowercase();
            let matches = recipe.matcher.intents.iter().any(|i| {
                i.to_lowercase() == intent_lower || intent_lower.contains(&i.to_lowercase())
            });
            if matches {
                breakdown.intent_score = 1.0;
            } else {
                // Partial match via similarity key
                let sim_key = recipe.matcher.similarity_key.to_lowercase();
                if !sim_key.is_empty()
                    && (intent_lower.contains(&sim_key) || sim_key.contains(&intent_lower))
                {
                    breakdown.intent_score = 0.6;
                }
            }
        }

        // Keyword similarity (30%) - Jaccard index
        if !query.keywords.is_empty() && !recipe.matcher.keywords.is_empty() {
            let query_set: HashSet<_> = query.keywords.iter().map(|s| s.to_lowercase()).collect();
            let recipe_set: HashSet<_> = recipe
                .matcher
                .keywords
                .iter()
                .map(|s| s.to_lowercase())
                .collect();

            let intersection = query_set.intersection(&recipe_set).count();
            let union = query_set.union(&recipe_set).count();

            if union > 0 {
                breakdown.keyword_score = intersection as f32 / union as f32;
            }
        }

        // Entity matching (20%)
        let mut extracted_vars = HashMap::new();
        if !query.entities.is_empty() {
            // Try to match entities against patterns
            for entity in &query.entities {
                for pattern in &recipe.matcher.entity_patterns {
                    if pattern.contains("*") {
                        // Wildcard pattern
                        let prefix = pattern.trim_end_matches('*');
                        if entity.starts_with(prefix) {
                            breakdown.entity_score = 1.0;
                            // Extract the entity as a variable
                            extracted_vars.insert("entity".to_string(), entity.clone());
                            break;
                        }
                    } else if pattern == entity || pattern.to_lowercase() == entity.to_lowercase() {
                        breakdown.entity_score = 1.0;
                        extracted_vars.insert("entity".to_string(), entity.clone());
                        break;
                    }
                }
            }

            // Also try direct entity match in keywords
            if breakdown.entity_score < 1.0 {
                let recipe_kw: HashSet<_> = recipe
                    .matcher
                    .keywords
                    .iter()
                    .map(|s| s.to_lowercase())
                    .collect();
                for entity in &query.entities {
                    if recipe_kw.contains(&entity.to_lowercase()) {
                        breakdown.entity_score = 0.8;
                        extracted_vars.insert("entity".to_string(), entity.clone());
                        break;
                    }
                }
            }
        }

        // Maturity bonus (up to +0.1 for proven recipes)
        if recipe.stats.is_mature() {
            breakdown.maturity_bonus = 0.1 * recipe.stats.success_rate();
        }

        // Health penalty (up to -0.2 for failing recipes)
        if recipe.stats.is_mature() && recipe.stats.success_rate() < MIN_SUCCESS_RATE {
            breakdown.health_penalty = 0.2 * (1.0 - recipe.stats.success_rate());
        }

        let score = breakdown.total();

        // Skip if score is too low
        if score < self.min_score {
            return None;
        }

        // Evaluate preconditions if enabled
        let preconditions_met = if self.eval_preconditions && !recipe.preconditions.is_empty() {
            recipe
                .preconditions
                .iter()
                .all(|cond| cond.evaluate(&extracted_vars).success)
        } else {
            true
        };

        Some(MatchResult {
            recipe: recipe.clone(),
            score,
            breakdown,
            preconditions_met,
            extracted_vars,
        })
    }
}
