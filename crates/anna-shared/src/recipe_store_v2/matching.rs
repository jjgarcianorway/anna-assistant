//! Recipe matching logic

use crate::recipe_engine::Recipe;
use super::types::{MatchType, RecipeMatch, RecipeStoreV2};

impl RecipeStoreV2 {
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
    pub(crate) fn compute_match_score(&self, recipe: &Recipe, query: &str, keywords: &[String]) -> f32 {
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
}

/// Extract keywords from query
pub(crate) fn extract_keywords(query: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_engine::RecipeKind;

    #[test]
    fn test_find_matches() {
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
}
