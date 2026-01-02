// v0.0.579: Settings Dashboard Core (Phase 155)
// Main dashboard implementation

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::changes::RecentChange;
use super::summary::{CategorySummary, DashboardStats};
use super::types::{DashboardSection, HealthLevel};

/// Settings dashboard
#[derive(Debug, Clone, Default)]
pub struct SettingsDashboard {
    /// Category summaries
    categories: Vec<CategorySummary>,
    /// Statistics
    stats: DashboardStats,
    /// Recent changes
    recent_changes: Vec<RecentChange>,
    /// Visible sections
    visible_sections: Vec<DashboardSection>,
}

impl SettingsDashboard {
    /// Create new dashboard
    pub fn new() -> Self {
        Self {
            visible_sections: vec![
                DashboardSection::Overview,
                DashboardSection::Health,
                DashboardSection::RecentChanges,
                DashboardSection::Recommendations,
                DashboardSection::QuickActions,
            ],
            ..Default::default()
        }
    }

    /// Refresh dashboard data
    pub fn refresh(&mut self, settings: &UnifiedSettings) {
        self.categories.clear();

        // Generate category summaries
        let categories = [
            (SettingsCategory::Personality, 4),
            (SettingsCategory::Risk, 5),
            (SettingsCategory::Learning, 3),
            (SettingsCategory::Escalation, 3),
            (SettingsCategory::Verbosity, 4),
            (SettingsCategory::Confirmation, 8),
            (SettingsCategory::Timeout, 10),
            (SettingsCategory::OutputStyle, 5),
            (SettingsCategory::Privacy, 4),
            (SettingsCategory::Backup, 4),
            (SettingsCategory::Update, 4),
            (SettingsCategory::Model, 3),
        ];

        let mut total = 0;
        let mut healthy = 0;

        for (cat, count) in categories {
            let summary = CategorySummary::new(cat)
                .with_settings_count(count)
                .with_health(HealthLevel::Good);
            if summary.health == HealthLevel::Good || summary.health == HealthLevel::Excellent {
                healthy += 1;
            }
            total += count;
            self.categories.push(summary);
        }

        // Update stats
        self.stats.total_settings = total;
        self.stats.categories_count = self.categories.len();
        self.stats.healthy_categories = healthy;
        self.stats.overall_health = self.calculate_overall_health(settings);
    }

    fn calculate_overall_health(&self, _settings: &UnifiedSettings) -> HealthLevel {
        let healthy_ratio = self.stats.healthy_categories as f32 / self.stats.categories_count.max(1) as f32;
        if healthy_ratio >= 0.9 {
            HealthLevel::Excellent
        } else if healthy_ratio >= 0.7 {
            HealthLevel::Good
        } else if healthy_ratio >= 0.5 {
            HealthLevel::Fair
        } else if healthy_ratio >= 0.3 {
            HealthLevel::Poor
        } else {
            HealthLevel::Critical
        }
    }

    /// Get category summaries
    pub fn categories(&self) -> &[CategorySummary] {
        &self.categories
    }

    /// Get statistics
    pub fn stats(&self) -> &DashboardStats {
        &self.stats
    }

    /// Get recent changes
    pub fn recent_changes(&self) -> &[RecentChange] {
        &self.recent_changes
    }

    /// Add recent change
    pub fn add_change(&mut self, change: RecentChange) {
        self.recent_changes.insert(0, change);
        while self.recent_changes.len() > 20 {
            self.recent_changes.pop();
        }
        self.stats.recent_changes = self.recent_changes.len();
    }

    /// Get visible sections
    pub fn visible_sections(&self) -> &[DashboardSection] {
        &self.visible_sections
    }

    /// Show/hide section
    pub fn set_section_visible(&mut self, section: DashboardSection, visible: bool) {
        if visible {
            if !self.visible_sections.contains(&section) {
                self.visible_sections.push(section);
            }
        } else {
            self.visible_sections.retain(|s| s != &section);
        }
    }

    /// Get category summary
    pub fn get_category(&self, category: SettingsCategory) -> Option<&CategorySummary> {
        self.categories.iter().find(|c| c.category == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_dashboard_new() {
        let dashboard = SettingsDashboard::new();
        assert!(!dashboard.visible_sections().is_empty());
    }

    #[test]
    fn test_settings_dashboard_refresh() {
        let mut dashboard = SettingsDashboard::new();
        let settings = UnifiedSettings::default();
        dashboard.refresh(&settings);
        assert!(dashboard.stats().total_settings > 0);
    }

    #[test]
    fn test_settings_dashboard_add_change() {
        let mut dashboard = SettingsDashboard::new();
        let change = RecentChange::new(
            SettingsCategory::Risk,
            "tolerance",
            "Low",
            "High",
        );
        dashboard.add_change(change);
        assert_eq!(dashboard.recent_changes().len(), 1);
    }
}
