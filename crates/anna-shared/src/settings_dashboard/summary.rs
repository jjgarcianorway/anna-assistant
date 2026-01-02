// v0.0.579: Settings Dashboard Summary (Phase 155)
// Category summary and statistics types

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

use super::types::HealthLevel;

/// Category summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category
    pub category: SettingsCategory,
    /// Number of settings
    pub settings_count: usize,
    /// Modified count
    pub modified_count: usize,
    /// Last modified
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Health level
    pub health: HealthLevel,
}

impl CategorySummary {
    /// Create new summary
    pub fn new(category: SettingsCategory) -> Self {
        Self {
            category,
            settings_count: 0,
            modified_count: 0,
            last_modified: None,
            health: HealthLevel::Good,
        }
    }

    /// Set settings count
    pub fn with_settings_count(mut self, count: usize) -> Self {
        self.settings_count = count;
        self
    }

    /// Set modified count
    pub fn with_modified_count(mut self, count: usize) -> Self {
        self.modified_count = count;
        self
    }

    /// Set health
    pub fn with_health(mut self, health: HealthLevel) -> Self {
        self.health = health;
        self
    }

    /// Modification percentage
    pub fn modification_percent(&self) -> f32 {
        if self.settings_count == 0 {
            0.0
        } else {
            (self.modified_count as f32 / self.settings_count as f32) * 100.0
        }
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Total settings count
    pub total_settings: usize,
    /// Modified settings count
    pub modified_settings: usize,
    /// Categories count
    pub categories_count: usize,
    /// Healthy categories
    pub healthy_categories: usize,
    /// Active recommendations
    pub active_recommendations: usize,
    /// Recent changes count
    pub recent_changes: usize,
    /// Last backup
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
    /// Overall health
    pub overall_health: HealthLevel,
}

impl DashboardStats {
    /// Calculate health score (0-100)
    pub fn health_score(&self) -> u8 {
        match self.overall_health {
            HealthLevel::Excellent => 100,
            HealthLevel::Good => 80,
            HealthLevel::Fair => 60,
            HealthLevel::Poor => 40,
            HealthLevel::Critical => 20,
        }
    }

    /// Customization percentage
    pub fn customization_percent(&self) -> f32 {
        if self.total_settings == 0 {
            0.0
        } else {
            (self.modified_settings as f32 / self.total_settings as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_summary_new() {
        let summary = CategorySummary::new(SettingsCategory::Personality);
        assert_eq!(summary.category, SettingsCategory::Personality);
        assert_eq!(summary.health, HealthLevel::Good);
    }

    #[test]
    fn test_category_summary_builder() {
        let summary = CategorySummary::new(SettingsCategory::Risk)
            .with_settings_count(5)
            .with_modified_count(2)
            .with_health(HealthLevel::Fair);
        assert_eq!(summary.settings_count, 5);
        assert_eq!(summary.modified_count, 2);
        assert_eq!(summary.modification_percent(), 40.0);
    }

    #[test]
    fn test_dashboard_stats_health_score() {
        let mut stats = DashboardStats::default();
        stats.overall_health = HealthLevel::Excellent;
        assert_eq!(stats.health_score(), 100);
        stats.overall_health = HealthLevel::Poor;
        assert_eq!(stats.health_score(), 40);
    }
}
