// Learning Engine
// Phase 3.7: Predictive Intelligence
//
// Rule-based pattern detection over persistent context.
// Identifies recurring patterns, anomalies, and user preferences
// to enable proactive system management.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pattern type detected in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Recurring maintenance window (e.g., updates every Sunday)
    MaintenanceWindow,
    /// Repeated command usage (user habits)
    CommandUsage,
    /// Recurring failure (service repeatedly failing)
    RecurringFailure,
    /// Resource trend (disk/memory usage pattern)
    ResourceTrend,
    /// Time-based pattern (peak usage hours)
    TimePattern,
    /// Dependency pattern (service A fails -> service B fails)
    DependencyChain,
}

/// Confidence level of a detected pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Confidence {
    Low,      // 1-2 occurrences
    Medium,   // 3-5 occurrences
    High,     // 6-10 occurrences
    VeryHigh, // 11+ occurrences
}

impl Confidence {
    /// Get confidence from occurrence count
    pub fn from_occurrences(count: usize) -> Self {
        match count {
            0..=2 => Confidence::Low,
            3..=5 => Confidence::Medium,
            6..=10 => Confidence::High,
            _ => Confidence::VeryHigh,
        }
    }

    /// Get percentage (for display)
    pub fn as_percentage(&self) -> u8 {
        match self {
            Confidence::Low => 40,
            Confidence::Medium => 65,
            Confidence::High => 85,
            Confidence::VeryHigh => 95,
        }
    }
}

/// Detected pattern in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Unique pattern ID
    pub id: String,

    /// Pattern type
    pub pattern_type: PatternType,

    /// Pattern description
    pub description: String,

    /// When pattern was first detected
    pub first_seen: DateTime<Utc>,

    /// Last occurrence of this pattern
    pub last_seen: DateTime<Utc>,

    /// Number of times pattern occurred
    pub occurrence_count: usize,

    /// Confidence level
    pub confidence: Confidence,

    /// Pattern-specific metadata
    pub metadata: HashMap<String, String>,

    /// Whether pattern is actionable
    pub actionable: bool,
}

impl DetectedPattern {
    /// Create new pattern
    pub fn new(pattern_type: PatternType, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_type,
            description: description.into(),
            first_seen: now,
            last_seen: now,
            occurrence_count: 1,
            confidence: Confidence::Low,
            metadata: HashMap::new(),
            actionable: false,
        }
    }

    /// Record another occurrence
    pub fn record_occurrence(&mut self) {
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
        self.confidence = Confidence::from_occurrences(self.occurrence_count);
    }

    /// Mark as actionable
    pub fn actionable(mut self) -> Self {
        self.actionable = true;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if pattern is recent (within last 30 days)
    pub fn is_recent(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.last_seen);
        age < Duration::days(30)
    }

    /// Check if pattern is strong enough to act on
    pub fn is_actionable(&self) -> bool {
        self.actionable && self.confidence >= Confidence::Medium && self.is_recent()
    }
}

/// Action history summary for pattern detection
#[derive(Debug)]
pub struct ActionSummary {
    pub action_type: String,
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub avg_duration_ms: u64,
    pub last_execution: DateTime<Utc>,
}

/// Learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total patterns detected
    pub total_patterns: usize,

    /// Actionable patterns
    pub actionable_patterns: usize,

    /// Total actions analyzed
    pub actions_analyzed: usize,

    /// Analysis time range (days)
    pub analysis_days: u64,

    /// Confidence distribution
    pub confidence_distribution: HashMap<String, usize>,

    /// Last learning run
    pub last_run: DateTime<Utc>,
}

impl LearningStats {
    pub fn new() -> Self {
        Self {
            total_patterns: 0,
            actionable_patterns: 0,
            actions_analyzed: 0,
            analysis_days: 30,
            confidence_distribution: HashMap::new(),
            last_run: Utc::now(),
        }
    }
}

impl Default for LearningStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning engine for pattern detection
pub struct LearningEngine {
    /// Detected patterns
    patterns: Vec<DetectedPattern>,

    /// Learning statistics
    stats: LearningStats,

    /// Minimum occurrences to consider a pattern
    min_occurrences: usize,

    /// Analysis window (days)
    analysis_window_days: i64,
}

impl LearningEngine {
    /// Create new learning engine
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            stats: LearningStats::new(),
            min_occurrences: 2,
            analysis_window_days: 30,
        }
    }

    /// Set minimum occurrences threshold
    pub fn with_min_occurrences(mut self, min: usize) -> Self {
        self.min_occurrences = min;
        self
    }

    /// Set analysis window
    pub fn with_analysis_window(mut self, days: i64) -> Self {
        self.analysis_window_days = days;
        self
    }

    /// Detect maintenance window patterns
    ///
    /// Analyzes when updates typically happen to predict future maintenance windows
    pub fn detect_maintenance_patterns(&mut self, actions: &[ActionSummary]) {
        let update_actions: Vec<_> = actions
            .iter()
            .filter(|a| a.action_type == "update" || a.action_type == "system_upgrade")
            .collect();

        if update_actions.len() >= self.min_occurrences {
            // Analyze day-of-week patterns
            let pattern = DetectedPattern::new(
                PatternType::MaintenanceWindow,
                format!(
                    "System updates typically performed {} times in last {} days",
                    update_actions.len(),
                    self.analysis_window_days
                ),
            )
            .actionable()
            .with_metadata("action_type", "update")
            .with_metadata("frequency", format!("{}", update_actions.len()));

            self.patterns.push(pattern);
        }
    }

    /// Detect recurring failures
    ///
    /// Identifies services or actions that repeatedly fail
    pub fn detect_failure_patterns(&mut self, actions: &[ActionSummary]) {
        for action in actions {
            if action.failure_count >= self.min_occurrences {
                let failure_rate =
                    (action.failure_count as f64 / action.total_count as f64) * 100.0;

                if failure_rate > 20.0 {
                    // More than 20% failure rate
                    let mut pattern = DetectedPattern::new(
                        PatternType::RecurringFailure,
                        format!(
                            "Action '{}' fails frequently ({:.1}% failure rate, {} failures)",
                            action.action_type, failure_rate, action.failure_count
                        ),
                    )
                    .with_metadata("action_type", action.action_type.clone())
                    .with_metadata("failure_rate", format!("{:.1}", failure_rate))
                    .with_metadata("failure_count", action.failure_count.to_string());

                    pattern.occurrence_count = action.failure_count;
                    pattern.confidence = Confidence::from_occurrences(action.failure_count);

                    if failure_rate > 50.0 {
                        pattern = pattern.actionable();
                    }

                    self.patterns.push(pattern);
                }
            }
        }
    }

    /// Detect command usage patterns
    ///
    /// Identifies frequently used commands (user habits)
    pub fn detect_usage_patterns(&mut self, actions: &[ActionSummary]) {
        let high_usage: Vec<_> = actions.iter().filter(|a| a.total_count >= 5).collect();

        for action in high_usage {
            let pattern = DetectedPattern::new(
                PatternType::CommandUsage,
                format!(
                    "Frequently used: '{}' ({} times, {:.1}s avg)",
                    action.action_type,
                    action.total_count,
                    action.avg_duration_ms as f64 / 1000.0
                ),
            )
            .with_metadata("action_type", action.action_type.clone())
            .with_metadata("usage_count", action.total_count.to_string())
            .with_metadata("avg_duration_ms", action.avg_duration_ms.to_string());

            self.patterns.push(pattern);
        }
    }

    /// Run full pattern analysis
    ///
    /// Analyzes all available data and detects patterns
    pub fn analyze(&mut self, actions: Vec<ActionSummary>) {
        self.patterns.clear();

        self.detect_maintenance_patterns(&actions);
        self.detect_failure_patterns(&actions);
        self.detect_usage_patterns(&actions);

        // Update statistics
        self.stats.total_patterns = self.patterns.len();
        self.stats.actionable_patterns = self.patterns.iter().filter(|p| p.is_actionable()).count();
        self.stats.actions_analyzed = actions.len();
        self.stats.analysis_days = self.analysis_window_days as u64;
        self.stats.last_run = Utc::now();

        // Confidence distribution
        self.stats.confidence_distribution.clear();
        for pattern in &self.patterns {
            let key = format!("{:?}", pattern.confidence);
            *self.stats.confidence_distribution.entry(key).or_insert(0) += 1;
        }
    }

    /// Get all detected patterns
    pub fn get_patterns(&self) -> &[DetectedPattern] {
        &self.patterns
    }

    /// Get actionable patterns only
    pub fn get_actionable_patterns(&self) -> Vec<&DetectedPattern> {
        self.patterns.iter().filter(|p| p.is_actionable()).collect()
    }

    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: PatternType) -> Vec<&DetectedPattern> {
        self.patterns
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .collect()
    }

    /// Get learning statistics
    pub fn get_stats(&self) -> &LearningStats {
        &self.stats
    }

    /// Clear all patterns
    pub fn clear(&mut self) {
        self.patterns.clear();
        self.stats = LearningStats::new();
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_levels() {
        assert_eq!(Confidence::from_occurrences(1), Confidence::Low);
        assert_eq!(Confidence::from_occurrences(4), Confidence::Medium);
        assert_eq!(Confidence::from_occurrences(8), Confidence::High);
        assert_eq!(Confidence::from_occurrences(15), Confidence::VeryHigh);

        assert_eq!(Confidence::High.as_percentage(), 85);
    }

    #[test]
    fn test_detected_pattern() {
        let mut pattern = DetectedPattern::new(PatternType::RecurringFailure, "Test pattern")
            .actionable()
            .with_metadata("key", "value");

        assert_eq!(pattern.occurrence_count, 1);
        assert_eq!(pattern.confidence, Confidence::Low);

        pattern.record_occurrence();
        pattern.record_occurrence();
        pattern.record_occurrence();

        assert_eq!(pattern.occurrence_count, 4);
        assert_eq!(pattern.confidence, Confidence::Medium);
        assert!(pattern.is_actionable());
    }

    #[test]
    fn test_learning_engine() {
        let mut engine = LearningEngine::new().with_min_occurrences(2);

        let actions = vec![
            ActionSummary {
                action_type: "update".to_string(),
                total_count: 10,
                success_count: 9,
                failure_count: 1,
                avg_duration_ms: 45000,
                last_execution: Utc::now(),
            },
            ActionSummary {
                action_type: "health_check".to_string(),
                total_count: 20,
                success_count: 15,
                failure_count: 5,
                avg_duration_ms: 2000,
                last_execution: Utc::now(),
            },
        ];

        engine.analyze(actions);

        let patterns = engine.get_patterns();
        assert!(!patterns.is_empty());

        let stats = engine.get_stats();
        assert_eq!(stats.actions_analyzed, 2);
        assert!(stats.total_patterns > 0);
    }

    #[test]
    fn test_failure_detection() {
        let mut engine = LearningEngine::new().with_min_occurrences(2);

        let actions = vec![ActionSummary {
            action_type: "flaky_service".to_string(),
            total_count: 10,
            success_count: 3,
            failure_count: 7,
            avg_duration_ms: 1000,
            last_execution: Utc::now(),
        }];

        engine.analyze(actions);

        let failure_patterns = engine.get_patterns_by_type(PatternType::RecurringFailure);
        assert!(!failure_patterns.is_empty());

        let pattern = failure_patterns[0];
        assert!(pattern.description.contains("fails frequently"));
        assert!(pattern.is_actionable());
    }

    #[test]
    fn test_usage_patterns() {
        let mut engine = LearningEngine::new();

        let actions = vec![ActionSummary {
            action_type: "status".to_string(),
            total_count: 50,
            success_count: 50,
            failure_count: 0,
            avg_duration_ms: 500,
            last_execution: Utc::now(),
        }];

        engine.analyze(actions);

        let usage_patterns = engine.get_patterns_by_type(PatternType::CommandUsage);
        assert_eq!(usage_patterns.len(), 1);
        assert!(usage_patterns[0].description.contains("Frequently used"));
    }
}
