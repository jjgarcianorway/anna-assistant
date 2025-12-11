//! Trigger patterns for recipe matching (v0.0.420).

use serde::{Deserialize, Serialize};

/// A trigger pattern that can activate a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPattern {
    /// Normalized intent like "enable_syntax_vim", "check_boot_time"
    pub intent: String,
    /// Keywords that trigger this pattern ["vim", "syntax", "highlight"]
    pub keywords: Vec<String>,
    /// Minimum matcher confidence to auto-apply this recipe (0.0 - 1.0)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
}

fn default_min_confidence() -> f32 {
    0.7
}

impl TriggerPattern {
    /// Create a new trigger pattern
    pub fn new(intent: &str, keywords: Vec<&str>) -> Self {
        Self {
            intent: intent.to_string(),
            keywords: keywords.into_iter().map(String::from).collect(),
            min_confidence: default_min_confidence(),
        }
    }

    /// Set minimum confidence
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Check if this trigger matches an intent (exact or prefix)
    pub fn matches_intent(&self, intent: &str) -> f32 {
        let self_lower = self.intent.to_lowercase();
        let intent_lower = intent.to_lowercase();

        if self_lower == intent_lower {
            1.0
        } else if intent_lower.starts_with(&self_lower) || self_lower.starts_with(&intent_lower) {
            0.5
        } else {
            0.0
        }
    }

    /// Calculate keyword overlap (Jaccard similarity)
    pub fn keyword_overlap(&self, query_keywords: &[String]) -> f32 {
        if self.keywords.is_empty() || query_keywords.is_empty() {
            return 0.0;
        }

        let self_set: std::collections::HashSet<String> =
            self.keywords.iter().map(|k| k.to_lowercase()).collect();
        let query_set: std::collections::HashSet<String> =
            query_keywords.iter().map(|k| k.to_lowercase()).collect();

        let intersection = self_set.intersection(&query_set).count();
        let union = self_set.union(&query_set).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Calculate combined match score
    pub fn match_score(&self, intent: &str, keywords: &[String]) -> f32 {
        let intent_score = self.matches_intent(intent);
        let keyword_score = self.keyword_overlap(keywords);

        // Weight: 60% intent, 40% keywords
        intent_score * 0.6 + keyword_score * 0.4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_match() {
        let trigger = TriggerPattern::new("enable_vim_syntax", vec!["vim", "syntax"]);
        assert_eq!(trigger.matches_intent("enable_vim_syntax"), 1.0);
        assert_eq!(trigger.matches_intent("enable_vim"), 0.5);
        assert_eq!(trigger.matches_intent("disable_vim_syntax"), 0.0);
    }

    #[test]
    fn test_keyword_overlap() {
        let trigger = TriggerPattern::new("test", vec!["vim", "syntax", "highlight"]);
        let query_kw: Vec<String> = vec!["vim".into(), "syntax".into()];
        let overlap = trigger.keyword_overlap(&query_kw);
        // intersection=2, union=3, so 2/3 ≈ 0.667
        assert!(overlap > 0.6 && overlap < 0.7);
    }
}
