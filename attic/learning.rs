//! Learning Cache - Passive intelligence tracking action outcomes
//!
//! Sprint 3: Intelligence, Policies & Event Reactions
//!
//! Records outcomes of self-healing and policy reactions to build a success/failure
//! history. Used to adjust retry priorities and action selection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Learning system errors
#[derive(Debug, Error)]
pub enum LearningError {
    #[error("Failed to load learning data: {0}")]
    LoadError(String),

    #[error("Failed to save learning data: {0}")]
    SaveError(String),

    #[error("Action not found: {0}")]
    #[allow(dead_code)]
    ActionNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Outcome of an action execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Success,
    Failure,
    Partial,
}

/// Single outcome record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub timestamp: u64,
    pub outcome: Outcome,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub context: HashMap<String, String>,
}

#[allow(dead_code)]
impl OutcomeRecord {
    pub fn new(outcome: Outcome) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            outcome,
            duration_ms: None,
            error_message: None,
            context: HashMap::new(),
        }
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Action statistics tracking success/failure rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStats {
    pub action_name: String,
    pub total_executions: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub partial_count: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub last_execution: u64,
    pub recent_outcomes: Vec<OutcomeRecord>,
}

#[allow(dead_code)]
impl ActionStats {
    pub fn new(action_name: impl Into<String>) -> Self {
        Self {
            action_name: action_name.into(),
            total_executions: 0,
            success_count: 0,
            failure_count: 0,
            partial_count: 0,
            success_rate: 0.0,
            avg_duration_ms: 0.0,
            last_execution: 0,
            recent_outcomes: Vec::new(),
        }
    }

    /// Record a new outcome
    pub fn record_outcome(&mut self, record: OutcomeRecord) {
        self.total_executions += 1;
        self.last_execution = record.timestamp;

        match record.outcome {
            Outcome::Success => self.success_count += 1,
            Outcome::Failure => self.failure_count += 1,
            Outcome::Partial => self.partial_count += 1,
        }

        // Update success rate
        self.success_rate = if self.total_executions > 0 {
            (self.success_count as f64) / (self.total_executions as f64)
        } else {
            0.0
        };

        // Update average duration
        if let Some(duration) = record.duration_ms {
            let total_duration = self.avg_duration_ms * ((self.total_executions - 1) as f64);
            self.avg_duration_ms = (total_duration + duration as f64) / (self.total_executions as f64);
        }

        // Keep last 100 outcomes
        self.recent_outcomes.push(record);
        if self.recent_outcomes.len() > 100 {
            self.recent_outcomes.remove(0);
        }
    }

    /// Get priority score (higher = better candidate for retry)
    pub fn priority_score(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.5; // Neutral for untested actions
        }

        // Weight recent outcomes more heavily
        let recent_success_rate = if self.recent_outcomes.len() >= 10 {
            let recent_successes = self.recent_outcomes
                .iter()
                .rev()
                .take(10)
                .filter(|r| r.outcome == Outcome::Success)
                .count();
            recent_successes as f64 / 10.0
        } else {
            self.success_rate
        };

        // Factor in execution count (more data = more confidence)
        let confidence = (self.total_executions as f64 / 100.0).min(1.0);

        // Combine factors: 70% recent success rate, 30% confidence
        recent_success_rate * 0.7 + confidence * 0.3
    }

    /// Check if action should be retried based on history
    pub fn should_retry(&self, max_consecutive_failures: usize) -> bool {
        if self.recent_outcomes.is_empty() {
            return true; // Try at least once
        }

        // Check consecutive failures
        let mut consecutive_failures = 0;
        for outcome in self.recent_outcomes.iter().rev() {
            if outcome.outcome == Outcome::Failure {
                consecutive_failures += 1;
                if consecutive_failures >= max_consecutive_failures {
                    return false;
                }
            } else {
                break;
            }
        }

        true
    }
}

/// Learning cache storage
#[derive(Debug, Serialize, Deserialize)]
struct LearningData {
    version: String,
    actions: HashMap<String, ActionStats>,
}

/// Learning cache - tracks action outcomes and provides intelligence
pub struct LearningCache {
    data: Arc<RwLock<LearningData>>,
    storage_path: PathBuf,
    auto_save: bool,
}

impl LearningCache {
    /// Create a new learning cache
    pub fn new(storage_path: impl AsRef<Path>) -> Self {
        Self {
            data: Arc::new(RwLock::new(LearningData {
                version: "1.0".to_string(),
                actions: HashMap::new(),
            })),
            storage_path: storage_path.as_ref().to_path_buf(),
            auto_save: true,
        }
    }

    /// Load learning data from disk
    pub fn load(&self) -> Result<(), LearningError> {
        if !self.storage_path.exists() {
            // No existing data, start fresh
            return Ok(());
        }

        let contents = fs::read_to_string(&self.storage_path)
            .map_err(|e| LearningError::LoadError(e.to_string()))?;

        let loaded: LearningData = serde_json::from_str(&contents)?;

        let mut data = self.data.write().unwrap();
        *data = loaded;

        Ok(())
    }

    /// Save learning data to disk
    pub fn save(&self) -> Result<(), LearningError> {
        let data = self.data.read().unwrap();
        let json = serde_json::to_string_pretty(&*data)?;

        // Ensure parent directory exists
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| LearningError::SaveError(e.to_string()))?;
        }

        fs::write(&self.storage_path, json)
            .map_err(|e| LearningError::SaveError(e.to_string()))?;

        Ok(())
    }

    /// Record an action outcome
    #[allow(dead_code)]
    pub fn record_action(
        &self,
        action_name: impl Into<String>,
        outcome: OutcomeRecord,
    ) -> Result<(), LearningError> {
        let action_name = action_name.into();
        let mut data = self.data.write().unwrap();

        let stats = data.actions
            .entry(action_name.clone())
            .or_insert_with(|| ActionStats::new(&action_name));

        stats.record_outcome(outcome);

        drop(data);

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Get statistics for an action
    pub fn get_stats(&self, action_name: &str) -> Option<ActionStats> {
        let data = self.data.read().unwrap();
        data.actions.get(action_name).cloned()
    }

    /// Get all action statistics
    pub fn get_all_stats(&self) -> Vec<ActionStats> {
        let data = self.data.read().unwrap();
        data.actions.values().cloned().collect()
    }

    /// Get recommended actions sorted by priority
    pub fn get_recommended_actions(&self) -> Vec<(String, f64)> {
        let data = self.data.read().unwrap();
        let mut actions: Vec<_> = data.actions
            .iter()
            .map(|(name, stats)| (name.clone(), stats.priority_score()))
            .collect();

        actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        actions
    }

    /// Check if action should be attempted based on history
    #[allow(dead_code)]
    pub fn should_attempt(&self, action_name: &str, max_consecutive_failures: usize) -> bool {
        let data = self.data.read().unwrap();
        if let Some(stats) = data.actions.get(action_name) {
            stats.should_retry(max_consecutive_failures)
        } else {
            true // No history, try it
        }
    }

    /// Clear all learning data
    pub fn clear(&self) -> Result<(), LearningError> {
        let mut data = self.data.write().unwrap();
        data.actions.clear();
        drop(data);

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Get total number of tracked actions
    pub fn action_count(&self) -> usize {
        let data = self.data.read().unwrap();
        data.actions.len()
    }

    /// Get total number of recorded outcomes
    pub fn total_outcomes(&self) -> usize {
        let data = self.data.read().unwrap();
        data.actions.values().map(|s| s.total_executions).sum()
    }

    /// Get global success rate
    pub fn global_success_rate(&self) -> f64 {
        let data = self.data.read().unwrap();
        let total: usize = data.actions.values().map(|s| s.total_executions).sum();
        let successes: usize = data.actions.values().map(|s| s.success_count).sum();

        if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Learning analytics and insights
#[allow(dead_code)]
pub struct LearningAnalytics {
    cache: Arc<LearningCache>,
}

#[allow(dead_code)]
impl LearningAnalytics {
    pub fn new(cache: Arc<LearningCache>) -> Self {
        Self { cache }
    }

    /// Get top performing actions
    pub fn top_performers(&self, limit: usize) -> Vec<ActionStats> {
        let all_stats = self.cache.get_all_stats();
        let mut stats: Vec<_> = all_stats
            .into_iter()
            .filter(|s| s.total_executions >= 5) // Minimum sample size
            .collect();

        stats.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        stats.truncate(limit);
        stats
    }

    /// Get worst performing actions
    pub fn worst_performers(&self, limit: usize) -> Vec<ActionStats> {
        let all_stats = self.cache.get_all_stats();
        let mut stats: Vec<_> = all_stats
            .into_iter()
            .filter(|s| s.total_executions >= 5)
            .collect();

        stats.sort_by(|a, b| a.success_rate.partial_cmp(&b.success_rate).unwrap());
        stats.truncate(limit);
        stats
    }

    /// Get recently executed actions
    pub fn recent_activity(&self, limit: usize) -> Vec<ActionStats> {
        let all_stats = self.cache.get_all_stats();
        let mut stats = all_stats;

        stats.sort_by(|a, b| b.last_execution.cmp(&a.last_execution));
        stats.truncate(limit);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_outcome_recording() {
        let mut stats = ActionStats::new("test_action");

        let record = OutcomeRecord::new(Outcome::Success)
            .with_duration(100);

        stats.record_outcome(record);

        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.success_rate, 1.0);
        assert_eq!(stats.avg_duration_ms, 100.0);
    }

    #[test]
    fn test_priority_score() {
        let mut stats = ActionStats::new("test_action");

        // Record successes
        for _ in 0..8 {
            stats.record_outcome(OutcomeRecord::new(Outcome::Success));
        }

        // Record failures
        for _ in 0..2 {
            stats.record_outcome(OutcomeRecord::new(Outcome::Failure));
        }

        assert_eq!(stats.success_rate, 0.8);
        assert!(stats.priority_score() > 0.5);
    }

    #[test]
    fn test_should_retry() {
        let mut stats = ActionStats::new("test_action");

        // Record consecutive failures
        for _ in 0..5 {
            stats.record_outcome(OutcomeRecord::new(Outcome::Failure));
        }

        assert!(!stats.should_retry(3));
        assert!(!stats.should_retry(5));
        assert!(stats.should_retry(10));
    }

    #[test]
    fn test_learning_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("learning.json");

        {
            let cache = LearningCache::new(&cache_path);
            cache.record_action("action1", OutcomeRecord::new(Outcome::Success)).unwrap();
            cache.record_action("action2", OutcomeRecord::new(Outcome::Failure)).unwrap();
            cache.save().unwrap();
        }

        // Load in new instance
        let cache = LearningCache::new(&cache_path);
        cache.load().unwrap();

        assert_eq!(cache.action_count(), 2);
        assert!(cache.get_stats("action1").is_some());
        assert!(cache.get_stats("action2").is_some());
    }

    #[test]
    fn test_recommended_actions() {
        let temp_dir = TempDir::new().unwrap();
        let cache = LearningCache::new(temp_dir.path().join("learning.json"));

        // Action with high success
        for _ in 0..10 {
            cache.record_action("good_action", OutcomeRecord::new(Outcome::Success)).unwrap();
        }

        // Action with low success
        for _ in 0..10 {
            cache.record_action("bad_action", OutcomeRecord::new(Outcome::Failure)).unwrap();
        }

        let recommended = cache.get_recommended_actions();
        assert_eq!(recommended[0].0, "good_action");
        assert!(recommended[0].1 > recommended[1].1);
    }

    #[test]
    fn test_analytics() {
        let temp_dir = TempDir::new().unwrap();
        let cache = Arc::new(LearningCache::new(temp_dir.path().join("learning.json")));

        // Create some test data
        for _ in 0..10 {
            cache.record_action("success_action", OutcomeRecord::new(Outcome::Success)).unwrap();
        }

        for i in 0..5 {
            let outcome = if i < 2 { Outcome::Success } else { Outcome::Failure };
            cache.record_action("mixed_action", OutcomeRecord::new(outcome)).unwrap();
        }

        let analytics = LearningAnalytics::new(cache);
        let top = analytics.top_performers(5);

        assert!(!top.is_empty());
        assert_eq!(top[0].action_name, "success_action");
    }
}
