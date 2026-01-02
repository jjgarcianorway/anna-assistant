//! Clarification question logic (Part E).
//!
//! Clarification questions must be rare and precise.

use super::super::intent_schema::CanonicalIntent;

/// Maximum length for clarifying questions.
pub const MAX_CLARIFICATION_LENGTH: usize = 120;

/// Clarification question builder.
#[derive(Debug, Clone)]
pub struct ClarificationBuilder {
    /// Possible clarifying questions by intent.
    questions: Vec<(CanonicalIntent, &'static str)>,
}

impl ClarificationBuilder {
    /// Create new builder with standard questions.
    pub fn new() -> Self {
        Self {
            questions: vec![
                (
                    CanonicalIntent::BootPerf,
                    "Do you mean boot time (startup) or wake-from-sleep resume time?",
                ),
                (
                    CanonicalIntent::MemStatus,
                    "Do you want total RAM, available RAM, or memory usage by application?",
                ),
                (
                    CanonicalIntent::DiskUsage,
                    "Which partition? Root (/), home (/home), or all?",
                ),
                (
                    CanonicalIntent::SvcStatus,
                    "Which service do you want to check?",
                ),
                (
                    CanonicalIntent::NetHealth,
                    "Do you mean WiFi, Ethernet, or DNS connectivity?",
                ),
                (
                    CanonicalIntent::AudioHealth,
                    "Is the issue with playback, recording, or both?",
                ),
            ],
        }
    }

    /// Get clarifying question for an intent.
    pub fn get_question(&self, intent: CanonicalIntent) -> Option<&'static str> {
        self.questions
            .iter()
            .find(|(i, _)| *i == intent)
            .map(|(_, q)| *q)
    }

    /// Build a clarifying question (max 120 chars).
    pub fn build_question(question: &str) -> String {
        if question.len() <= MAX_CLARIFICATION_LENGTH {
            question.to_string()
        } else {
            format!("{}...", &question[..MAX_CLARIFICATION_LENGTH - 3])
        }
    }

    /// Check if clarification is needed based on query ambiguity.
    pub fn needs_clarification(query: &str, intent: CanonicalIntent) -> bool {
        // Only clarify for genuinely ambiguous queries
        let ambiguous_patterns = [
            ("boot", vec!["slow", "time", "fast"]),
            ("memory", vec!["usage", "much"]),
            ("disk", vec!["full", "usage", "space"]),
        ];

        let query_lower = query.to_lowercase();

        // Check if query is too vague
        if query_lower.split_whitespace().count() <= 2 {
            return matches!(intent, CanonicalIntent::Unknown);
        }

        // Don't clarify if query is specific enough
        for (topic, keywords) in ambiguous_patterns.iter() {
            if query_lower.contains(topic) {
                let has_specific = keywords.iter().any(|k| query_lower.contains(k));
                if has_specific {
                    return false; // Specific enough
                }
            }
        }

        // Only clarify for truly unknown intents
        matches!(intent, CanonicalIntent::Unknown)
    }
}

impl Default for ClarificationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarification_max_length() {
        let long_question = "a".repeat(200);
        let truncated = ClarificationBuilder::build_question(&long_question);
        assert!(truncated.len() <= MAX_CLARIFICATION_LENGTH);
    }

    #[test]
    fn test_needs_clarification() {
        // Specific queries don't need clarification
        assert!(!ClarificationBuilder::needs_clarification(
            "how much RAM is available",
            CanonicalIntent::MemStatus
        ));

        // Unknown intents may need clarification
        assert!(ClarificationBuilder::needs_clarification(
            "?",
            CanonicalIntent::Unknown
        ));
    }
}
