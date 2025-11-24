//! v6.30.0: Meta Insight Telemetry
//!
//! Tracks how often insights are triggered, shown, and resolved to enable
//! Anna's self-optimization without user configuration.
//!
//! ## Purpose
//!
//! - Record insight emission patterns
//! - Track resolution timing (when conditions clear)
//! - Enable detection of "noisy" vs "high-value" insights
//! - Support deterministic self-tuning behavior
//!
//! ## Design Principles
//!
//! 1. **Fail-Safe**: DB errors never break status/query handling
//! 2. **Cheap**: Minimal overhead on insight generation
//! 3. **Deterministic**: Pure rules-based, no randomness
//! 4. **Privacy-Safe**: No sensitive data, just counts and timestamps

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::insights_engine::InsightSeverity;

/// Metadata about how an insight kind behaves over time
///
/// Used to detect noisy insights (frequent, never resolved, user ignores)
/// vs high-value insights (accurate predictions, quick resolutions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightMetaStats {
    /// Insight kind identifier (e.g. "disk_space", "boot_regression")
    pub insight_kind: String,

    /// Severity level (Info, Warning, Critical)
    pub severity: InsightSeverity,

    /// How many times this insight was triggered (detected)
    #[serde(default)]
    pub trigger_count: u64,

    /// How many times this insight was suppressed (detected but not shown)
    #[serde(default)]
    pub suppressed_count: u64,

    /// Last time the underlying condition was detected
    #[serde(default)]
    pub last_triggered_at: Option<DateTime<Utc>>,

    /// Last time this insight was actually shown to the user
    #[serde(default)]
    pub last_shown_at: Option<DateTime<Utc>>,

    /// When the underlying condition was resolved (cleared)
    #[serde(default)]
    pub last_resolved_at: Option<DateTime<Utc>>,
}

impl InsightMetaStats {
    /// Create new meta stats for an insight kind
    pub fn new(insight_kind: impl Into<String>, severity: InsightSeverity) -> Self {
        Self {
            insight_kind: insight_kind.into(),
            severity,
            trigger_count: 0,
            suppressed_count: 0,
            last_triggered_at: None,
            last_shown_at: None,
            last_resolved_at: None,
        }
    }

    /// Record that this insight was triggered (detected)
    pub fn record_trigger(&mut self, shown: bool, now: DateTime<Utc>) {
        self.trigger_count += 1;
        self.last_triggered_at = Some(now);

        if shown {
            self.last_shown_at = Some(now);
        } else {
            self.suppressed_count += 1;
        }
    }

    /// Record that the underlying condition was resolved
    pub fn record_resolution(&mut self, now: DateTime<Utc>) {
        self.last_resolved_at = Some(now);
    }

    /// Check if this insight is currently unresolved
    pub fn is_unresolved(&self) -> bool {
        match (self.last_triggered_at, self.last_resolved_at) {
            (Some(triggered), Some(resolved)) => triggered > resolved,
            (Some(_), None) => true,
            _ => false,
        }
    }

    /// Check if this insight was recently triggered (within days)
    pub fn triggered_within_days(&self, days: i64, now: DateTime<Utc>) -> bool {
        if let Some(last_trigger) = self.last_triggered_at {
            let age_days = (now - last_trigger).num_days();
            age_days <= days
        } else {
            false
        }
    }

    /// Check if this insight has been unresolved for a long time
    pub fn unresolved_for_days(&self, days: i64, now: DateTime<Utc>) -> bool {
        if !self.is_unresolved() {
            return false;
        }

        if let Some(last_trigger) = self.last_triggered_at {
            let age_days = (now - last_trigger).num_days();
            age_days >= days
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_meta_stats() {
        let stats = InsightMetaStats::new("disk_space", InsightSeverity::Warning);

        assert_eq!(stats.insight_kind, "disk_space");
        assert_eq!(stats.severity, InsightSeverity::Warning);
        assert_eq!(stats.trigger_count, 0);
        assert_eq!(stats.suppressed_count, 0);
        assert!(stats.last_triggered_at.is_none());
        assert!(stats.last_shown_at.is_none());
        assert!(stats.last_resolved_at.is_none());
    }

    #[test]
    fn test_record_trigger_shown() {
        let mut stats = InsightMetaStats::new("boot_regression", InsightSeverity::Info);
        let now = Utc::now();

        stats.record_trigger(true, now);

        assert_eq!(stats.trigger_count, 1);
        assert_eq!(stats.suppressed_count, 0);
        assert_eq!(stats.last_triggered_at, Some(now));
        assert_eq!(stats.last_shown_at, Some(now));
    }

    #[test]
    fn test_record_trigger_suppressed() {
        let mut stats = InsightMetaStats::new("network_latency", InsightSeverity::Info);
        let now = Utc::now();

        stats.record_trigger(false, now);

        assert_eq!(stats.trigger_count, 1);
        assert_eq!(stats.suppressed_count, 1);
        assert_eq!(stats.last_triggered_at, Some(now));
        assert!(stats.last_shown_at.is_none());
    }

    #[test]
    fn test_is_unresolved() {
        let mut stats = InsightMetaStats::new("cpu_pressure", InsightSeverity::Warning);
        let now = Utc::now();

        // No triggers yet
        assert!(!stats.is_unresolved());

        // Triggered but not resolved
        stats.record_trigger(true, now);
        assert!(stats.is_unresolved());

        // Resolved
        stats.record_resolution(now);
        assert!(!stats.is_unresolved());
    }

    #[test]
    fn test_triggered_within_days() {
        let mut stats = InsightMetaStats::new("disk_space", InsightSeverity::Warning);
        let now = Utc::now();
        let five_days_ago = now - chrono::Duration::days(5);

        stats.record_trigger(true, five_days_ago);

        assert!(stats.triggered_within_days(7, now));
        assert!(!stats.triggered_within_days(3, now));
    }

    #[test]
    fn test_unresolved_for_days() {
        let mut stats = InsightMetaStats::new("disk_space", InsightSeverity::Warning);
        let now = Utc::now();
        let ten_days_ago = now - chrono::Duration::days(10);

        // Not triggered yet
        assert!(!stats.unresolved_for_days(7, now));

        // Triggered 10 days ago, never resolved
        stats.record_trigger(true, ten_days_ago);
        assert!(stats.unresolved_for_days(7, now));
        assert!(stats.unresolved_for_days(9, now));
        assert!(!stats.unresolved_for_days(11, now));

        // Now resolved
        stats.record_resolution(now);
        assert!(!stats.unresolved_for_days(7, now));
    }
}
