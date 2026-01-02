// v0.0.577: Settings Analytics - Event (Phase 153)

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::MetricType;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
