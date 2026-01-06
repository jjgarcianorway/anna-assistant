//! Recipe matching and execution (v0.0.830).
//!
//! This module handles:
//! - Finding recipes that match parsed queries
//! - Executing recipes by running probes and rendering templates
//!
//! v0.0.830: Improved recipe matching with synonym support and fuzzy matching

use anna_shared::learning_engine::{LearnedRecipe, RecipeLibrary};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::probes;
use super::types::ParsedQuery;

/// Common synonyms for better recipe matching
fn get_synonyms(word: &str) -> Vec<&'static str> {
    match word.to_lowercase().as_str() {
        // Memory related
        "ram" | "memory" => vec!["ram", "memory", "mem"],
        "mem" => vec!["ram", "memory", "mem"],
        // Storage related
        "disk" | "storage" | "space" => vec!["disk", "storage", "space", "drive"],
        "drive" => vec!["disk", "storage", "space", "drive"],
        // Process related
        "process" | "processes" | "running" => vec!["process", "processes", "running", "task"],
        "task" | "tasks" => vec!["process", "processes", "running", "task"],
        // Update related
        "update" | "updates" | "upgrade" => vec!["update", "updates", "upgrade", "patch"],
        "patch" => vec!["update", "updates", "upgrade", "patch"],
        // Service related
        "service" | "services" | "daemon" => vec!["service", "services", "daemon", "unit"],
        "unit" | "units" => vec!["service", "services", "daemon", "unit"],
        // Network related
        "network" | "networking" | "net" => vec!["network", "networking", "net", "connection"],
        "connection" | "connections" => vec!["network", "connection", "connections"],
        // Package related
        "package" | "packages" | "pkg" => vec!["package", "packages", "pkg", "software"],
        "software" => vec!["package", "packages", "pkg", "software"],
        // Check/status related
        "check" | "status" | "show" | "list" => vec!["check", "status", "show", "list", "view"],
        "view" => vec!["check", "status", "show", "list", "view"],
        // Help related
        "help" | "how" | "what" => vec!["help", "how", "what", "explain"],
        // Default
        _ => vec![],
    }
}

/// Normalize a word for matching
fn normalize_word(word: &str) -> String {
    word.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Find a recipe that matches the parsed query
/// v0.0.830: Improved matching with intent normalization and synonym support
pub fn find_matching_recipe(library: &RecipeLibrary, parsed: &ParsedQuery) -> Option<LearnedRecipe> {
    // Normalize intent for matching
    let normalized_intent = normalize_intent(&parsed.intent);

    // First try exact intent match
    let by_intent = library.by_intent(&parsed.intent);
    if !by_intent.is_empty() {
        for recipe in by_intent {
            if recipe.enabled {
                info!("Recipe matched by exact intent: {}", recipe.id);
                return Some(recipe.clone());
            }
        }
    }

    // Try normalized intent match
    let by_normalized = library.by_intent(&normalized_intent);
    if !by_normalized.is_empty() {
        for recipe in by_normalized {
            if recipe.enabled {
                info!("Recipe matched by normalized intent: {}", recipe.id);
                return Some(recipe.clone());
            }
        }
    }

    // Then try domain match with improved keyword scoring
    let by_domain = library.by_domain(&parsed.domain);
    let mut best_match: Option<(LearnedRecipe, u32)> = None;

    // Extract query words and their synonyms
    let query_words: Vec<String> = parsed
        .intent
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(normalize_word)
        .collect();

    // Build expanded word set including synonyms
    let mut expanded_words: std::collections::HashSet<String> = query_words.iter().cloned().collect();
    for word in &query_words {
        for syn in get_synonyms(word) {
            expanded_words.insert(syn.to_string());
        }
    }

    for recipe in by_domain {
        if !recipe.enabled {
            continue;
        }

        let mut score = 0u32;

        // Score by keyword overlap (with synonyms)
        for keyword in &recipe.pattern.keywords {
            let normalized_keyword = normalize_word(keyword);

            // Exact match: 10 points
            if expanded_words.contains(&normalized_keyword) {
                score += 10;
                continue;
            }

            // Synonym match: 8 points
            for syn in get_synonyms(&normalized_keyword) {
                if expanded_words.contains(syn) {
                    score += 8;
                    break;
                }
            }

            // Prefix/suffix match: 5 points
            for query_word in &query_words {
                if query_word.starts_with(&normalized_keyword) ||
                   normalized_keyword.starts_with(query_word) {
                    score += 5;
                    break;
                }
            }
        }

        // Bonus for intent similarity
        let recipe_intent = normalize_word(&recipe.pattern.intent);
        if recipe_intent == normalized_intent {
            score += 20;
        } else if recipe_intent.contains(&normalized_intent) ||
                  normalized_intent.contains(&recipe_intent) {
            score += 10;
        }

        // Minimum score threshold to avoid false matches
        if score >= 10 && (best_match.is_none() || score > best_match.as_ref().unwrap().1) {
            best_match = Some((recipe.clone(), score));
        }
    }

    if let Some((recipe, score)) = &best_match {
        info!("Recipe matched by keywords (score={}): {}", score, recipe.id);
    }

    best_match.map(|(r, _)| r)
}

/// Normalize intent string for better matching
fn normalize_intent(intent: &str) -> String {
    intent
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
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
