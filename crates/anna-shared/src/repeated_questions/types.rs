//! Type definitions for repeated questions tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::normalization::{calculate_similarity, normalize_question};
use super::category::detect_category;

/// Similarity threshold for considering questions as "same"
pub const SIMILARITY_THRESHOLD: f32 = 0.75;

/// Minimum times a question must appear to be "repeated"
pub const MIN_REPEAT_COUNT: u32 = 2;

/// A recorded question with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedQuestion {
    /// Normalized form of the question
    pub normalized: String,
    /// Original forms seen
    pub variants: Vec<String>,
    /// Number of times asked
    pub count: u32,
    /// Unix timestamp of first occurrence
    pub first_seen: u64,
    /// Unix timestamp of last occurrence
    pub last_seen: u64,
    /// Whether this was answered successfully
    pub resolved: bool,
    /// Category if detected
    pub category: Option<String>,
}

impl RecordedQuestion {
    /// Create a new recorded question
    pub fn new(question: &str, timestamp: u64) -> Self {
        Self {
            normalized: normalize_question(question),
            variants: vec![question.to_string()],
            count: 1,
            first_seen: timestamp,
            last_seen: timestamp,
            resolved: false,
            category: detect_category(question),
        }
    }

    /// Record another occurrence of this question
    pub fn record_occurrence(&mut self, question: &str, timestamp: u64) {
        self.count += 1;
        self.last_seen = timestamp;
        if !self.variants.contains(&question.to_string()) {
            self.variants.push(question.to_string());
        }
    }

    /// Mark as resolved
    pub fn mark_resolved(&mut self) {
        self.resolved = true;
    }

    /// Check if this is a repeated question
    pub fn is_repeated(&self) -> bool {
        self.count >= MIN_REPEAT_COUNT
    }

    /// Get days since first seen
    pub fn days_since_first(&self, now: u64) -> u64 {
        (now - self.first_seen) / 86400
    }
}

/// Question history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestionHistory {
    /// Recorded questions indexed by normalized form
    pub questions: HashMap<String, RecordedQuestion>,
    /// Total questions recorded
    pub total_recorded: u64,
}

impl QuestionHistory {
    /// Create new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a question
    pub fn record(&mut self, question: &str, timestamp: u64) -> Option<&RecordedQuestion> {
        self.total_recorded += 1;
        let normalized = normalize_question(question);

        // Check for exact normalized match
        if let Some(existing) = self.questions.get_mut(&normalized) {
            existing.record_occurrence(question, timestamp);
            return self.questions.get(&normalized);
        }

        // Check for similar questions
        if let Some(similar_key) = self.find_similar(&normalized) {
            if let Some(existing) = self.questions.get_mut(&similar_key) {
                existing.record_occurrence(question, timestamp);
                return self.questions.get(&similar_key);
            }
        }

        // New question
        self.questions
            .insert(normalized.clone(), RecordedQuestion::new(question, timestamp));
        self.questions.get(&normalized)
    }

    /// Find a similar existing question
    fn find_similar(&self, normalized: &str) -> Option<String> {
        for (key, _) in &self.questions {
            if calculate_similarity(key, normalized) >= SIMILARITY_THRESHOLD {
                return Some(key.clone());
            }
        }
        None
    }

    /// Get all repeated questions
    pub fn get_repeated(&self) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.is_repeated())
            .collect()
    }

    /// Get top repeated questions by count
    pub fn top_repeated(&self, limit: usize) -> Vec<&RecordedQuestion> {
        let mut repeated: Vec<_> = self.get_repeated();
        repeated.sort_by(|a, b| b.count.cmp(&a.count));
        repeated.into_iter().take(limit).collect()
    }

    /// Get questions by category
    pub fn by_category(&self, category: &str) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.category.as_deref() == Some(category))
            .collect()
    }

    /// Get unresolved repeated questions
    pub fn unresolved_repeated(&self) -> Vec<&RecordedQuestion> {
        self.questions
            .values()
            .filter(|q| q.is_repeated() && !q.resolved)
            .collect()
    }

    /// Count of repeated questions
    pub fn repeated_count(&self) -> usize {
        self.get_repeated().len()
    }

    /// Mark a question as resolved
    pub fn mark_resolved(&mut self, question: &str) {
        let normalized = normalize_question(question);
        if let Some(q) = self.questions.get_mut(&normalized) {
            q.mark_resolved();
        }
    }

    /// Generate summary
    pub fn summary(&self) -> RepeatedQuestionsSummary {
        let repeated = self.get_repeated();
        let most_repeated = repeated.iter().max_by_key(|q| q.count);

        let mut categories: Vec<String> = repeated
            .iter()
            .filter_map(|q| q.category.clone())
            .collect();
        categories.sort();
        categories.dedup();

        RepeatedQuestionsSummary {
            total_unique: self.questions.len(),
            repeated_count: repeated.len(),
            most_repeated: most_repeated.map(|q| q.variants.first().cloned().unwrap_or_default()),
            most_repeated_count: most_repeated.map(|q| q.count).unwrap_or(0),
            categories_with_repeats: categories,
            unresolved_count: self.unresolved_repeated().len(),
        }
    }
}

/// Summary of repeated questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedQuestionsSummary {
    /// Total unique questions
    pub total_unique: usize,
    /// Number of repeated questions
    pub repeated_count: usize,
    /// Most repeated question
    pub most_repeated: Option<String>,
    /// Most repeated count
    pub most_repeated_count: u32,
    /// Categories with repeats
    pub categories_with_repeats: Vec<String>,
    /// Unresolved repeated count
    pub unresolved_count: usize,
}
