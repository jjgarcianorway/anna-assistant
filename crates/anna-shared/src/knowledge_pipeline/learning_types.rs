//! Learning types and data structures (v0.0.432).

use super::sources::KnowledgeSource;
use serde::{Deserialize, Serialize};

/// Recipe statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStats {
    /// Times this recipe was used.
    pub uses: u64,
    /// Times it succeeded.
    pub successes: u64,
    /// Times it failed.
    pub failures: u64,
    /// Average execution time (ms).
    pub avg_time_ms: u64,
    /// Last used timestamp (unix).
    pub last_used: u64,
    /// User satisfaction scores (1-5).
    pub satisfaction_scores: Vec<u8>,
}

impl RecipeStats {
    /// Success rate (0.0 to 1.0).
    pub fn success_rate(&self) -> f32 {
        if self.uses == 0 {
            0.0
        } else {
            self.successes as f32 / self.uses as f32
        }
    }

    /// Average satisfaction (1.0 to 5.0).
    pub fn avg_satisfaction(&self) -> f32 {
        if self.satisfaction_scores.is_empty() {
            0.0
        } else {
            self.satisfaction_scores
                .iter()
                .map(|&s| s as f32)
                .sum::<f32>()
                / self.satisfaction_scores.len() as f32
        }
    }

    /// Record a use.
    pub fn record_use(&mut self, success: bool, time_ms: u64) {
        self.uses += 1;
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
        // Update running average
        self.avg_time_ms = (self.avg_time_ms * (self.uses - 1) + time_ms) / self.uses;
        self.last_used = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Record satisfaction.
    pub fn record_satisfaction(&mut self, score: u8) {
        let clamped = score.clamp(1, 5);
        self.satisfaction_scores.push(clamped);
        // Keep only last 100 scores
        if self.satisfaction_scores.len() > 100 {
            self.satisfaction_scores.remove(0);
        }
    }
}

/// A learned pattern from successful research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Pattern identifier.
    pub id: String,
    /// Query patterns that match this (regex-like).
    pub query_patterns: Vec<String>,
    /// Sources to use for this pattern.
    pub sources: Vec<KnowledgeSource>,
    /// Expected citations.
    pub expected_citations: Vec<String>,
    /// Statistics.
    pub stats: RecipeStats,
    /// When this pattern was learned.
    pub learned_at: u64,
    /// Last refinement timestamp.
    pub refined_at: u64,
}

impl LearnedPattern {
    /// Create a new pattern.
    pub fn new(id: &str, query_patterns: Vec<&str>, sources: Vec<KnowledgeSource>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id: id.to_string(),
            query_patterns: query_patterns.into_iter().map(String::from).collect(),
            sources,
            expected_citations: Vec::new(),
            stats: RecipeStats::default(),
            learned_at: now,
            refined_at: now,
        }
    }

    /// Check if this pattern matches a query.
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.query_patterns.iter().any(|pattern| {
            // Check if any keyword from the pattern is in the query
            let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
            pattern_words.iter().any(|word| query_lower.contains(word))
        })
    }

    /// Confidence in this pattern (based on stats).
    pub fn confidence(&self) -> f32 {
        let success_factor = self.stats.success_rate();
        let usage_factor = (self.stats.uses as f32 / 10.0).min(1.0); // More uses = more confidence
        let satisfaction_factor = self.stats.avg_satisfaction() / 5.0;

        (success_factor * 0.5 + usage_factor * 0.3 + satisfaction_factor * 0.2).min(1.0)
    }
}

/// Outcome of a learning attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningOutcome {
    /// New pattern learned.
    NewPattern { pattern_id: String },
    /// Existing pattern reinforced.
    Reinforced {
        pattern_id: String,
        new_confidence: f32,
    },
    /// Pattern deprecated (too many failures).
    Deprecated { pattern_id: String, reason: String },
    /// Not enough data to learn.
    Insufficient { reason: String },
}
