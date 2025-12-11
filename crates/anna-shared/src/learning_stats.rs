//! Learning statistics tracking (v0.0.401).
//!
//! Tracks Anna's learning progress: lessons learned, patterns extracted,
//! facts discovered, probes boosted, etc.

use crate::clarification_learning::ClarificationLearningStore;
use crate::facts::FactsStore;
use crate::facts_types::FactSource;
use crate::probe_learning::ProbeLearningStore;
use crate::recipe::recipe_dir;
use crate::specialist_learning::SpecialistLearningStore;
use serde::{Deserialize, Serialize};

/// Summary of Anna's learning statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningStats {
    /// Total lessons captured from specialist interactions
    pub lessons_total: usize,
    /// Lessons with generic patterns (reusable across similar queries)
    pub lessons_with_patterns: usize,
    /// Pending patterns awaiting more successes
    pub pending_patterns: usize,
    /// Total facts learned (from various sources)
    pub facts_total: usize,
    /// Facts learned from specialist answers
    pub facts_from_specialists: usize,
    /// Number of probe categories with learning data
    pub probe_categories_learned: usize,
    /// Total probe effectiveness entries
    pub probe_entries_total: usize,
    /// Recipes created from learning
    pub recipes_learned: usize,
    /// High confidence lessons (80+)
    pub high_confidence_lessons: usize,
    /// Pattern categories detected
    pub pattern_categories: Vec<String>,
    /// Clarification lessons learned (from user responses)
    pub clarification_lessons: usize,
    /// Trusted clarification patterns (can auto-answer)
    pub clarification_auto_answers: usize,
}

impl LearningStats {
    /// Collect current learning statistics from all stores
    pub fn collect() -> Self {
        let mut stats = Self::default();

        // Specialist learning stats
        let specialist_store = SpecialistLearningStore::load();
        stats.lessons_total = specialist_store.lessons.len();
        stats.pending_patterns = specialist_store.pending_patterns.len();

        let mut pattern_categories = std::collections::HashSet::new();
        for lesson in specialist_store.lessons.values() {
            if lesson.generic_pattern.is_some() {
                stats.lessons_with_patterns += 1;
                if let Some(ref pattern) = lesson.generic_pattern {
                    pattern_categories.insert(format!("{:?}", pattern.category));
                }
            }
            if lesson.confidence >= 80 {
                stats.high_confidence_lessons += 1;
            }
        }
        stats.pattern_categories = pattern_categories.into_iter().collect();

        // Facts stats
        let facts_store = FactsStore::load();
        stats.facts_total = facts_store.verified_count();
        // Count facts from specialist answers
        for fact in facts_store.verified_facts() {
            if let Some(FactSource::SpecialistAnswer) = &fact.fact_source {
                stats.facts_from_specialists += 1;
            }
        }

        // Probe learning stats
        let probe_store = ProbeLearningStore::load();
        stats.probe_categories_learned = probe_store.effectiveness.len();
        stats.probe_entries_total = probe_store
            .effectiveness
            .values()
            .map(|m| m.len())
            .sum();

        // Recipe stats - count recipes in the store
        if let Ok(entries) = std::fs::read_dir(recipe_dir()) {
            stats.recipes_learned = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|s| s == "json").unwrap_or(false))
                .count();
        }

        // Clarification learning stats
        let clarification_store = ClarificationLearningStore::load();
        stats.clarification_lessons = clarification_store.lessons.len();
        stats.clarification_auto_answers = clarification_store.quick_answers.len();

        stats
    }

    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = vec![];

        lines.push("**Anna's Learning Progress:**\n".to_string());

        if self.lessons_total > 0 {
            lines.push(format!(
                "- **Lessons learned:** {} ({} with reusable patterns, {} high-confidence)",
                self.lessons_total, self.lessons_with_patterns, self.high_confidence_lessons
            ));
        }

        if self.pending_patterns > 0 {
            lines.push(format!(
                "- **Pending patterns:** {} (need more successes)",
                self.pending_patterns
            ));
        }

        if !self.pattern_categories.is_empty() {
            lines.push(format!(
                "- **Pattern types learned:** {}",
                self.pattern_categories.join(", ")
            ));
        }

        if self.facts_total > 0 {
            lines.push(format!(
                "- **Facts discovered:** {} ({} from specialist answers)",
                self.facts_total, self.facts_from_specialists
            ));
        }

        if self.probe_entries_total > 0 {
            lines.push(format!(
                "- **Probe effectiveness:** {} entries across {} categories",
                self.probe_entries_total, self.probe_categories_learned
            ));
        }

        if self.recipes_learned > 0 {
            lines.push(format!("- **Recipes created:** {}", self.recipes_learned));
        }

        if self.clarification_lessons > 0 {
            lines.push(format!(
                "- **User preferences learned:** {} ({} can be auto-answered)",
                self.clarification_lessons, self.clarification_auto_answers
            ));
        }

        if lines.len() == 1 {
            lines.push("- No learning data yet. Anna learns from successful interactions!".to_string());
        }

        lines.join("\n")
    }

    /// Check if Anna has learned anything
    pub fn has_learning(&self) -> bool {
        self.lessons_total > 0
            || self.facts_total > 0
            || self.probe_entries_total > 0
            || self.recipes_learned > 0
            || self.clarification_lessons > 0
    }

    /// Get a brief hint about learning progress (for greetings)
    pub fn brief_hint(&self) -> Option<String> {
        if self.lessons_total == 0 {
            return None;
        }

        if self.high_confidence_lessons > 5 {
            Some(format!(
                "I've learned {} patterns from our interactions",
                self.high_confidence_lessons
            ))
        } else if self.lessons_total > 0 {
            Some(format!(
                "I'm learning! {} lessons captured so far",
                self.lessons_total
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_stats() {
        let stats = LearningStats::collect();
        // Should at least return valid stats even with empty stores
        assert!(stats.lessons_total >= 0);
        assert!(stats.facts_total >= 0);
    }

    #[test]
    fn test_summary_empty() {
        let stats = LearningStats::default();
        let summary = stats.summary();
        assert!(summary.contains("Learning Progress"));
    }

    #[test]
    fn test_summary_with_data() {
        let mut stats = LearningStats::default();
        stats.lessons_total = 5;
        stats.high_confidence_lessons = 3;
        stats.lessons_with_patterns = 2;
        stats.facts_total = 10;

        let summary = stats.summary();
        // v0.0.402: Updated to match markdown bold format
        assert!(summary.contains("Lessons learned:**"));
        assert!(summary.contains("high-confidence"));
    }
}
