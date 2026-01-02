//! Learned query patterns with solutions.

use serde::{Deserialize, Serialize};

use super::utils::current_millis;

/// A learned query pattern with solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Pattern ID (hash of keywords)
    pub id: String,
    /// Keywords that trigger this pattern
    pub keywords: Vec<String>,
    /// Domain
    pub domain: String,
    /// Intent
    pub intent: String,
    /// Answer template (with {placeholders})
    pub answer_template: String,
    /// Required probes to run
    pub required_probes: Vec<String>,
    /// Evidence extraction hints
    pub evidence_hints: Vec<EvidenceHint>,
    /// Times this pattern was used successfully
    pub usage_count: u32,
    /// Last success timestamp
    pub last_success: u64,
}

/// Hint for extracting evidence from probe output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHint {
    /// Probe ID
    pub probe_id: String,
    /// Pattern to match in output
    pub match_pattern: String,
    /// Placeholder name in answer template
    pub placeholder: String,
}

impl LearnedPattern {
    pub fn new(keywords: Vec<String>, domain: &str, intent: &str) -> Self {
        let id = compute_pattern_id(&keywords, domain, intent);
        Self {
            id,
            keywords,
            domain: domain.to_string(),
            intent: intent.to_string(),
            answer_template: String::new(),
            required_probes: vec![],
            evidence_hints: vec![],
            usage_count: 0,
            last_success: 0,
        }
    }

    /// Check if pattern matches query
    pub fn matches(&self, keywords: &[String], domain: &str, intent: &str) -> bool {
        if self.domain != domain || self.intent != intent {
            return false;
        }
        // At least half keywords must match
        let matches = keywords
            .iter()
            .filter(|k| {
                self.keywords
                    .iter()
                    .any(|pk| pk.to_lowercase() == k.to_lowercase())
            })
            .count();
        matches >= 1 && matches >= keywords.len() / 2
    }

    /// Record successful usage
    pub fn record_success(&mut self) {
        self.usage_count += 1;
        self.last_success = current_millis();
    }

    /// Check if pattern is trusted enough for direct answers
    pub fn is_trusted(&self) -> bool {
        self.usage_count >= 3
    }
}

/// Compute pattern ID from keywords
fn compute_pattern_id(keywords: &[String], domain: &str, intent: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    intent.hash(&mut hasher);
    for kw in keywords {
        kw.to_lowercase().hash(&mut hasher);
    }
    format!("pat_{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learned_pattern() {
        let pattern = LearnedPattern::new(
            vec!["disk".to_string(), "space".to_string()],
            "storage",
            "diagnose",
        );

        assert!(pattern.matches(&["disk".to_string()], "storage", "diagnose"));
        assert!(!pattern.matches(&["disk".to_string()], "network", "diagnose"));
    }

    #[test]
    fn test_pattern_trust() {
        let mut pattern = LearnedPattern::new(vec!["test".to_string()], "system", "diagnose");

        assert!(!pattern.is_trusted());

        pattern.record_success();
        pattern.record_success();
        pattern.record_success();

        assert!(pattern.is_trusted());
    }
}
