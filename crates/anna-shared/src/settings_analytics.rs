// v0.0.577: Settings Analytics (Phase 153)
// Track settings usage patterns and provide insights

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Analytics time period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalyticsPeriod {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// All time
    AllTime,
}

impl std::fmt::Display for AnalyticsPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Day => write!(f, "Last 24 Hours"),
            Self::Week => write!(f, "Last 7 Days"),
            Self::Month => write!(f, "Last 30 Days"),
            Self::AllTime => write!(f, "All Time"),
        }
    }
}

/// Analytics metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Change count
    ChangeCount,
    /// Access count
    AccessCount,
    /// Revert count
    RevertCount,
    /// Export count
    ExportCount,
    /// Import count
    ImportCount,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeCount => write!(f, "Changes"),
            Self::AccessCount => write!(f, "Accesses"),
            Self::RevertCount => write!(f, "Reverts"),
            Self::ExportCount => write!(f, "Exports"),
            Self::ImportCount => write!(f, "Imports"),
        }
    }
}

/// Single analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub metric: MetricType,
    /// Category (if applicable)
    pub category: Option<SettingsCategory>,
    /// Setting name (if applicable)
    pub setting: Option<String>,
    /// Additional context
    pub context: Option<String>,
}

impl AnalyticsEvent {
    /// Create new event
    pub fn new(metric: MetricType) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            metric,
            category: None,
            setting: None,
            context: None,
        }
    }

    /// With category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// With setting name
    pub fn with_setting(mut self, setting: impl Into<String>) -> Self {
        self.setting = Some(setting.into());
        self
    }

    /// With context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Category statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    /// Change count
    pub changes: u64,
    /// Access count
    pub accesses: u64,
    /// Revert count
    pub reverts: u64,
    /// Last modified
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Most changed settings
    pub top_settings: Vec<(String, u64)>,
}

impl CategoryStats {
    /// Record a change
    pub fn record_change(&mut self, setting: Option<&str>) {
        self.changes += 1;
        self.last_modified = Some(chrono::Utc::now());
        if let Some(s) = setting {
            self.add_to_top(s);
        }
    }

    /// Record an access
    pub fn record_access(&mut self) {
        self.accesses += 1;
    }

    /// Record a revert
    pub fn record_revert(&mut self) {
        self.reverts += 1;
    }

    fn add_to_top(&mut self, setting: &str) {
        if let Some(pos) = self.top_settings.iter().position(|(s, _)| s == setting) {
            self.top_settings[pos].1 += 1;
        } else {
            self.top_settings.push((setting.to_string(), 1));
        }
        self.top_settings.sort_by(|a, b| b.1.cmp(&a.1));
        self.top_settings.truncate(5);
    }
}

/// Overall analytics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    /// Total changes
    pub total_changes: u64,
    /// Total accesses
    pub total_accesses: u64,
    /// Total reverts
    pub total_reverts: u64,
    /// Total exports
    pub total_exports: u64,
    /// Total imports
    pub total_imports: u64,
    /// Most active category
    pub most_active_category: Option<SettingsCategory>,
    /// First activity
    pub first_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Last activity
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl AnalyticsSummary {
    /// Calculate totals
    pub fn total_events(&self) -> u64 {
        self.total_changes + self.total_accesses + self.total_reverts
            + self.total_exports + self.total_imports
    }

    /// Activity level (0-100)
    pub fn activity_score(&self) -> u8 {
        let total = self.total_events();
        match total {
            0 => 0,
            1..=10 => 20,
            11..=50 => 40,
            51..=100 => 60,
            101..=500 => 80,
            _ => 100,
        }
    }
}

/// Settings analytics tracker
#[derive(Debug, Clone, Default)]
pub struct SettingsAnalytics {
    /// Events history
    events: Vec<AnalyticsEvent>,
    /// Per-category stats
    category_stats: HashMap<SettingsCategory, CategoryStats>,
    /// Summary
    summary: AnalyticsSummary,
    /// Max events to keep
    max_events: usize,
}

impl SettingsAnalytics {
    /// Create new analytics tracker
    pub fn new() -> Self {
        Self {
            max_events: 1000,
            ..Default::default()
        }
    }

    /// Record an event
    pub fn record(&mut self, event: AnalyticsEvent) {
        // Update summary
        match event.metric {
            MetricType::ChangeCount => self.summary.total_changes += 1,
            MetricType::AccessCount => self.summary.total_accesses += 1,
            MetricType::RevertCount => self.summary.total_reverts += 1,
            MetricType::ExportCount => self.summary.total_exports += 1,
            MetricType::ImportCount => self.summary.total_imports += 1,
        }

        // Update timestamps
        if self.summary.first_activity.is_none() {
            self.summary.first_activity = Some(event.timestamp);
        }
        self.summary.last_activity = Some(event.timestamp);

        // Update category stats
        if let Some(category) = event.category {
            let stats = self.category_stats.entry(category).or_default();
            match event.metric {
                MetricType::ChangeCount => stats.record_change(event.setting.as_deref()),
                MetricType::AccessCount => stats.record_access(),
                MetricType::RevertCount => stats.record_revert(),
                _ => {}
            }
        }

        // Store event
        self.events.push(event);
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }

        // Update most active category
        self.update_most_active();
    }

    /// Record a change
    pub fn record_change(&mut self, category: SettingsCategory, setting: &str) {
        self.record(
            AnalyticsEvent::new(MetricType::ChangeCount)
                .with_category(category)
                .with_setting(setting)
        );
    }

    /// Record an access
    pub fn record_access(&mut self, category: SettingsCategory) {
        self.record(
            AnalyticsEvent::new(MetricType::AccessCount)
                .with_category(category)
        );
    }

    /// Record a revert
    pub fn record_revert(&mut self, category: SettingsCategory) {
        self.record(
            AnalyticsEvent::new(MetricType::RevertCount)
                .with_category(category)
        );
    }

    /// Record an export
    pub fn record_export(&mut self) {
        self.record(AnalyticsEvent::new(MetricType::ExportCount));
    }

    /// Record an import
    pub fn record_import(&mut self) {
        self.record(AnalyticsEvent::new(MetricType::ImportCount));
    }

    fn update_most_active(&mut self) {
        let most_active = self.category_stats
            .iter()
            .max_by_key(|(_, stats)| stats.changes)
            .map(|(cat, _)| *cat);
        self.summary.most_active_category = most_active;
    }

    /// Get summary
    pub fn summary(&self) -> &AnalyticsSummary {
        &self.summary
    }

    /// Get category stats
    pub fn category_stats(&self, category: SettingsCategory) -> Option<&CategoryStats> {
        self.category_stats.get(&category)
    }

    /// Get all category stats
    pub fn all_category_stats(&self) -> &HashMap<SettingsCategory, CategoryStats> {
        &self.category_stats
    }

    /// Get events for period
    pub fn events_for_period(&self, period: AnalyticsPeriod) -> Vec<&AnalyticsEvent> {
        let cutoff = match period {
            AnalyticsPeriod::Day => chrono::Utc::now() - chrono::Duration::days(1),
            AnalyticsPeriod::Week => chrono::Utc::now() - chrono::Duration::weeks(1),
            AnalyticsPeriod::Month => chrono::Utc::now() - chrono::Duration::days(30),
            AnalyticsPeriod::AllTime => chrono::DateTime::<chrono::Utc>::MIN_UTC,
        };

        self.events.iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect()
    }

    /// Get recent events
    pub fn recent(&self, count: usize) -> Vec<&AnalyticsEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// Total events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Format analytics for display
pub fn format_analytics(analytics: &SettingsAnalytics) -> String {
    let mut output = String::new();
    let summary = analytics.summary();

    output.push_str("=== Settings Analytics ===\n\n");
    output.push_str(&format!("Activity Score: {}%\n", summary.activity_score()));
    output.push_str(&format!("Total Events: {}\n\n", summary.total_events()));

    output.push_str("--- Metrics ---\n");
    output.push_str(&format!("Changes: {}\n", summary.total_changes));
    output.push_str(&format!("Accesses: {}\n", summary.total_accesses));
    output.push_str(&format!("Reverts: {}\n", summary.total_reverts));
    output.push_str(&format!("Exports: {}\n", summary.total_exports));
    output.push_str(&format!("Imports: {}\n\n", summary.total_imports));

    if let Some(cat) = summary.most_active_category {
        output.push_str(&format!("Most Active Category: {}\n", cat));
    }

    output
}

/// Check if query is about analytics
pub fn is_analytics_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("analytics")
        || lower.contains("statistics")
        || lower.contains("settings usage")
        || lower.contains("how often")
}

/// Fun fact about analytics
pub fn settings_analytics_fun_fact() -> &'static str {
    "Anna tracks your settings usage to help you understand your preferences!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_period_display() {
        assert_eq!(format!("{}", AnalyticsPeriod::Day), "Last 24 Hours");
        assert_eq!(format!("{}", AnalyticsPeriod::Week), "Last 7 Days");
    }

    #[test]
    fn test_metric_type_display() {
        assert_eq!(format!("{}", MetricType::ChangeCount), "Changes");
        assert_eq!(format!("{}", MetricType::ExportCount), "Exports");
    }

    #[test]
    fn test_analytics_event_new() {
        let event = AnalyticsEvent::new(MetricType::ChangeCount);
        assert_eq!(event.metric, MetricType::ChangeCount);
        assert!(event.category.is_none());
    }

    #[test]
    fn test_analytics_event_with_category() {
        let event = AnalyticsEvent::new(MetricType::ChangeCount)
            .with_category(SettingsCategory::Personality);
        assert_eq!(event.category, Some(SettingsCategory::Personality));
    }

    #[test]
    fn test_category_stats_record_change() {
        let mut stats = CategoryStats::default();
        stats.record_change(Some("test_setting"));
        assert_eq!(stats.changes, 1);
        assert!(stats.last_modified.is_some());
    }

    #[test]
    fn test_analytics_summary_total_events() {
        let mut summary = AnalyticsSummary::default();
        summary.total_changes = 5;
        summary.total_accesses = 10;
        assert_eq!(summary.total_events(), 15);
    }

    #[test]
    fn test_analytics_summary_activity_score() {
        let mut summary = AnalyticsSummary::default();
        assert_eq!(summary.activity_score(), 0);
        summary.total_changes = 50;
        assert_eq!(summary.activity_score(), 40);
    }

    #[test]
    fn test_settings_analytics_new() {
        let analytics = SettingsAnalytics::new();
        assert_eq!(analytics.event_count(), 0);
    }

    #[test]
    fn test_settings_analytics_record_change() {
        let mut analytics = SettingsAnalytics::new();
        analytics.record_change(SettingsCategory::Personality, "mode");
        assert_eq!(analytics.summary().total_changes, 1);
    }

    #[test]
    fn test_settings_analytics_record_export() {
        let mut analytics = SettingsAnalytics::new();
        analytics.record_export();
        assert_eq!(analytics.summary().total_exports, 1);
    }

    #[test]
    fn test_settings_analytics_category_stats() {
        let mut analytics = SettingsAnalytics::new();
        analytics.record_change(SettingsCategory::Risk, "tolerance");
        let stats = analytics.category_stats(SettingsCategory::Risk);
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().changes, 1);
    }

    #[test]
    fn test_format_analytics() {
        let analytics = SettingsAnalytics::new();
        let output = format_analytics(&analytics);
        assert!(output.contains("Analytics"));
    }

    #[test]
    fn test_is_analytics_query() {
        assert!(is_analytics_query("show analytics"));
        assert!(is_analytics_query("settings statistics"));
        assert!(!is_analytics_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_analytics_fun_fact();
        assert!(fact.contains("track"));
    }
}
