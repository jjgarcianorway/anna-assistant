//! Usage Tracking - v0.25.0
//!
//! Tracks user behavior and system events for relevance scoring.
//! No hardcoded knowledge - all patterns learned from actual usage.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::protocol_v25::{UsageEvent, UsageEventType};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Maximum events to keep in history
pub const DEFAULT_MAX_EVENTS: usize = 10000;

/// Default rolling window for period stats (24 hours)
pub const DEFAULT_PERIOD_WINDOW_SECS: i64 = 86400;

/// Default event batch size before persisting
pub const DEFAULT_BATCH_SIZE: usize = 100;

/// Configuration for usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrackingConfig {
    /// Maximum events in memory
    pub max_events: usize,
    /// Rolling window for period stats (seconds)
    pub period_window_secs: i64,
    /// Batch size before persist
    pub batch_size: usize,
    /// Enable tracking (can be disabled for privacy)
    pub enabled: bool,
    /// Track app launches
    pub track_apps: bool,
    /// Track file access
    pub track_files: bool,
    /// Track commands
    pub track_commands: bool,
    /// Track queries
    pub track_queries: bool,
}

impl Default for UsageTrackingConfig {
    fn default() -> Self {
        Self {
            max_events: DEFAULT_MAX_EVENTS,
            period_window_secs: DEFAULT_PERIOD_WINDOW_SECS,
            batch_size: DEFAULT_BATCH_SIZE,
            enabled: true,
            track_apps: true,
            track_files: true,
            track_commands: true,
            track_queries: true,
        }
    }
}

// ============================================================================
// USAGE TRACKER
// ============================================================================

/// Main usage tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct UsageTracker {
    /// Configuration
    pub config: UsageTrackingConfig,
    /// Event history (most recent first)
    pub events: VecDeque<UsageEvent>,
    /// Aggregated stats per entity
    pub entity_stats: HashMap<String, EntityUsageStats>,
    /// Pattern detector state
    pub patterns: PatternDetector,
    /// Current session ID
    pub current_session: Option<String>,
    /// Pending events (not yet persisted)
    pub pending_count: usize,
}


impl UsageTracker {
    /// Create with config
    pub fn with_config(config: UsageTrackingConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Set current session
    pub fn set_session(&mut self, session_id: String) {
        self.current_session = Some(session_id);
    }

    /// Record a usage event
    pub fn record(&mut self, mut event: UsageEvent) {
        if !self.config.enabled {
            return;
        }

        // Check if event type is tracked
        if !self.should_track(&event.event_type) {
            return;
        }

        // Add session if not set
        if event.session_id.is_none() {
            event.session_id = self.current_session.clone();
        }

        // Update entity stats
        self.entity_stats
            .entry(event.entity.clone())
            .or_default()
            .record(&event);

        // Update pattern detector
        self.patterns.observe(&event);

        // Add to history
        self.events.push_front(event);

        // Trim if needed
        while self.events.len() > self.config.max_events {
            self.events.pop_back();
        }

        self.pending_count += 1;
    }

    /// Check if event type should be tracked
    fn should_track(&self, event_type: &UsageEventType) -> bool {
        match event_type {
            UsageEventType::AppLaunch | UsageEventType::AppFocus | UsageEventType::AppClose => {
                self.config.track_apps
            }
            UsageEventType::FileAccess => self.config.track_files,
            UsageEventType::CommandExec => self.config.track_commands,
            UsageEventType::UserQuery | UsageEventType::AnswerProvided => self.config.track_queries,
            UsageEventType::ProbeSuccess => true, // Always track probes
        }
    }

    /// Check if batch should be persisted
    pub fn should_persist(&self) -> bool {
        self.pending_count >= self.config.batch_size
    }

    /// Mark as persisted
    pub fn mark_persisted(&mut self) {
        self.pending_count = 0;
    }

    /// Get recent events
    pub fn recent_events(&self, limit: usize) -> Vec<&UsageEvent> {
        self.events.iter().take(limit).collect()
    }

    /// Get events for entity
    pub fn events_for_entity(&self, entity: &str, limit: usize) -> Vec<&UsageEvent> {
        self.events
            .iter()
            .filter(|e| e.entity == entity)
            .take(limit)
            .collect()
    }

    /// Get top entities by usage
    pub fn top_entities(&self, limit: usize) -> Vec<(&String, &EntityUsageStats)> {
        let mut sorted: Vec<_> = self.entity_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.total_count.cmp(&a.1.total_count));
        sorted.into_iter().take(limit).collect()
    }

    /// Clean old events outside period window
    pub fn clean_old_events(&mut self) {
        let cutoff = chrono::Utc::now().timestamp() - self.config.period_window_secs;
        while let Some(last) = self.events.back() {
            if last.timestamp < cutoff {
                self.events.pop_back();
            } else {
                break;
            }
        }
    }
}

// ============================================================================
// ENTITY USAGE STATS
// ============================================================================

/// Usage statistics for a single entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityUsageStats {
    /// Total usage count
    pub total_count: u64,
    /// Count in current period
    pub period_count: u64,
    /// Last used timestamp
    pub last_used: Option<i64>,
    /// First used timestamp
    pub first_used: Option<i64>,
    /// Usage by hour of day (0-23)
    pub hourly_distribution: [u32; 24],
    /// Usage by day of week (0-6, 0=Sunday)
    pub daily_distribution: [u32; 7],
    /// Recent timestamps (for pattern detection)
    pub recent_timestamps: VecDeque<i64>,
}

impl EntityUsageStats {
    /// Max recent timestamps to keep
    const MAX_RECENT: usize = 50;

    /// Record a usage event
    pub fn record(&mut self, event: &UsageEvent) {
        self.total_count += 1;
        self.period_count += 1;
        self.last_used = Some(event.timestamp);

        if self.first_used.is_none() {
            self.first_used = Some(event.timestamp);
        }

        // Update distributions
        let dt = chrono::DateTime::from_timestamp(event.timestamp, 0);
        if let Some(dt) = dt {
            let hour = dt.format("%H").to_string().parse::<usize>().unwrap_or(0);
            let weekday = dt.format("%w").to_string().parse::<usize>().unwrap_or(0);
            self.hourly_distribution[hour] += 1;
            self.daily_distribution[weekday] += 1;
        }

        // Track recent timestamps
        self.recent_timestamps.push_front(event.timestamp);
        while self.recent_timestamps.len() > Self::MAX_RECENT {
            self.recent_timestamps.pop_back();
        }
    }

    /// Get peak usage hour
    pub fn peak_hour(&self) -> usize {
        self.hourly_distribution
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(hour, _)| hour)
            .unwrap_or(0)
    }

    /// Get peak usage day
    pub fn peak_day(&self) -> usize {
        self.daily_distribution
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(day, _)| day)
            .unwrap_or(0)
    }

    /// Calculate average interval between uses (seconds)
    pub fn average_interval(&self) -> Option<i64> {
        if self.recent_timestamps.len() < 2 {
            return None;
        }

        let mut intervals: Vec<i64> = Vec::new();
        let timestamps: Vec<_> = self.recent_timestamps.iter().collect();
        for i in 0..timestamps.len() - 1 {
            intervals.push(timestamps[i] - timestamps[i + 1]);
        }

        if intervals.is_empty() {
            return None;
        }

        Some(intervals.iter().sum::<i64>() / intervals.len() as i64)
    }
}

// ============================================================================
// PATTERN DETECTOR
// ============================================================================

/// Detects usage patterns from events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternDetector {
    /// Sequence patterns (A followed by B)
    pub sequences: HashMap<String, SequencePattern>,
    /// Co-occurrence patterns (A and B used together)
    pub cooccurrences: HashMap<String, u32>,
    /// Previous event entity (for sequence detection)
    pub last_entity: Option<String>,
    /// Last event timestamp
    pub last_timestamp: Option<i64>,
}

/// A detected sequence pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequencePattern {
    /// How many times this sequence occurred
    pub count: u32,
    /// Average time between events (seconds)
    pub avg_interval: f64,
}

impl PatternDetector {
    /// Time window for considering events as related (5 minutes)
    const SEQUENCE_WINDOW_SECS: i64 = 300;

    /// Observe an event for pattern detection
    pub fn observe(&mut self, event: &UsageEvent) {
        // Detect sequence patterns
        if let (Some(last), Some(last_ts)) = (&self.last_entity, self.last_timestamp) {
            let interval = event.timestamp - last_ts;
            if interval <= Self::SEQUENCE_WINDOW_SECS {
                // v4.5.5: ASCII only
                let key = format!("{}->{}", last, event.entity);
                let pattern = self.sequences.entry(key).or_default();
                pattern.count += 1;
                pattern.avg_interval = (pattern.avg_interval * (pattern.count - 1) as f64
                    + interval as f64)
                    / pattern.count as f64;
            }
        }

        self.last_entity = Some(event.entity.clone());
        self.last_timestamp = Some(event.timestamp);
    }

    /// Get predicted next entities based on current
    pub fn predict_next(&self, current: &str, limit: usize) -> Vec<(String, f32)> {
        // v4.5.5: ASCII only
        let prefix = format!("{}->", current);
        let mut predictions: Vec<_> = self
            .sequences
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, p)| {
                let next = k.strip_prefix(&prefix).unwrap_or("").to_string();
                (next, p.count as f32)
            })
            .collect();

        // Normalize to probabilities
        let total: f32 = predictions.iter().map(|(_, c)| c).sum();
        if total > 0.0 {
            for (_, c) in &mut predictions {
                *c /= total;
            }
        }

        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predictions.into_iter().take(limit).collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_tracker_basic() {
        let mut tracker = UsageTracker::default();
        tracker.set_session("session-001".to_string());

        let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string());
        tracker.record(event);

        assert_eq!(tracker.events.len(), 1);
        assert!(tracker.entity_stats.contains_key("firefox"));
    }

    #[test]
    fn test_entity_stats() {
        let mut stats = EntityUsageStats::default();
        let event = UsageEvent::new(UsageEventType::AppLaunch, "test".to_string());
        stats.record(&event);

        assert_eq!(stats.total_count, 1);
        assert!(stats.last_used.is_some());
    }

    #[test]
    fn test_pattern_detector() {
        let mut detector = PatternDetector::default();

        let e1 = UsageEvent::new(UsageEventType::AppLaunch, "terminal".to_string());
        let mut e2 = UsageEvent::new(UsageEventType::AppLaunch, "nvim".to_string());
        e2.timestamp = e1.timestamp + 10; // 10 seconds later

        detector.observe(&e1);
        detector.observe(&e2);

        // v4.5.5: ASCII only
        assert!(detector.sequences.contains_key("terminal->nvim"));
    }

    #[test]
    fn test_predict_next() {
        let mut detector = PatternDetector::default();

        // Record pattern: terminal -> nvim (multiple times)
        for i in 0..5 {
            let mut e1 = UsageEvent::new(UsageEventType::AppLaunch, "terminal".to_string());
            e1.timestamp = i * 1000;
            let mut e2 = UsageEvent::new(UsageEventType::AppLaunch, "nvim".to_string());
            e2.timestamp = e1.timestamp + 10;

            detector.observe(&e1);
            detector.observe(&e2);
        }

        let predictions = detector.predict_next("terminal", 5);
        assert!(!predictions.is_empty());
        assert_eq!(predictions[0].0, "nvim");
    }

    #[test]
    fn test_top_entities() {
        let mut tracker = UsageTracker::default();

        for _ in 0..10 {
            let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string());
            tracker.record(event);
        }

        for _ in 0..5 {
            let event = UsageEvent::new(UsageEventType::AppLaunch, "chrome".to_string());
            tracker.record(event);
        }

        let top = tracker.top_entities(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "firefox");
    }
}
