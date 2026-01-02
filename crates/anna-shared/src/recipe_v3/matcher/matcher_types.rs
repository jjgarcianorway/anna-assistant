//! Types for recipe matching (v0.0.423).
//!
//! This module defines the core data structures used in recipe matching:
//! - MatchResult: The result of matching a recipe
//! - MatchBreakdown: Scoring breakdown
//! - MatchQuery: Query context for matching

use std::collections::HashMap;

use crate::recipe_v3::RecipeV3;

use super::matcher_helpers::{detect_domain, detect_intent, extract_entities, extract_keywords};

/// Result of matching a query against recipes
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Matched recipe
    pub recipe: RecipeV3,
    /// Match score (0.0 to 1.0)
    pub score: f32,
    /// Breakdown of scoring
    pub breakdown: MatchBreakdown,
    /// Preconditions that were evaluated
    pub preconditions_met: bool,
    /// Variables extracted from query
    pub extracted_vars: HashMap<String, String>,
}

/// Breakdown of how match score was calculated
#[derive(Debug, Clone, Default)]
pub struct MatchBreakdown {
    /// Domain match contribution
    pub domain_score: f32,
    /// Intent match contribution
    pub intent_score: f32,
    /// Keyword similarity contribution
    pub keyword_score: f32,
    /// Entity match contribution
    pub entity_score: f32,
    /// Maturity bonus
    pub maturity_bonus: f32,
    /// Health penalty (for failing recipes)
    pub health_penalty: f32,
}

impl MatchBreakdown {
    /// Calculate total score
    pub fn total(&self) -> f32 {
        let base = self.domain_score * 0.15
            + self.intent_score * 0.35
            + self.keyword_score * 0.30
            + self.entity_score * 0.20;

        (base + self.maturity_bonus - self.health_penalty).clamp(0.0, 1.0)
    }
}

/// Query context for matching
#[derive(Debug, Clone, Default)]
pub struct MatchQuery {
    /// Original question
    pub question: String,
    /// Detected domain
    pub domain: Option<String>,
    /// Detected intent
    pub intent: Option<String>,
    /// Extracted keywords
    pub keywords: Vec<String>,
    /// Extracted entities (service names, package names, etc.)
    pub entities: Vec<String>,
}

impl MatchQuery {
    /// Create from raw question
    pub fn from_question(question: &str) -> Self {
        let keywords = extract_keywords(question);
        let entities = extract_entities(question);
        let domain = detect_domain(question);
        let intent = detect_intent(question);

        Self {
            question: question.to_string(),
            domain,
            intent,
            keywords,
            entities,
        }
    }

    /// Builder: set domain
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Builder: set intent
    pub fn with_intent(mut self, intent: &str) -> Self {
        self.intent = Some(intent.to_string());
        self
    }

    /// Builder: add entity
    pub fn with_entity(mut self, entity: &str) -> Self {
        self.entities.push(entity.to_string());
        self
    }
}
