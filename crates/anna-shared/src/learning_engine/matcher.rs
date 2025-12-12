//! Recipe matcher for learning engine (v0.0.427).
//!
//! Matches incoming questions to existing recipes:
//! - Intent matching
//! - Keyword overlap
//! - Required signal availability
//! - Score-based ranking

use super::{LearnedRecipe, RecipeLibrary};
use serde::{Deserialize, Serialize};

/// Match result for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatch {
    /// Recipe ID
    pub recipe_id: String,
    /// Match score (0.0 to 1.0)
    pub score: f32,
    /// Breakdown of how score was computed
    pub breakdown: MatchBreakdown,
    /// Parameters extracted from question
    pub params: std::collections::HashMap<String, String>,
    /// Missing required signals (probes needed)
    pub missing_signals: Vec<String>,
}

impl RecipeMatch {
    /// Check if this is a strong match (can execute without LLM)
    pub fn is_strong(&self) -> bool {
        self.score >= super::AUTO_EXECUTE_SCORE && self.missing_signals.is_empty()
    }

    /// Check if this is usable (above minimum threshold)
    pub fn is_usable(&self) -> bool {
        self.score >= super::MIN_RECIPE_MATCH_SCORE
    }
}

/// Breakdown of match score components
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchBreakdown {
    /// Intent match score (0.0 to 1.0)
    pub intent_score: f32,
    /// Keyword overlap score (0.0 to 1.0)
    pub keyword_score: f32,
    /// Signal availability score (0.0 to 1.0)
    pub signal_score: f32,
    /// Domain match bonus (0.0 or 0.1)
    pub domain_bonus: f32,
    /// Maturity bonus (based on successful uses)
    pub maturity_bonus: f32,
    /// Health penalty (if recipe has failures)
    pub health_penalty: f32,
}

impl MatchBreakdown {
    /// Compute total score with weights
    pub fn total(&self) -> f32 {
        let base = self.intent_score * 0.4
            + self.keyword_score * 0.3
            + self.signal_score * 0.2
            + self.domain_bonus
            + self.maturity_bonus;
        (base - self.health_penalty).clamp(0.0, 1.0)
    }
}

/// Query for matching recipes
#[derive(Debug, Clone)]
pub struct MatchQuery {
    /// User question (original)
    pub question: String,
    /// Extracted intent
    pub intent: String,
    /// Extracted keywords
    pub keywords: Vec<String>,
    /// Domain hint (if known)
    pub domain: Option<String>,
    /// Available signals (probes already run)
    pub available_signals: Vec<String>,
    /// Extracted parameters
    pub params: std::collections::HashMap<String, String>,
}

impl MatchQuery {
    /// Create a new match query from a question
    pub fn from_question(question: &str) -> Self {
        let intent = super::extract_intent(question);
        let keywords = extract_keywords(question);
        let params = super::extract_params(question).into_iter().collect();

        Self {
            question: question.to_string(),
            intent,
            keywords,
            domain: None,
            available_signals: vec![],
            params,
        }
    }

    /// Add domain hint
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Add available signals
    pub fn with_signals(mut self, signals: &[&str]) -> Self {
        self.available_signals = signals.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Find matching recipes for a query
pub fn find_matches(library: &RecipeLibrary, query: &MatchQuery) -> Vec<RecipeMatch> {
    let mut matches = vec![];

    for recipe in library.enabled() {
        if let Some(match_result) = match_recipe(recipe, query) {
            if match_result.is_usable() {
                matches.push(match_result);
            }
        }
    }

    // Sort by score (descending)
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return top matches
    matches.into_iter().take(5).collect()
}

/// Find the best matching recipe
pub fn find_best_match(library: &RecipeLibrary, query: &MatchQuery) -> Option<RecipeMatch> {
    find_matches(library, query).into_iter().next()
}

/// Match a single recipe against a query
fn match_recipe(recipe: &LearnedRecipe, query: &MatchQuery) -> Option<RecipeMatch> {
    let mut breakdown = MatchBreakdown::default();

    // Intent matching
    breakdown.intent_score = compute_intent_score(&recipe.pattern.intent, &query.intent);

    // If intent is completely wrong, skip
    if breakdown.intent_score < 0.3 {
        return None;
    }

    // Keyword matching
    breakdown.keyword_score = compute_keyword_score(&recipe.pattern.keywords, &query.keywords);

    // Signal availability
    let (signal_score, missing) =
        compute_signal_score(&recipe.pattern.required_signals, &query.available_signals);
    breakdown.signal_score = signal_score;

    // Domain bonus
    if let Some(ref query_domain) = query.domain {
        if recipe.domain.contains(query_domain) || query_domain.contains(&recipe.domain) {
            breakdown.domain_bonus = 0.1;
        }
    }

    // Maturity bonus (reliable recipes get a boost)
    if recipe.stats.is_reliable() {
        breakdown.maturity_bonus = 0.05;
    }

    // Health penalty (recipes with failures get penalized)
    if recipe.stats.uses > 0 && recipe.stats.success_rate() < 0.5 {
        breakdown.health_penalty = 0.15;
    } else if recipe.stats.uses > 0 && recipe.stats.success_rate() < 0.7 {
        breakdown.health_penalty = 0.05;
    }

    let score = breakdown.total();

    // Extract params that match recipe inputs
    let mut matched_params = std::collections::HashMap::new();
    for (key, value) in &query.params {
        if recipe.inputs.params.contains_key(key)
            || recipe.inputs.params.contains_key(&format!("{}?", key))
        {
            matched_params.insert(key.clone(), value.clone());
        }
    }

    Some(RecipeMatch {
        recipe_id: recipe.id.clone(),
        score,
        breakdown,
        params: matched_params,
        missing_signals: missing,
    })
}

/// Compute intent similarity score
fn compute_intent_score(recipe_intent: &str, query_intent: &str) -> f32 {
    if recipe_intent == query_intent {
        return 1.0;
    }

    // Check for partial match
    if recipe_intent.contains(query_intent) || query_intent.contains(recipe_intent) {
        return 0.7;
    }

    // Check for word overlap
    let recipe_words: std::collections::HashSet<_> = recipe_intent.split('_').collect();
    let query_words: std::collections::HashSet<_> = query_intent.split('_').collect();

    let intersection = recipe_words.intersection(&query_words).count();
    let union = recipe_words.union(&query_words).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Compute keyword overlap score (Jaccard similarity)
fn compute_keyword_score(recipe_keywords: &[String], query_keywords: &[String]) -> f32 {
    if recipe_keywords.is_empty() || query_keywords.is_empty() {
        return 0.5; // Neutral if no keywords to compare
    }

    let recipe_set: std::collections::HashSet<_> =
        recipe_keywords.iter().map(|s| s.to_lowercase()).collect();
    let query_set: std::collections::HashSet<_> =
        query_keywords.iter().map(|s| s.to_lowercase()).collect();

    let intersection = recipe_set.intersection(&query_set).count();
    let union = recipe_set.union(&query_set).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Compute signal availability score and return missing signals
fn compute_signal_score(required: &[String], available: &[String]) -> (f32, Vec<String>) {
    if required.is_empty() {
        return (1.0, vec![]); // No requirements = full score
    }

    let available_set: std::collections::HashSet<_> = available.iter().collect();
    let mut missing = vec![];

    for signal in required {
        if !available_set.contains(signal) {
            missing.push(signal.clone());
        }
    }

    let found = required.len() - missing.len();
    let score = found as f32 / required.len() as f32;

    (score, missing)
}

/// Extract keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "to", "of", "in", "for", "on", "with", "at", "by", "from", "or", "and", "not", "no",
        "but", "if", "then", "else", "this", "that", "these", "those", "it", "its", "my", "your",
        "how", "what", "why", "when", "where", "who", "which",
    ];

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_engine::{RecipeOrigin, RecipePattern};

    fn make_recipe(id: &str, intent: &str, keywords: &[&str]) -> LearnedRecipe {
        LearnedRecipe::new(id, "test")
            .with_pattern(RecipePattern::new(intent).with_keywords(keywords))
    }

    #[test]
    fn test_intent_score_exact() {
        assert_eq!(compute_intent_score("check_ram", "check_ram"), 1.0);
    }

    #[test]
    fn test_intent_score_partial() {
        let score = compute_intent_score("check_free_ram", "check_ram");
        assert!(score > 0.5);
    }

    #[test]
    fn test_keyword_score() {
        let recipe_kw = vec!["memory".to_string(), "ram".to_string(), "free".to_string()];
        let query_kw = vec!["memory".to_string(), "available".to_string()];

        let score = compute_keyword_score(&recipe_kw, &query_kw);
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_signal_score() {
        let required = vec!["probe:free".to_string(), "probe:vmstat".to_string()];
        let available = vec!["probe:free".to_string()];

        let (score, missing) = compute_signal_score(&required, &available);
        assert_eq!(score, 0.5);
        assert_eq!(missing.len(), 1);
    }

    #[test]
    fn test_match_query_creation() {
        let query = MatchQuery::from_question("How much RAM is free?");
        assert_eq!(query.intent, "check_free_ram");
        assert!(query.keywords.contains(&"ram".to_string()));
    }

    #[test]
    fn test_find_matches() {
        let mut library = RecipeLibrary::new();
        // Use keywords that overlap better with query "how much ram available"
        library
            .add(make_recipe(
                "ram-check",
                "check_free_ram",
                &["ram", "available", "much"],
            ))
            .unwrap();
        library
            .add(make_recipe(
                "disk-check",
                "check_disk",
                &["disk", "space", "storage"],
            ))
            .unwrap();

        let query = MatchQuery::from_question("How much RAM is available?");
        let matches = find_matches(&library, &query);

        // With good keyword overlap, should match above threshold
        assert!(!matches.is_empty(), "Expected matches for RAM query");
        assert_eq!(matches[0].recipe_id, "ram-check");
    }

    #[test]
    fn test_no_match_wrong_intent() {
        let mut library = RecipeLibrary::new();
        library
            .add(make_recipe("disk-check", "check_disk", &["disk", "space"]))
            .unwrap();

        let query = MatchQuery::from_question("How much RAM is available?");
        let matches = find_matches(&library, &query);

        // Should not match disk recipe to RAM question
        assert!(
            matches.is_empty() || matches[0].score < crate::learning_engine::MIN_RECIPE_MATCH_SCORE
        );
    }
}
