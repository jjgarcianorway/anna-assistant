//! Learning loop with recipe statistics (v0.0.432).
//!
//! Tracks successful research patterns and turns them into reusable recipes.
//! No hardcoded answers - everything comes from evidence.

use super::sources::{Citation, KnowledgeSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Learning loop manager.
pub struct LearningLoop {
    /// Stored patterns.
    patterns: HashMap<String, LearnedPattern>,
    /// Storage path.
    storage_path: PathBuf,
    /// Minimum success rate to keep a pattern.
    min_success_rate: f32,
    /// Minimum uses before evaluating.
    min_uses_for_eval: u64,
}

impl LearningLoop {
    /// Create a new learning loop.
    pub fn new(storage_path: &Path) -> Self {
        let mut loop_instance = Self {
            patterns: HashMap::new(),
            storage_path: storage_path.to_path_buf(),
            min_success_rate: 0.7,
            min_uses_for_eval: 5,
        };
        loop_instance.load();
        loop_instance
    }

    /// Load patterns from storage.
    fn load(&mut self) {
        let path = self.storage_path.join("learned_patterns.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(patterns) = serde_json::from_str(&content) {
                    self.patterns = patterns;
                }
            }
        }
    }

    /// Save patterns to storage.
    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(&self.storage_path).map_err(|e| e.to_string())?;
        let path = self.storage_path.join("learned_patterns.json");
        let json = serde_json::to_string_pretty(&self.patterns).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Find matching patterns for a query.
    pub fn find_patterns(&self, query: &str) -> Vec<&LearnedPattern> {
        let mut matches: Vec<_> = self
            .patterns
            .values()
            .filter(|p| p.matches(query))
            .collect();

        // Sort by confidence
        matches.sort_by(|a, b| {
            b.confidence()
                .partial_cmp(&a.confidence())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Learn from a successful research outcome.
    pub fn learn_from_success(
        &mut self,
        query: &str,
        sources: &[KnowledgeSource],
        citations: &[Citation],
        time_ms: u64,
    ) -> LearningOutcome {
        // Check if we already have a matching pattern
        let existing = self
            .patterns
            .values()
            .find(|p| p.matches(query))
            .map(|p| p.id.clone());

        if let Some(id) = existing {
            // Reinforce existing pattern
            let new_confidence = if let Some(pattern) = self.patterns.get_mut(&id) {
                pattern.stats.record_use(true, time_ms);
                pattern.refined_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                pattern.confidence()
            } else {
                return LearningOutcome::Insufficient {
                    reason: "Pattern not found".to_string(),
                };
            };

            let _ = self.save();
            return LearningOutcome::Reinforced {
                pattern_id: id,
                new_confidence,
            };
        }

        // Create new pattern
        let pattern_id = generate_pattern_id(query);
        let query_pattern = extract_query_pattern(query);

        let mut pattern = LearnedPattern::new(&pattern_id, vec![&query_pattern], sources.to_vec());
        pattern.expected_citations = citations.iter().map(|c| c.source.description()).collect();
        pattern.stats.record_use(true, time_ms);

        self.patterns.insert(pattern_id.clone(), pattern);
        let _ = self.save();

        LearningOutcome::NewPattern { pattern_id }
    }

    /// Record a failure for a pattern.
    pub fn record_failure(&mut self, query: &str, time_ms: u64) -> Option<LearningOutcome> {
        let existing = self
            .patterns
            .values()
            .find(|p| p.matches(query))
            .map(|p| p.id.clone());

        if let Some(id) = existing {
            if let Some(pattern) = self.patterns.get_mut(&id) {
                pattern.stats.record_use(false, time_ms);

                // Check if pattern should be deprecated
                if pattern.stats.uses >= self.min_uses_for_eval
                    && pattern.stats.success_rate() < self.min_success_rate
                {
                    let reason = format!(
                        "Success rate {:.0}% below threshold {:.0}%",
                        pattern.stats.success_rate() * 100.0,
                        self.min_success_rate * 100.0
                    );
                    self.patterns.remove(&id);
                    let _ = self.save();
                    return Some(LearningOutcome::Deprecated {
                        pattern_id: id,
                        reason,
                    });
                }

                let _ = self.save();
            }
        }

        None
    }

    /// Record user satisfaction.
    pub fn record_satisfaction(&mut self, query: &str, score: u8) {
        let existing = self
            .patterns
            .values()
            .find(|p| p.matches(query))
            .map(|p| p.id.clone());

        if let Some(id) = existing {
            if let Some(pattern) = self.patterns.get_mut(&id) {
                pattern.stats.record_satisfaction(score);
                let _ = self.save();
            }
        }
    }

    /// Get all pattern statistics.
    pub fn all_stats(&self) -> Vec<(&str, &RecipeStats)> {
        self.patterns
            .iter()
            .map(|(id, p)| (id.as_str(), &p.stats))
            .collect()
    }

    /// Get top performing patterns.
    pub fn top_patterns(&self, limit: usize) -> Vec<&LearnedPattern> {
        let mut patterns: Vec<_> = self.patterns.values().collect();
        patterns.sort_by(|a, b| {
            b.confidence()
                .partial_cmp(&a.confidence())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        patterns.into_iter().take(limit).collect()
    }

    /// Get patterns that need attention (low success rate).
    pub fn patterns_needing_attention(&self) -> Vec<&LearnedPattern> {
        self.patterns
            .values()
            .filter(|p| {
                p.stats.uses >= self.min_uses_for_eval
                    && p.stats.success_rate() < self.min_success_rate + 0.1
            })
            .collect()
    }

    /// Total patterns count.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Clear all patterns (for testing).
    pub fn clear(&mut self) {
        self.patterns.clear();
        let _ = self.save();
    }
}

/// Generate a pattern ID from a query.
fn generate_pattern_id(query: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.to_lowercase().hash(&mut hasher);
    format!("pat_{:x}", hasher.finish())
}

/// Extract a reusable pattern from a query.
fn extract_query_pattern(query: &str) -> String {
    // Simple: lowercase, strip punctuation, and extract key words
    let query_clean: String = query
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();

    let words: Vec<&str> = query_clean
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .filter(|w| !STOP_WORDS.contains(w))
        .collect();

    words.join(" ")
}

/// Common stop words to filter out.
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "can", "what",
    "how", "why", "when", "where", "which", "who", "whom", "this", "that", "these", "those", "i",
    "me", "my", "we", "our", "you", "your", "it", "its",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeStats::default();

        stats.record_use(true, 100);
        stats.record_use(true, 200);
        stats.record_use(false, 150);

        assert_eq!(stats.uses, 3);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 1);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_satisfaction_tracking() {
        let mut stats = RecipeStats::default();

        stats.record_satisfaction(5);
        stats.record_satisfaction(4);
        stats.record_satisfaction(3);

        assert_eq!(stats.avg_satisfaction(), 4.0);
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = LearnedPattern::new(
            "test",
            vec!["memory", "ram"],
            vec![KnowledgeSource::probe("meminfo")],
        );

        assert!(pattern.matches("how much memory do I have?"));
        assert!(pattern.matches("check RAM usage"));
        assert!(!pattern.matches("cpu information"));
    }

    #[test]
    fn test_learning_loop() {
        let path = format!("/tmp/anna_learning_test_{}", std::process::id());
        let mut learning = LearningLoop::new(Path::new(&path));
        learning.clear();

        // Learn from success
        let outcome = learning.learn_from_success(
            "how much memory?",
            &[KnowledgeSource::probe("meminfo")],
            &[],
            50,
        );
        assert!(matches!(outcome, LearningOutcome::NewPattern { .. }));

        // Should find the pattern now
        let patterns = learning.find_patterns("memory usage");
        assert_eq!(patterns.len(), 1);

        // Reinforce
        let outcome2 = learning.learn_from_success(
            "check memory",
            &[KnowledgeSource::probe("meminfo")],
            &[],
            45,
        );
        assert!(matches!(outcome2, LearningOutcome::Reinforced { .. }));

        let _ = fs::remove_dir_all(&path);
    }
}
