//! LLM-based semantic similarity for recipe matching (v0.0.293).
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

use anna_shared::recipe::Recipe;
use anna_shared::recipe_index::RecipeIndex;
use tracing::{debug, info, warn};

use crate::ollama;

/// Minimum similarity score (0-100) to consider queries equivalent
/// v0.0.290: Increased from 75 to 85 to reduce false positives
/// v0.0.293: Increased to 90 - small models are unreliable at semantic judgment
const SIMILARITY_THRESHOLD: u8 = 90;

/// Maximum number of recipe candidates to check with LLM
const MAX_CANDIDATES: usize = 5;

/// Result of semantic similarity check
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub is_similar: bool,
    pub score: u8,
    pub matched_recipe: Option<Recipe>,
    pub original_query: String,
}

impl SimilarityResult {
    fn no_match() -> Self {
        Self {
            is_similar: false,
            score: 0,
            matched_recipe: None,
            original_query: String::new(),
        }
    }
}

/// Build prompt to check semantic similarity between two queries
fn build_similarity_prompt(new_query: &str, recipe_query: &str) -> String {
    format!(
        r#"Are these two Linux queries asking for EXACTLY the same thing?

Query A: "{}"
Query B: "{}"

Reply with ONLY a JSON object: {{"similar": true/false, "score": 0-100, "reason": "brief explanation"}}

STRICT RULES (be very conservative):
- "similar" = true ONLY if both queries would have the EXACT SAME answer
- "score" = confidence 0-100 (100 = identical question)
- If queries are about DIFFERENT topics (e.g., wallpaper vs disk), score must be 0
- If Query B looks like test data (e.g., "test-123"), score must be 0
- Different domains = NOT similar (network vs storage vs desktop)
- Only match if intent + subject are BOTH the same
- When in doubt, return similar=false and low score

Examples of NOT similar:
- "how is my wallpaper loaded" vs "disk usage" → different topics
- "install nano" vs "memory usage" → different topics
- "network speed" vs "disk space" → different domains

Output raw JSON only."#,
        new_query, recipe_query
    )
}

/// Parse similarity response from LLM
fn parse_similarity_response(response: &str) -> Option<(bool, u8)> {
    // Try to extract JSON
    let json_str = if let (Some(s), Some(e)) = (response.find('{'), response.rfind('}')) {
        &response[s..=e]
    } else {
        return None;
    };

    // Parse JSON
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

    let similar = json.get("similar")?.as_bool()?;
    let score = json.get("score")?.as_u64()? as u8;

    Some((similar, score.min(100)))
}

/// v0.0.293: Domain keywords to detect query domain for cross-domain guard
fn detect_query_domain(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();

    // Git/version control
    if q.contains("git") || q.contains("commit") || q.contains("push") || q.contains("pull")
        || q.contains("branch") || q.contains("merge") || q.contains("rebase") {
        return Some("git");
    }

    // Storage/disk
    if q.contains("disk") || q.contains("storage") || q.contains("space") || q.contains("df") {
        return Some("storage");
    }

    // Network
    if q.contains("network") || q.contains("wifi") || q.contains("internet") || q.contains("ip")
        || q.contains("dns") || q.contains("ping") {
        return Some("network");
    }

    // CPU/performance
    if q.contains("cpu") || q.contains("core") || q.contains("processor") || q.contains("load") {
        return Some("cpu");
    }

    // Memory
    if q.contains("memory") || q.contains("ram") || q.contains("swap") {
        return Some("memory");
    }

    // Packages
    if q.contains("install") || q.contains("package") || q.contains("pacman") || q.contains("apt") {
        return Some("packages");
    }

    None
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_similarity_prompt() {
        let prompt = build_similarity_prompt("how much disk space", "what is storage usage");
        assert!(prompt.contains("how much disk space"));
        assert!(prompt.contains("what is storage usage"));
        assert!(prompt.contains("similar"));
    }

    #[test]
    fn test_parse_similarity_response_valid() {
        let response = r#"{"similar": true, "score": 85, "reason": "both ask about disk"}"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
        let (similar, score) = result.unwrap();
        assert!(similar);
        assert_eq!(score, 85);
    }

    #[test]
    fn test_parse_similarity_response_not_similar() {
        let response = r#"{"similar": false, "score": 20, "reason": "different topics"}"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
        let (similar, score) = result.unwrap();
        assert!(!similar);
        assert_eq!(score, 20);
    }

    #[test]
    fn test_parse_similarity_response_with_markdown() {
        let response = r#"Here's the result:
```json
{"similar": true, "score": 90, "reason": "same question"}
```"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_similarity_response_invalid() {
        let response = "I don't understand";
        let result = parse_similarity_response(response);
        assert!(result.is_none());
    }
}
