// v0.0.577: Settings Analytics - Tracker (Phase 153)

use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::types::{AnalyticsPeriod, MetricType};
use super::event::AnalyticsEvent;
use super::stats::{CategoryStats, AnalyticsSummary};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
