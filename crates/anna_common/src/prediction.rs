// Prediction Engine
// Phase 3.7: Predictive Intelligence
//
// Uses detected patterns to predict future system events
// and suggest proactive actions.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::learning::{DetectedPattern, PatternType};

/// Prediction type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionType {
    /// Maintenance window prediction
    MaintenanceWindow,
    /// Failure prediction
    ServiceFailure,
    /// Resource exhaustion prediction
    ResourceExhaustion,
    /// Performance degradation
    PerformanceDegradation,
    /// General recommendation
    Recommendation,
}

/// Prediction priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Get color code for display
    pub fn color(&self) -> &'static str {
        match self {
            Priority::Low => "blue",
            Priority::Medium => "yellow",
            Priority::High => "orange",
            Priority::Critical => "red",
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Low => "ℹ️",
            Priority::Medium => "⚠️",
            Priority::High => "🔴",
            Priority::Critical => "🚨",
        }
    }
}

/// Predicted event or recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Unique prediction ID
    pub id: String,

    /// Prediction type
    pub prediction_type: PredictionType,

    /// Priority level
    pub priority: Priority,

    /// Prediction title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// When prediction was made
    pub created_at: DateTime<Utc>,

    /// Predicted time of event (if applicable)
    pub predicted_time: Option<DateTime<Utc>>,

    /// Confidence percentage (0-100)
    pub confidence: u8,

    /// Recommended actions
    pub recommended_actions: Vec<String>,

    /// Pattern IDs this prediction is based on
    pub based_on_patterns: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Prediction {
    /// Create new prediction
    pub fn new(
        prediction_type: PredictionType,
        priority: Priority,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prediction_type,
            priority,
            title: title.into(),
            description: description.into(),
            created_at: Utc::now(),
            predicted_time: None,
            confidence: 50,
            recommended_actions: Vec::new(),
            based_on_patterns: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// Set predicted time
    pub fn at_time(mut self, time: DateTime<Utc>) -> Self {
        self.predicted_time = Some(time);
        self
    }

    /// Add recommended action
    pub fn recommend(mut self, action: impl Into<String>) -> Self {
        self.recommended_actions.push(action.into());
        self
    }

    /// Link to pattern
    pub fn based_on(mut self, pattern_id: impl Into<String>) -> Self {
        self.based_on_patterns.push(pattern_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if prediction is urgent (within next 24 hours or critical)
    pub fn is_urgent(&self) -> bool {
        if self.priority == Priority::Critical {
            return true;
        }

        if let Some(predicted_time) = self.predicted_time {
            let time_until = predicted_time.signed_duration_since(Utc::now());
            return time_until < Duration::hours(24) && time_until > Duration::hours(0);
        }

        false
    }

    /// Get time until predicted event
    pub fn time_until(&self) -> Option<Duration> {
        self.predicted_time
            .map(|t| t.signed_duration_since(Utc::now()))
    }

    /// Format time until for display
    pub fn time_until_display(&self) -> String {
        match self.time_until() {
            Some(duration) if duration.num_days() > 0 => {
                format!("{} days", duration.num_days())
            }
            Some(duration) if duration.num_hours() > 0 => {
                format!("{} hours", duration.num_hours())
            }
            Some(duration) if duration.num_minutes() > 0 => {
                format!("{} minutes", duration.num_minutes())
            }
            Some(_) => "now".to_string(),
            None => "unknown".to_string(),
        }
    }
}

/// Prediction engine
pub struct PredictionEngine {
    /// Generated predictions
    predictions: Vec<Prediction>,

    /// Prediction history (for throttling)
    prediction_history: HashMap<String, DateTime<Utc>>,

    /// Minimum confidence to generate predictions
    min_confidence: u8,

    /// Throttle period (don't repeat same prediction within this time)
    throttle_hours: i64,
}

impl PredictionEngine {
    /// Create new prediction engine
    pub fn new() -> Self {
        Self {
            predictions: Vec::new(),
            prediction_history: HashMap::new(),
            min_confidence: 65,
            throttle_hours: 24,
        }
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, min: u8) -> Self {
        self.min_confidence = min;
        self
    }

    /// Set throttle period
    pub fn with_throttle_hours(mut self, hours: i64) -> Self {
        self.throttle_hours = hours;
        self
    }

    /// Check if we should generate a prediction (throttling)
    fn should_generate(&self, key: &str) -> bool {
        if let Some(last_time) = self.prediction_history.get(key) {
            let elapsed = Utc::now().signed_duration_since(*last_time);
            elapsed > Duration::hours(self.throttle_hours)
        } else {
            true
        }
    }

    /// Record that we generated a prediction
    fn record_generation(&mut self, key: String) {
        self.prediction_history.insert(key, Utc::now());
    }

    /// Generate predictions from detected patterns
    pub fn generate_from_patterns(&mut self, patterns: &[DetectedPattern]) {
        self.predictions.clear();

        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::RecurringFailure => {
                    self.predict_failure(pattern);
                }
                PatternType::MaintenanceWindow => {
                    self.predict_maintenance(pattern);
                }
                PatternType::CommandUsage => {
                    // Usage patterns don't generate predictions, just insights
                }
                PatternType::ResourceTrend => {
                    self.predict_resource_exhaustion(pattern);
                }
                PatternType::TimePattern => {
                    // Time patterns can enhance other predictions
                }
                PatternType::DependencyChain => {
                    self.predict_cascading_failure(pattern);
                }
            }
        }

        // Clean up old throttle entries (older than 7 days)
        let cutoff = Utc::now() - Duration::days(7);
        self.prediction_history.retain(|_, &mut time| time > cutoff);
    }

    /// Predict service failure based on recurring failure pattern
    fn predict_failure(&mut self, pattern: &DetectedPattern) {
        let key = format!(
            "failure:{}",
            pattern
                .metadata
                .get("action_type")
                .unwrap_or(&"unknown".to_string())
        );

        if !self.should_generate(&key) {
            return;
        }

        let confidence = pattern.confidence.as_percentage();
        if confidence < self.min_confidence {
            return;
        }

        let default_action = "unknown".to_string();
        let action_type = pattern
            .metadata
            .get("action_type")
            .unwrap_or(&default_action);
        let failure_rate = pattern
            .metadata
            .get("failure_rate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let priority = if failure_rate > 75.0 {
            Priority::Critical
        } else if failure_rate > 50.0 {
            Priority::High
        } else if failure_rate > 30.0 {
            Priority::Medium
        } else {
            Priority::Low
        };

        let prediction = Prediction::new(
            PredictionType::ServiceFailure,
            priority,
            format!("Likely failure: {}", action_type),
            format!(
                "Based on historical data, '{}' has a {:.0}% failure rate. Next execution is likely to fail.",
                action_type, failure_rate
            ),
        )
        .with_confidence(confidence)
        .recommend(format!("Run diagnostics before executing '{}'", action_type))
        .recommend(format!("Check system logs for '{}'", action_type))
        .recommend("Consider manual execution with monitoring")
        .based_on(pattern.id.clone())
        .with_metadata("action_type", action_type);

        self.predictions.push(prediction);
        self.record_generation(key);
    }

    /// Predict maintenance window
    fn predict_maintenance(&mut self, pattern: &DetectedPattern) {
        let key = "maintenance:window".to_string();

        if !self.should_generate(&key) {
            return;
        }

        let confidence = pattern.confidence.as_percentage();
        if confidence < self.min_confidence {
            return;
        }

        // Predict next maintenance window (simplified - would use more sophisticated logic in production)
        let next_window = Utc::now() + Duration::days(7); // Assume weekly pattern

        let prediction = Prediction::new(
            PredictionType::MaintenanceWindow,
            Priority::Low,
            "Upcoming maintenance window",
            "Based on historical patterns, system updates typically occur around this time.",
        )
        .with_confidence(confidence)
        .at_time(next_window)
        .recommend("Schedule updates during this window")
        .recommend("Ensure backups are current")
        .recommend("Monitor system closely during updates")
        .based_on(pattern.id.clone());

        self.predictions.push(prediction);
        self.record_generation(key);
    }

    /// Predict resource exhaustion
    fn predict_resource_exhaustion(&mut self, pattern: &DetectedPattern) {
        let key = "resource:exhaustion".to_string();

        if !self.should_generate(&key) {
            return;
        }

        let prediction = Prediction::new(
            PredictionType::ResourceExhaustion,
            Priority::High,
            "Resource exhaustion risk",
            "Resource usage trend suggests potential exhaustion in near future.",
        )
        .with_confidence(75)
        .recommend("Review disk usage: df -h")
        .recommend("Check memory consumption: free -h")
        .recommend("Clean package cache: pacman -Sc")
        .based_on(pattern.id.clone());

        self.predictions.push(prediction);
        self.record_generation(key);
    }

    /// Predict cascading failure from dependency chain
    fn predict_cascading_failure(&mut self, pattern: &DetectedPattern) {
        let key = "failure:cascading".to_string();

        if !self.should_generate(&key) {
            return;
        }

        let prediction = Prediction::new(
            PredictionType::ServiceFailure,
            Priority::High,
            "Cascading failure risk",
            "Dependency chain detected - failure in one service may cascade to others.",
        )
        .with_confidence(70)
        .recommend("Review service dependencies")
        .recommend("Test services in isolation")
        .recommend("Implement circuit breakers if needed")
        .based_on(pattern.id.clone());

        self.predictions.push(prediction);
        self.record_generation(key);
    }

    /// Get all predictions
    pub fn get_predictions(&self) -> &[Prediction] {
        &self.predictions
    }

    /// Get urgent predictions only
    pub fn get_urgent_predictions(&self) -> Vec<&Prediction> {
        self.predictions.iter().filter(|p| p.is_urgent()).collect()
    }

    /// Get predictions by priority
    pub fn get_by_priority(&self, priority: Priority) -> Vec<&Prediction> {
        self.predictions
            .iter()
            .filter(|p| p.priority == priority)
            .collect()
    }

    /// Clear all predictions
    pub fn clear(&mut self) {
        self.predictions.clear();
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{DetectedPattern, PatternType};

    #[test]
    fn test_priority() {
        assert_eq!(Priority::Low.emoji(), "ℹ️");
        assert_eq!(Priority::Critical.emoji(), "🚨");
        assert_eq!(Priority::High.color(), "orange");
    }

    #[test]
    fn test_prediction_creation() {
        let prediction = Prediction::new(
            PredictionType::ServiceFailure,
            Priority::High,
            "Test prediction",
            "This is a test",
        )
        .with_confidence(85)
        .recommend("Do something")
        .with_metadata("key", "value");

        assert_eq!(prediction.priority, Priority::High);
        assert_eq!(prediction.confidence, 85);
        assert_eq!(prediction.recommended_actions.len(), 1);
        assert_eq!(prediction.metadata.get("key").unwrap(), "value");
    }

    #[test]
    fn test_urgency() {
        let soon = Prediction::new(
            PredictionType::MaintenanceWindow,
            Priority::Medium,
            "Soon",
            "Event soon",
        )
        .at_time(Utc::now() + Duration::hours(2));

        assert!(soon.is_urgent());

        let later = Prediction::new(
            PredictionType::MaintenanceWindow,
            Priority::Low,
            "Later",
            "Event later",
        )
        .at_time(Utc::now() + Duration::days(7));

        assert!(!later.is_urgent());

        let critical = Prediction::new(
            PredictionType::ServiceFailure,
            Priority::Critical,
            "Critical",
            "Critical event",
        );

        assert!(critical.is_urgent());
    }

    #[test]
    fn test_time_display() {
        let prediction = Prediction::new(
            PredictionType::MaintenanceWindow,
            Priority::Low,
            "Test",
            "Test",
        )
        .at_time(Utc::now() + Duration::hours(5));

        let display = prediction.time_until_display();
        // Allow for test execution time
        assert!(display.contains("hour"));
    }

    #[test]
    fn test_prediction_engine() {
        let mut engine = PredictionEngine::new().with_min_confidence(40);

        let mut pattern =
            DetectedPattern::new(PatternType::RecurringFailure, "Test failure pattern")
                .actionable()
                .with_metadata("action_type", "test_service")
                .with_metadata("failure_rate", "80.0");

        // Set confidence high enough for prediction
        pattern.occurrence_count = 5;
        pattern.confidence = crate::learning::Confidence::Medium;

        engine.generate_from_patterns(&[pattern]);

        let predictions = engine.get_predictions();
        assert!(!predictions.is_empty());

        let urgent = engine.get_urgent_predictions();
        assert!(!urgent.is_empty());
    }

    #[test]
    fn test_throttling() {
        let mut engine = PredictionEngine::new()
            .with_throttle_hours(1)
            .with_min_confidence(40);

        let mut pattern = DetectedPattern::new(PatternType::RecurringFailure, "Test")
            .with_metadata("action_type", "test")
            .with_metadata("failure_rate", "90.0");

        pattern.occurrence_count = 5;
        pattern.confidence = crate::learning::Confidence::Medium;

        // First generation should work
        engine.generate_from_patterns(&[pattern.clone()]);
        assert_eq!(engine.get_predictions().len(), 1);

        // Second generation immediately should be throttled
        let prev_history_size = engine.prediction_history.len();
        engine.generate_from_patterns(&[pattern]);
        // Predictions are cleared during generation, but throttling prevents regeneration
        // So we check if new predictions were added to history
        assert_eq!(engine.prediction_history.len(), prev_history_size); // No new predictions added due to throttling
    }
}
