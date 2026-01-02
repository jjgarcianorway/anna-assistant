//! Recipe matching and execution.
//!
//! This module handles:
//! - Finding recipes that match parsed queries
//! - Executing recipes by running probes and rendering templates

use anna_shared::learning_engine::{LearnedRecipe, RecipeLibrary};
use std::collections::HashMap;
use tracing::warn;

use crate::probes;
use super::types::ParsedQuery;

/// Find a recipe that matches the parsed query
pub fn find_matching_recipe(library: &RecipeLibrary, parsed: &ParsedQuery) -> Option<LearnedRecipe> {
    // First try exact intent match
    let by_intent = library.by_intent(&parsed.intent);
    if !by_intent.is_empty() {
        for recipe in by_intent {
            if recipe.enabled {
                return Some(recipe.clone());
            }
        }
    }

    // Then try domain match with keyword scoring
    let by_domain = library.by_domain(&parsed.domain);
    let mut best_match: Option<(LearnedRecipe, u32)> = None;

    for recipe in by_domain {
        if !recipe.enabled {
            continue;
        }

        // Score by keyword overlap
        let query_words: std::collections::HashSet<_> = parsed
            .intent
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut score = 0u32;
        for keyword in &recipe.pattern.keywords {
            if query_words.contains(keyword.as_str()) {
                score += 10;
            }
        }

        if score > 0 && (best_match.is_none() || score > best_match.as_ref().unwrap().1) {
            best_match = Some((recipe.clone(), score));
        }
    }

    best_match.map(|(r, _)| r)
}

/// Execute a recipe and return the answer
pub async fn execute_recipe(recipe: &LearnedRecipe, parsed: &ParsedQuery) -> String {
    let mut values: HashMap<String, String> = HashMap::new();

    for probe in &recipe.probes {
        match probes::run_command(&probe.tool) {
            Ok(output) => {
                values.insert(probe.id.clone(), output.trim().to_string());
            }
            Err(e) => {
                warn!("Probe {} failed: {}", probe.id, e);
                values.insert(probe.id.clone(), format!("(error: {})", e));
            }
        }
    }

    for (k, v) in &parsed.entities {
        values.insert(k.clone(), v.clone());
    }

    recipe.answer_template.render_detailed(&values)
}
