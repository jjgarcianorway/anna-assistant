//! Recipe Matcher - Match queries to deterministic recipes
//!
//! Beta.200: Focused module for recipe matching
//!
//! Responsibilities:
//! - Match user queries to known deterministic recipes
//! - Return recipe ID or None if no match
//! - Provide match confidence scoring

use anyhow::Result;

/// Recipe matcher for deterministic action plans
pub struct RecipeMatcher;

impl RecipeMatcher {
    /// Create a new recipe matcher
    pub fn new() -> Self {
        Self
    }

    /// Attempt to match a query to a known recipe
    ///
    /// Returns the recipe name if a match is found, None otherwise.
    ///
    /// Beta.200: This is a stub that delegates to existing recipe infrastructure.
    /// Full implementation will use the recipes module.
    pub fn match_recipe(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();

        // Simple pattern matching for common recipes
        // This will be replaced with full recipe matching logic

        if query_lower.contains("install docker") {
            return Some("docker".to_string());
        }

        if query_lower.contains("install nginx") {
            return Some("nginx".to_string());
        }

        if query_lower.contains("install postgres") {
            return Some("postgresql".to_string());
        }

        // Add more patterns as needed
        None
    }

    /// Get match confidence for a query/recipe pair
    ///
    /// Returns a score from 0.0 (no match) to 1.0 (perfect match)
    pub fn match_confidence(&self, query: &str, recipe: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let recipe_lower = recipe.to_lowercase();

        if query_lower.contains(&recipe_lower) {
            0.9
        } else {
            0.0
        }
    }
}

impl Default for RecipeMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_matching() {
        let matcher = RecipeMatcher::new();

        assert_eq!(
            matcher.match_recipe("install docker"),
            Some("docker".to_string())
        );
        assert_eq!(
            matcher.match_recipe("install nginx"),
            Some("nginx".to_string())
        );
        assert_eq!(matcher.match_recipe("what is my CPU?"), None);
    }

    #[test]
    fn test_match_confidence() {
        let matcher = RecipeMatcher::new();

        assert!(matcher.match_confidence("install docker", "docker") > 0.5);
        assert!(matcher.match_confidence("what is my CPU?", "docker") < 0.1);
    }
}
