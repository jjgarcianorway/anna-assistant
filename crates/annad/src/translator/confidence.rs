//! Deterministic confidence scoring for the Translator.
//!
//! Confidence is calculated using explicit, deterministic rules:
//! - Pattern matches: high confidence (0.90-0.95)
//! - Keyword matches: medium confidence (0.70-0.85)
//! - Fuzzy matches: low confidence (0.40-0.65)
//!
//! No probabilistic or LLM-based scoring.

use super::intent::{ClassificationMethod, IntentAction, UserIntent};
use anna_shared::recipe::Recipe;

/// Confidence thresholds
pub const CONFIDENCE_HIGH: f32 = 0.85;
pub const CONFIDENCE_MEDIUM: f32 = 0.60;
pub const CONFIDENCE_LOW: f32 = 0.40;

/// Confidence score with breakdown
#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    /// Final confidence (0.0 - 1.0)
    pub total: f32,
    /// Breakdown of score components
    pub breakdown: ConfidenceBreakdown,
}

/// Breakdown of how confidence was calculated
#[derive(Debug, Clone)]
pub struct ConfidenceBreakdown {
    /// Score from pattern matching (0.0 - 0.4)
    pub pattern_score: f32,
    /// Score from keyword matching (0.0 - 0.3)
    pub keyword_score: f32,
    /// Score from recipe history (0.0 - 0.2)
    pub recipe_history_score: f32,
    /// Score from context match (0.0 - 0.1)
    pub context_score: f32,
}

impl ConfidenceScore {
    /// Calculate confidence for an intent (without recipe)
    pub fn for_intent(intent: &UserIntent) -> Self {
        // Pattern match: highest confidence (0.50 + 0.30 + 0.10 = 0.90)
        // Keyword match: medium confidence (0.35 + 0.25 + 0.10 = 0.70)
        // Fuzzy match: low confidence (0.20 + 0.15 + 0.10 = 0.45)
        let pattern_score = match intent.classification_method {
            ClassificationMethod::PatternMatch => 0.50,
            ClassificationMethod::KeywordMatch => 0.35,
            ClassificationMethod::FuzzyMatch => 0.20,
            ClassificationMethod::Unknown => 0.0,
        };

        let keyword_score = match intent.classification_method {
            ClassificationMethod::PatternMatch => 0.30, // Pattern includes keywords
            ClassificationMethod::KeywordMatch => 0.25,
            ClassificationMethod::FuzzyMatch => 0.15,
            ClassificationMethod::Unknown => 0.0,
        };

        let context_score = if intent.action != IntentAction::Unknown {
            0.10
        } else {
            0.0
        };

        let breakdown = ConfidenceBreakdown {
            pattern_score,
            keyword_score,
            recipe_history_score: 0.0,
            context_score,
        };

        Self {
            total: pattern_score + keyword_score + context_score,
            breakdown,
        }
    }

    /// Calculate confidence for intent + recipe combination
    pub fn for_intent_and_recipe(intent: &UserIntent, recipe: &Recipe) -> Self {
        let mut score = Self::for_intent(intent);

        // Add recipe history score
        let history_score = calculate_recipe_history_score(recipe);
        score.breakdown.recipe_history_score = history_score;
        score.total = (score.total + history_score).min(0.99);

        score
    }

    /// Check if this is high confidence
    pub fn is_high(&self) -> bool {
        self.total >= CONFIDENCE_HIGH
    }

    /// Check if this is medium confidence
    pub fn is_medium(&self) -> bool {
        self.total >= CONFIDENCE_MEDIUM && self.total < CONFIDENCE_HIGH
    }

    /// Check if this is low confidence
    pub fn is_low(&self) -> bool {
        self.total < CONFIDENCE_MEDIUM
    }

    /// Get a human-readable confidence level
    pub fn level(&self) -> &'static str {
        if self.is_high() {
            "high"
        } else if self.is_medium() {
            "medium"
        } else {
            "low"
        }
    }
}

/// Calculate recipe history score based on success count
fn calculate_recipe_history_score(recipe: &Recipe) -> f32 {
    if recipe.success_count == 0 {
        return 0.05; // Small base score for enabled recipes
    }

    // Logarithmic scaling: 1 success = ~0.06, 10 = ~0.12, 100 = ~0.18, 500 = ~0.20
    let raw = (recipe.success_count as f32).ln_1p() / 30.0;
    raw.min(0.20)
}

/// Calculate confidence boost from parameter extraction
pub fn parameter_confidence_boost(params: &[String]) -> f32 {
    match params.len() {
        0 => 0.0,
        1 => 0.05,
        2 => 0.08,
        _ => 0.10,
    }
}

/// Calculate confidence penalty for ambiguous input
pub fn ambiguity_penalty(input: &str) -> f32 {
    let lower = input.to_lowercase();

    // Penalty for very short inputs
    if input.len() < 10 {
        return 0.1;
    }

    // Penalty for question words without specific context
    let vague_patterns = ["something", "anything", "whatever", "stuff", "things"];
    for pattern in vague_patterns {
        if lower.contains(pattern) {
            return 0.15;
        }
    }

    // No penalty
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::intent::IntentSubject;
    use anna_shared::recipe::RecipeSource;

    fn test_intent() -> UserIntent {
        UserIntent {
            action: IntentAction::Query,
            subject: IntentSubject::DiskUsage,
            subject_raw: "disk".to_string(),
            parameters: vec![],
            original_input: "how much disk space".to_string(),
            confidence: 0.9,
            classification_method: ClassificationMethod::PatternMatch,
        }
    }

    fn test_recipe(success_count: u32) -> Recipe {
        Recipe {
            id: "test".to_string(),
            name: "Test".to_string(),
            keywords: vec![],
            patterns: vec![],
            context: Default::default(),
            commands: vec![],
            verification: None,
            source: RecipeSource::BuiltIn,
            success_count,
            last_used: None,
            enabled: true,
        }
    }

    #[test]
    fn test_confidence_for_pattern_match() {
        let intent = test_intent();
        let score = ConfidenceScore::for_intent(&intent);
        assert!(score.is_high());
        assert!(score.breakdown.pattern_score > 0.3);
    }

    #[test]
    fn test_confidence_with_recipe_history() {
        let intent = test_intent();
        let recipe = test_recipe(100);
        let score = ConfidenceScore::for_intent_and_recipe(&intent, &recipe);
        assert!(score.total > ConfidenceScore::for_intent(&intent).total);
    }

    #[test]
    fn test_recipe_history_score_scaling() {
        let score_0 = calculate_recipe_history_score(&test_recipe(0));
        let score_10 = calculate_recipe_history_score(&test_recipe(10));
        let score_100 = calculate_recipe_history_score(&test_recipe(100));

        assert!(score_0 < score_10);
        assert!(score_10 < score_100);
        assert!(score_100 <= 0.20);
    }

    #[test]
    fn test_ambiguity_penalty() {
        assert!(ambiguity_penalty("hi") > 0.0); // Too short
        assert!(ambiguity_penalty("do something with my disk") > 0.0); // Vague
        assert_eq!(ambiguity_penalty("check my disk usage"), 0.0); // Clear
    }

    #[test]
    fn test_confidence_level() {
        let high = ConfidenceScore {
            total: 0.90,
            breakdown: ConfidenceBreakdown {
                pattern_score: 0.4,
                keyword_score: 0.3,
                recipe_history_score: 0.15,
                context_score: 0.05,
            },
        };
        assert_eq!(high.level(), "high");

        let medium = ConfidenceScore {
            total: 0.70,
            breakdown: ConfidenceBreakdown {
                pattern_score: 0.3,
                keyword_score: 0.25,
                recipe_history_score: 0.10,
                context_score: 0.05,
            },
        };
        assert_eq!(medium.level(), "medium");
    }
}
