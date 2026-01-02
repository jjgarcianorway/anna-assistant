//! Core recipe types (v0.0.423).
//!
//! The Recipe struct and all supporting types for the learning and reuse engine.

// Re-export all public types
pub use super::types_enums::{
    ConfirmationPolicy, RecipeAuthor, RecipeDomain, RecipeOrigin, RecipeRiskLevel,
};
pub use super::types_matcher::RecipeMatcher;
pub use super::types_stats::RecipeStats;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeV3 {
    /// Unique identifier
    pub id: String,
    /// Version number (increments on updates)
    pub version: u32,
    /// Human-readable title
    pub title: String,
    /// Description of what this recipe does
    pub description: String,
    /// Origin (built-in, learned, user-authored)
    pub origin: RecipeOrigin,
    /// Author information
    pub author: RecipeAuthor,
    /// Matching criteria
    pub matcher: RecipeMatcher,
    /// Preconditions that must be true
    pub preconditions: Vec<super::RecipeCondition>,
    /// Steps to execute
    pub steps: Vec<super::RecipeStep>,
    /// Expected outcomes/assertions
    pub postconditions: Vec<super::RecipeCondition>,
    /// Risk level
    pub risk_level: RecipeRiskLevel,
    /// Confirmation policy
    pub confirmation: ConfirmationPolicy,
    /// Source citations
    pub citations: Vec<String>,
    /// Usage statistics
    pub stats: RecipeStats,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether recipe is enabled
    pub enabled: bool,
    /// Source ticket ID (if learned)
    pub source_ticket_id: Option<String>,
    /// Variables/parameters this recipe accepts
    pub parameters: HashMap<String, String>,
}

impl Default for RecipeV3 {
    fn default() -> Self {
        Self {
            id: String::new(),
            version: 1,
            title: String::new(),
            description: String::new(),
            origin: RecipeOrigin::default(),
            author: RecipeAuthor::default(),
            matcher: RecipeMatcher::default(),
            preconditions: vec![],
            steps: vec![],
            postconditions: vec![],
            risk_level: RecipeRiskLevel::default(),
            confirmation: ConfirmationPolicy::default(),
            citations: vec![],
            stats: RecipeStats::default(),
            tags: vec![],
            enabled: true,
            source_ticket_id: None,
            parameters: HashMap::new(),
        }
    }
}

impl RecipeV3 {
    /// Create a new recipe with ID
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            ..Default::default()
        }
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Builder: set origin
    pub fn with_origin(mut self, origin: RecipeOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Builder: set matcher
    pub fn with_matcher(mut self, matcher: RecipeMatcher) -> Self {
        self.matcher = matcher;
        self
    }

    /// Builder: add precondition
    pub fn with_precondition(mut self, cond: super::RecipeCondition) -> Self {
        self.preconditions.push(cond);
        self
    }

    /// Builder: add step
    pub fn with_step(mut self, step: super::RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Builder: set risk level
    pub fn with_risk(mut self, risk: RecipeRiskLevel) -> Self {
        self.risk_level = risk;
        self
    }

    /// Builder: add citation
    pub fn with_citation(mut self, citation: &str) -> Self {
        self.citations.push(citation.to_string());
        self
    }

    /// Builder: add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if recipe is healthy (good success rate)
    pub fn is_healthy(&self) -> bool {
        !self.stats.is_mature() || self.stats.success_rate() >= super::MIN_SUCCESS_RATE
    }

    /// Get primary domain
    pub fn domain(&self) -> RecipeDomain {
        self.matcher.domain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_builder() {
        let recipe = RecipeV3::new("test-1", "Test Recipe")
            .with_description("A test")
            .with_risk(RecipeRiskLevel::Low)
            .with_tag("test");

        assert_eq!(recipe.id, "test-1");
        assert_eq!(recipe.risk_level, RecipeRiskLevel::Low);
        assert!(recipe.tags.contains(&"test".to_string()));
    }
}
