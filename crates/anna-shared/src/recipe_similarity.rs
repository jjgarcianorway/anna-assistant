//! LLM-based recipe similarity scoring (v0.0.282).
//!
//! Uses the translator LLM to score semantic similarity between queries
//! and learned recipes, enabling intelligent fast-path matching that goes
//! beyond simple keyword matching.

use crate::recipe::Recipe;
use serde::{Deserialize, Serialize};

/// Similarity score result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityScore {
    /// The recipe being compared
    pub recipe_id: String,
    /// Similarity score (0-100)
    pub score: u8,
    /// Whether the queries have the same intent
    pub same_intent: bool,
    /// Whether the queries target the same subject
    pub same_target: bool,
    /// Confidence in the scoring (low, medium, high)
    pub confidence: ScoringConfidence,
    /// Brief explanation of the score
    pub reason: String,
}

/// Confidence level in similarity scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoringConfidence {
    Low,
    Medium,
    High,
}

impl Default for ScoringConfidence {
    fn default() -> Self {
        Self::Medium
    }
}

/// Request for LLM similarity scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityRequest {
    /// The new query to match
    pub query: String,
    /// Candidate recipes to score against
    pub candidates: Vec<RecipeCandidate>,
}

/// A candidate recipe for similarity scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    pub recipe_id: String,
    pub query_pattern: String,
    pub domain: String,
    pub intent_tags: Vec<String>,
    pub answer_preview: String,
}

impl RecipeCandidate {
    /// Create from a Recipe
    pub fn from_recipe(recipe: &Recipe) -> Self {
        let answer_preview = if recipe.answer_template.len() > 100 {
            format!("{}...", &recipe.answer_template[..100])
        } else {
            recipe.answer_template.clone()
        };

        Self {
            recipe_id: recipe.id.clone(),
            query_pattern: recipe.signature.query_pattern.clone(),
            domain: recipe.signature.domain.clone(),
            intent_tags: recipe.intent_tags.clone(),
            answer_preview,
        }
    }
}

/// Build the prompt for LLM similarity scoring
pub fn build_similarity_prompt(request: &SimilarityRequest) -> String {
    let mut prompt = String::new();

    prompt.push_str("Score the semantic similarity between a new query and learned recipes.\n\n");
    prompt.push_str(&format!("NEW QUERY: \"{}\"\n\n", request.query));
    prompt.push_str("CANDIDATE RECIPES:\n");

    for (i, candidate) in request.candidates.iter().enumerate() {
        prompt.push_str(&format!("\n{}. Recipe: {}\n", i + 1, candidate.recipe_id));
        prompt.push_str(&format!("   Pattern: \"{}\"\n", candidate.query_pattern));
        prompt.push_str(&format!("   Domain: {}\n", candidate.domain));
        prompt.push_str(&format!("   Tags: {}\n", candidate.intent_tags.join(", ")));
    }

    prompt.push_str("\nFor each recipe, respond with:\n");
    prompt.push_str("- score: 0-100 (semantic similarity)\n");
    prompt.push_str("- same_intent: true/false (same user goal)\n");
    prompt.push_str("- same_target: true/false (same subject)\n");
    prompt.push_str("- confidence: low/medium/high\n");
    prompt.push_str("- reason: brief explanation\n\n");
    prompt.push_str("Focus on semantic meaning, not exact wording.\n");
    prompt.push_str("\"how much disk space\" and \"disk usage\" are the same intent.\n");
    prompt.push_str("\"install vim\" and \"install nano\" are same intent, different target.\n\n");
    prompt.push_str("Respond as JSON array:\n");
    prompt.push_str("```json\n[\n");
    prompt.push_str("  {\"recipe_id\": \"...\", \"score\": 85, \"same_intent\": true, ");
    prompt.push_str("\"same_target\": true, \"confidence\": \"high\", ");
    prompt.push_str("\"reason\": \"both ask about disk space\"}\n]\n```");

    prompt
}

/// Parse LLM response into similarity scores
pub fn parse_similarity_response(response: &str) -> Vec<SimilarityScore> {
    // Extract JSON from response
    let json_start = response.find('[');
    let json_end = response.rfind(']');

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response[start..=end];
        if let Ok(scores) = serde_json::from_str::<Vec<SimilarityScore>>(json_str) {
            return scores;
        }
    }

    // Try parsing as single object
    if let Ok(score) = serde_json::from_str::<SimilarityScore>(response) {
        return vec![score];
    }

    Vec::new()
}

/// Quick heuristic pre-filter before expensive LLM scoring
pub fn quick_prefilter(query: &str, candidates: &[RecipeCandidate]) -> Vec<RecipeCandidate> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<(usize, &RecipeCandidate)> = candidates
        .iter()
        .map(|c| {
            let pattern_lower = c.query_pattern.to_lowercase();
            let pattern_words: Vec<&str> = pattern_lower.split_whitespace().collect();

            // Score based on word overlap
            let matches = query_words
                .iter()
                .filter(|w| {
                    w.len() > 2
                        && pattern_words
                            .iter()
                            .any(|pw| pw.contains(*w) || w.contains(pw))
                })
                .count();

            // Bonus for domain match
            let domain_bonus = if query_lower.contains(&c.domain.to_lowercase()) {
                2
            } else {
                0
            };

            // Bonus for intent tag match
            let tag_bonus = c
                .intent_tags
                .iter()
                .filter(|t| query_lower.contains(&t.to_lowercase()))
                .count();

            (matches + domain_bonus + tag_bonus, c)
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    // Return top 5 candidates
    scored
        .into_iter()
        .take(5)
        .filter(|(score, _)| *score > 0)
        .map(|(_, c)| c.clone())
        .collect()
}

/// Threshold for using recipe without LLM verification
pub const HIGH_SIMILARITY_THRESHOLD: u8 = 85;

/// Threshold for considering recipe as a match
pub const MATCH_THRESHOLD: u8 = 60;

/// Should we skip LLM specialist based on similarity score?
pub fn should_skip_llm(score: &SimilarityScore, recipe: &Recipe) -> bool {
    // Must have high score and confidence
    if score.score < HIGH_SIMILARITY_THRESHOLD {
        return false;
    }

    if score.confidence != ScoringConfidence::High {
        return false;
    }

    // Must have same intent
    if !score.same_intent {
        return false;
    }

    // Recipe must be mature (used successfully multiple times)
    if !recipe.is_mature() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_prefilter() {
        let candidates = vec![
            RecipeCandidate {
                recipe_id: "disk-1".into(),
                query_pattern: "how much disk space".into(),
                domain: "storage".into(),
                intent_tags: vec!["disk".into(), "space".into(), "storage".into()],
                answer_preview: "...".into(),
            },
            RecipeCandidate {
                recipe_id: "mem-1".into(),
                query_pattern: "memory usage".into(),
                domain: "memory".into(),
                intent_tags: vec!["memory".into(), "ram".into()],
                answer_preview: "...".into(),
            },
        ];

        let filtered = quick_prefilter("check disk usage", &candidates);
        // Both may match due to "usage" in both queries, but disk-1 should rank first
        assert!(!filtered.is_empty());
        assert_eq!(filtered[0].recipe_id, "disk-1");
    }

    #[test]
    fn test_parse_similarity_response() {
        let response = r#"```json
[
  {"recipe_id": "disk-1", "score": 90, "same_intent": true, "same_target": true, "confidence": "high", "reason": "both ask about disk"}
]
```"#;

        let scores = parse_similarity_response(response);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].score, 90);
        assert!(scores[0].same_intent);
    }

    #[test]
    fn test_build_prompt() {
        let request = SimilarityRequest {
            query: "check disk space".into(),
            candidates: vec![RecipeCandidate {
                recipe_id: "disk-1".into(),
                query_pattern: "how much disk".into(),
                domain: "storage".into(),
                intent_tags: vec!["disk".into()],
                answer_preview: "50%".into(),
            }],
        };

        let prompt = build_similarity_prompt(&request);
        assert!(prompt.contains("check disk space"));
        assert!(prompt.contains("disk-1"));
    }
}
