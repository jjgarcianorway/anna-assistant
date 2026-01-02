//! Recipe matching logic (v0.0.406).

use super::types::RecipeMatchResult;
use crate::recipe_file::loader::registry;
use crate::rpc::SpecialistDomain;
use std::collections::HashMap;
use tracing::info;

/// Find a matching recipe for the given domain, intent, and params
pub fn find_matching_recipe(
    domain: SpecialistDomain,
    intent: &str,
    params: &HashMap<String, String>,
    query: &str,
) -> Option<RecipeMatchResult> {
    let mut reg = registry();
    let recipes = reg.load();

    let domain_str = domain.to_string().to_lowercase();
    let intent_lower = intent.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut best_match: Option<RecipeMatchResult> = None;
    let mut best_score = 0u32;

    for recipe in recipes.values() {
        // Domain must match
        if recipe.id.domain.to_lowercase() != domain_str {
            continue;
        }

        // Intent must match
        if recipe.match_criteria.intent.to_lowercase() != intent_lower {
            continue;
        }

        let mut score = 50u32; // Base score for domain + intent match
        let mut matched_criteria = vec!["domain".to_string(), "intent".to_string()];

        // Check key match
        if let Some(ref key) = recipe.match_criteria.key {
            if query_lower.contains(&key.to_lowercase()) {
                score += 20;
                matched_criteria.push(format!("key:{}", key));
            }
        }

        // Check target match
        if let Some(ref target) = recipe.match_criteria.target {
            if let Some(param_target) = params.get("target") {
                if param_target.to_lowercase() == target.to_lowercase() {
                    score += 15;
                    matched_criteria.push(format!("target:{}", target));
                }
            }
        }

        // Check keyword matches (any)
        for kw in &recipe.match_criteria.keywords {
            if query_lower.contains(&kw.to_lowercase()) {
                score += 5;
                matched_criteria.push(format!("keyword:{}", kw));
            }
        }

        // Check required keywords (all must match)
        let all_required_match = recipe
            .match_criteria
            .required_keywords
            .iter()
            .all(|kw| query_lower.contains(&kw.to_lowercase()));
        if !recipe.match_criteria.required_keywords.is_empty() {
            if all_required_match {
                score += 25;
                matched_criteria.push("all_required_keywords".to_string());
            } else {
                // Skip this recipe if required keywords don't match
                continue;
            }
        }

        // Check param matches
        for (key, value) in &recipe.match_criteria.params {
            if let Some(param_value) = params.get(key) {
                if param_value.to_lowercase() == value.to_lowercase() {
                    score += 10;
                    matched_criteria.push(format!("param:{}={}", key, value));
                }
            }
        }

        // Convert score to confidence (0-100)
        let confidence = (score.min(100)) as u8;

        // Check minimum confidence threshold
        if confidence < recipe.match_criteria.min_confidence {
            continue;
        }

        if score > best_score {
            best_score = score;
            best_match = Some(RecipeMatchResult {
                recipe: recipe.clone(),
                confidence,
                matched_criteria,
            });
        }
    }

    if let Some(ref m) = best_match {
        info!(
            "Recipe match: {} (confidence={}%, criteria={:?})",
            m.recipe.full_id(),
            m.confidence,
            m.matched_criteria
        );
    }

    best_match
}
