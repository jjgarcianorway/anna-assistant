//! Learning Engine - Pattern learning with time decay
//!
//! v6.47.0: Learn user preferences and interaction patterns over time

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learning engine that tracks patterns with decay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEngine {
    /// Learned patterns
    patterns: HashMap<String, LearnedPattern>,

    /// Decay rate (days until 50% confidence)
    decay_half_life_days: i64,
}

/// A learned pattern with time decay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Pattern key (e.g., "preferred_personality_style")
    pub key: String,

    /// Pattern value (e.g., "Friendly")
    pub value: String,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,

    /// When this pattern was last reinforced
    pub last_reinforced: DateTime<Utc>,

    /// Number of times observed
    pub observation_count: u32,
}

/// Pattern observation for reinforcement
#[derive(Debug, Clone)]
pub struct Observation {
    pub key: String,
    pub value: String,
    pub weight: f64, // How much to reinforce (0.0 - 1.0)
}

impl LearningEngine {
    /// Create new learning engine with specified decay rate
    pub fn new(decay_half_life_days: i64) -> Self {
        Self {
            patterns: HashMap::new(),
            decay_half_life_days,
        }
    }

    /// Create with default decay (30 days)
    pub fn default() -> Self {
        Self::new(30)
    }

    /// Record an observation
    pub fn observe(&mut self, observation: Observation, now: DateTime<Utc>) {
        let pattern_exists = self.patterns.contains_key(&observation.key);

        if pattern_exists {
            // Pattern exists, reinforce it
            let pattern = self.patterns.get_mut(&observation.key).unwrap();

            // Calculate decay inline to avoid borrowing issues
            let time_passed = now.signed_duration_since(pattern.last_reinforced);
            let days_passed = time_passed.num_days() as f64;
            let decay_factor = if days_passed > 0.0 {
                0.5_f64.powf(days_passed / self.decay_half_life_days as f64)
            } else {
                1.0
            };
            let current_confidence = (pattern.confidence * decay_factor).max(0.0);

            if pattern.value == observation.value {
                // Same value, increase confidence
                pattern.confidence = (current_confidence + observation.weight * 0.2).min(0.95);
                pattern.observation_count += 1;
            } else {
                // Different value, decrease confidence
                pattern.confidence = (current_confidence - observation.weight * 0.3).max(0.1);

                // If confidence drops too low, replace value
                if pattern.confidence < 0.3 {
                    pattern.value = observation.value.clone();
                    pattern.confidence = observation.weight.min(0.6);
                    pattern.observation_count = 1;
                }
            }

            pattern.last_reinforced = now;
        } else {
            // New pattern
            self.patterns.insert(
                observation.key.clone(),
                LearnedPattern {
                    key: observation.key,
                    value: observation.value,
                    confidence: observation.weight.min(0.8), // Initial confidence capped
                    last_reinforced: now,
                    observation_count: 1,
                },
            );
        }
    }

    /// Get a learned pattern with current confidence (after decay)
    pub fn get_pattern(&self, key: &str, now: DateTime<Utc>) -> Option<LearnedPattern> {
        self.patterns.get(key).map(|p| {
            let mut pattern = p.clone();
            pattern.confidence = self.apply_decay(p, now);
            pattern
        })
    }

    /// Get all patterns above confidence threshold
    pub fn get_confident_patterns(&self, threshold: f64, now: DateTime<Utc>) -> Vec<LearnedPattern> {
        self.patterns
            .values()
            .filter_map(|p| {
                let confidence = self.apply_decay(p, now);
                if confidence >= threshold {
                    let mut pattern = p.clone();
                    pattern.confidence = confidence;
                    Some(pattern)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Apply exponential decay to confidence
    fn apply_decay(&self, pattern: &LearnedPattern, now: DateTime<Utc>) -> f64 {
        let time_passed = now.signed_duration_since(pattern.last_reinforced);
        let days_passed = time_passed.num_days() as f64;

        if days_passed <= 0.0 {
            return pattern.confidence;
        }

        // Exponential decay: confidence * (0.5)^(days / half_life)
        let decay_factor = 0.5_f64.powf(days_passed / self.decay_half_life_days as f64);
        (pattern.confidence * decay_factor).max(0.0)
    }

    /// Forget patterns below threshold
    pub fn prune(&mut self, threshold: f64, now: DateTime<Utc>) {
        let decay_half_life = self.decay_half_life_days;
        self.patterns.retain(|_, p| {
            let time_passed = now.signed_duration_since(p.last_reinforced);
            let days_passed = time_passed.num_days() as f64;

            if days_passed <= 0.0 {
                return p.confidence >= threshold;
            }

            let decay_factor = 0.5_f64.powf(days_passed / decay_half_life as f64);
            let current_confidence = (p.confidence * decay_factor).max(0.0);
            current_confidence >= threshold
        });
    }

    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_observation() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.7,
            },
            now,
        );

        let pattern = engine.get_pattern("style", now).unwrap();
        assert_eq!(pattern.value, "Friendly");
        assert!(pattern.confidence >= 0.6 && pattern.confidence <= 0.8);
        assert_eq!(pattern.observation_count, 1);
    }

    #[test]
    fn test_reinforcement_same_value() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        // First observation
        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.6,
            },
            now,
        );

        let initial_confidence = engine.get_pattern("style", now).unwrap().confidence;

        // Reinforce with same value
        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.7,
            },
            now,
        );

        let pattern = engine.get_pattern("style", now).unwrap();
        assert_eq!(pattern.value, "Friendly");
        assert!(pattern.confidence > initial_confidence);
        assert_eq!(pattern.observation_count, 2);
    }

    #[test]
    fn test_reinforcement_different_value() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        // First observation
        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.6,
            },
            now,
        );

        let initial_confidence = engine.get_pattern("style", now).unwrap().confidence;

        // Contradict with different value
        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Professional".to_string(),
                weight: 0.8,
            },
            now,
        );

        let pattern = engine.get_pattern("style", now).unwrap();
        assert!(pattern.confidence < initial_confidence);
    }

    #[test]
    fn test_value_replacement_on_low_confidence() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        // Establish pattern
        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.5,
            },
            now,
        );

        // Contradict multiple times
        for _ in 0..3 {
            engine.observe(
                Observation {
                    key: "style".to_string(),
                    value: "Professional".to_string(),
                    weight: 0.8,
                },
                now,
            );
        }

        let pattern = engine.get_pattern("style", now).unwrap();
        assert_eq!(pattern.value, "Professional");
    }

    #[test]
    fn test_time_decay() {
        let mut engine = LearningEngine::new(10); // 10-day half-life
        let now = Utc::now();

        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.8,
            },
            now,
        );

        let initial = engine.get_pattern("style", now).unwrap().confidence;

        // Check after 10 days (1 half-life)
        let future_10d = now + Duration::days(10);
        let after_10d = engine.get_pattern("style", future_10d).unwrap().confidence;
        assert!((after_10d - initial * 0.5).abs() < 0.05);

        // Check after 20 days (2 half-lives)
        let future_20d = now + Duration::days(20);
        let after_20d = engine.get_pattern("style", future_20d).unwrap().confidence;
        assert!((after_20d - initial * 0.25).abs() < 0.05);
    }

    #[test]
    fn test_get_confident_patterns() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        engine.observe(
            Observation {
                key: "high_conf".to_string(),
                value: "A".to_string(),
                weight: 0.8,
            },
            now,
        );

        engine.observe(
            Observation {
                key: "low_conf".to_string(),
                value: "B".to_string(),
                weight: 0.3,
            },
            now,
        );

        let confident = engine.get_confident_patterns(0.5, now);
        assert_eq!(confident.len(), 1);
        assert_eq!(confident[0].key, "high_conf");
    }

    #[test]
    fn test_pruning() {
        let mut engine = LearningEngine::new(5); // Short half-life
        let now = Utc::now();

        engine.observe(
            Observation {
                key: "old".to_string(),
                value: "X".to_string(),
                weight: 0.6,
            },
            now,
        );

        assert_eq!(engine.pattern_count(), 1);

        // Prune after decay
        let future = now + Duration::days(20);
        engine.prune(0.2, future);
        assert_eq!(engine.pattern_count(), 0);
    }

    #[test]
    fn test_multiple_patterns() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        engine.observe(
            Observation {
                key: "style".to_string(),
                value: "Friendly".to_string(),
                weight: 0.7,
            },
            now,
        );

        engine.observe(
            Observation {
                key: "time_preference".to_string(),
                value: "morning".to_string(),
                weight: 0.8,
            },
            now,
        );

        assert_eq!(engine.pattern_count(), 2);
        assert!(engine.get_pattern("style", now).is_some());
        assert!(engine.get_pattern("time_preference", now).is_some());
    }

    #[test]
    fn test_confidence_bounds() {
        let mut engine = LearningEngine::default();
        let now = Utc::now();

        // Test upper bound
        for _ in 0..10 {
            engine.observe(
                Observation {
                    key: "style".to_string(),
                    value: "Friendly".to_string(),
                    weight: 0.9,
                },
                now,
            );
        }

        let pattern = engine.get_pattern("style", now).unwrap();
        assert!(pattern.confidence <= 0.95);

        // Test lower bound after contradictions
        for _ in 0..10 {
            engine.observe(
                Observation {
                    key: "style".to_string(),
                    value: "Different".to_string(),
                    weight: 0.9,
                },
                now,
            );
        }

        let pattern_after = engine.get_pattern("style", now).unwrap();
        assert!(pattern_after.confidence >= 0.0);
    }
}
