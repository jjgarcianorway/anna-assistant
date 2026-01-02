//! Core semantic similarity matching logic.
//!
//! Uses the translator model to determine if a new query is semantically
//! similar to queries with learned recipes. This enables Anna to reuse
//! learned recipes for paraphrased queries that token matching would miss.
//!
//! Example: "how much disk space" and "what's my storage usage" are semantically
//! similar but share no common tokens.
//!
//! v0.0.293: Added domain guard to prevent cross-domain false matches.
//! v0.0.294: Stricter domain guard - domain-specific queries only match same-domain recipes.
//! v0.0.295: Skip recipes where query is in negative_match_patterns (learned from feedback).

use anna_shared::recipe::Recipe;
use anna_shared::recipe_index::RecipeIndex;
use tracing::{debug, info, warn};

use crate::ollama;

use super::domain::detect_query_domain;
use super::prompts::{build_similarity_prompt, parse_similarity_response};
use super::types::{SimilarityResult, MAX_CANDIDATES, SIMILARITY_THRESHOLD};

/// Check if a new query is semantically similar to any recipe in the index.
/// Uses the translator LLM for semantic comparison.
///
/// This is called when token-based matching fails to find a match.
pub async fn check_semantic_similarity(
    new_query: &str,
    index: &RecipeIndex,
    translator_model: &str,
    timeout_secs: u64,
) -> SimilarityResult {
    // Get top recipe candidates (by token overlap, even if below threshold)
    let candidates = index.search_recipes(new_query, MAX_CANDIDATES);

    if candidates.is_empty() {
        debug!("No recipe candidates for semantic similarity check");
        return SimilarityResult::no_match();
    }

    // v0.0.293: Detect query domain for cross-domain guard
    let new_domain = detect_query_domain(new_query);

    // Check each candidate with LLM
    for (recipe, _token_score) in candidates {
        let original_query = recipe.signature.query_pattern.clone();

        // Skip if queries are identical (already matched by tokens)
        if original_query.to_lowercase() == new_query.to_lowercase() {
            continue;
        }

        // v0.0.295: Skip if query is in negative match patterns (learned from "not helpful" feedback)
        if recipe.is_negative_match(new_query) {
            warn!(
                "Query in negative match list, skipping: {} vs recipe {}",
                new_query, recipe.id
            );
            continue;
        }

        // v0.0.293/294: Domain guard - reject matches between different domains
        // v0.0.294: Stricter - if new query has a domain, recipe MUST have same domain
        let recipe_domain = detect_query_domain(&original_query);
        match (new_domain, recipe_domain) {
            // Both have domains - must match
            (Some(nd), Some(rd)) if nd != rd => {
                warn!(
                    "Domain mismatch, skipping semantic check: {} ({}) vs {} ({})",
                    new_query, nd, original_query, rd
                );
                continue;
            }
            // New query has domain, recipe doesn't - skip (don't match git to general health)
            (Some(nd), None) => {
                warn!(
                    "New query has domain '{}' but recipe has none, skipping: {} vs {}",
                    nd, new_query, original_query
                );
                continue;
            }
            // Recipe has domain, new doesn't - skip (don't match general to specific)
            (None, Some(rd)) => {
                warn!(
                    "Recipe has domain '{}' but new query has none, skipping: {} vs {}",
                    rd, new_query, original_query
                );
                continue;
            }
            // Both match or both have no domain - proceed with LLM check
            _ => {}
        }

        let prompt = build_similarity_prompt(new_query, &original_query);

        match ollama::chat_with_timeout(translator_model, &prompt, timeout_secs).await {
            Ok(response) => {
                if let Some((is_similar, score)) = parse_similarity_response(&response) {
                    if is_similar && score >= SIMILARITY_THRESHOLD {
                        info!(
                            "Semantic match found: \"{}\" ~ \"{}\" (score: {})",
                            new_query, original_query, score
                        );
                        return SimilarityResult {
                            is_similar: true,
                            score,
                            matched_recipe: Some(recipe),
                            original_query: original_query.clone(),
                        };
                    }
                    debug!(
                        "Not similar enough: \"{}\" vs \"{}\" (similar={}, score={})",
                        new_query, original_query, is_similar, score
                    );
                }
            }
            Err(e) => {
                debug!("Similarity check failed: {}", e);
            }
        }
    }

    SimilarityResult::no_match()
}

/// Quick similarity check using just the translator model's classification.
/// Compares domain + intent to see if queries would route the same way.
/// This is faster than full semantic comparison.
pub fn check_classification_similarity(
    new_ticket: &anna_shared::rpc::TranslatorTicket,
    recipe: &Recipe,
) -> bool {
    use anna_shared::teams::Team;

    // Map recipe team to expected domain
    let recipe_domain = match recipe.team {
        Team::Storage => anna_shared::rpc::SpecialistDomain::Storage,
        Team::Network => anna_shared::rpc::SpecialistDomain::Network,
        Team::Security => anna_shared::rpc::SpecialistDomain::Security,
        _ => anna_shared::rpc::SpecialistDomain::System,
    };

    // Check if domains match
    new_ticket.domain == recipe_domain
}
