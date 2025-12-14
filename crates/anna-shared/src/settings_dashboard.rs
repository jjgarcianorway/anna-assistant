// v0.0.579: Settings Dashboard (Phase 155)
// Unified dashboard for settings overview and management

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Dashboard section type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardSection {
    /// Overview summary
    Overview,
    /// Recent changes
    RecentChanges,
    /// Active recommendations
    Recommendations,
    /// Quick actions
    QuickActions,
    /// Health status
    Health,
    /// Statistics
    Statistics,
}

impl std::fmt::Display for DashboardSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overview => write!(f, "Overview"),
            Self::RecentChanges => write!(f, "Recent Changes"),
            Self::Recommendations => write!(f, "Recommendations"),
            Self::QuickActions => write!(f, "Quick Actions"),
            Self::Health => write!(f, "Health"),
            Self::Statistics => write!(f, "Statistics"),
        }
    }
}

/// Health status level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HealthLevel {
    /// Excellent
    Excellent,
    /// Good
    #[default]
    Good,
    /// Fair
    Fair,
    /// Poor
    Poor,
    /// Critical
    Critical,
}

impl std::fmt::Display for HealthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Excellent => write!(f, "Excellent"),
            Self::Good => write!(f, "Good"),
            Self::Fair => write!(f, "Fair"),
            Self::Poor => write!(f, "Poor"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Quick action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickAction {
    /// Reset to defaults
    ResetDefaults,
    /// Export settings
    Export,
    /// Import settings
    Import,
    /// Create backup
    Backup,
    /// Run diagnostics
    Diagnostics,
    /// Apply recommended
    ApplyRecommended,
}

impl std::fmt::Display for QuickAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResetDefaults => write!(f, "Reset to Defaults"),
            Self::Export => write!(f, "Export Settings"),
            Self::Import => write!(f, "Import Settings"),
            Self::Backup => write!(f, "Create Backup"),
            Self::Diagnostics => write!(f, "Run Diagnostics"),
            Self::ApplyRecommended => write!(f, "Apply Recommendations"),
        }
    }
}

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

/// Recent change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentChange {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Category
    pub category: SettingsCategory,
    /// Setting name
    pub setting: String,
    /// Old value summary
    pub old_value: String,
    /// New value summary
    pub new_value: String,
}

impl RecentChange {
    /// Create new change
    pub fn new(
        category: SettingsCategory,
        setting: impl Into<String>,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            category,
            setting: setting.into(),
            old_value: old_value.into(),
            new_value: new_value.into(),
        }
    }

    /// Age of change
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }
}

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

/// Format dashboard for display
pub fn format_dashboard(dashboard: &SettingsDashboard) -> String {
    let mut output = String::new();
    let stats = dashboard.stats();

    output.push_str("=== Settings Dashboard ===\n\n");
    output.push_str(&format!("Health: {} ({}%)\n", stats.overall_health, stats.health_score()));
    output.push_str(&format!("Settings: {}\n", stats.total_settings));
    output.push_str(&format!("Categories: {} ({} healthy)\n", stats.categories_count, stats.healthy_categories));
    output.push_str(&format!("Customization: {:.1}%\n\n", stats.customization_percent()));

    output.push_str("--- Categories ---\n");
    for cat in dashboard.categories() {
        output.push_str(&format!(
            "• {} - {} settings ({})\n",
            cat.category, cat.settings_count, cat.health
        ));
    }

    output
}

/// Check if query is about dashboard
pub fn is_dashboard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("dashboard")
        || lower.contains("settings overview")
        || lower.contains("settings summary")
        || lower.contains("settings status")
}

/// Fun fact about dashboard
pub fn settings_dashboard_fun_fact() -> &'static str {
    "Anna's dashboard gives you a bird's-eye view of all your settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_section_display() {
        assert_eq!(format!("{}", DashboardSection::Overview), "Overview");
        assert_eq!(format!("{}", DashboardSection::Health), "Health");
    }

    #[test]
    fn test_health_level_display() {
        assert_eq!(format!("{}", HealthLevel::Excellent), "Excellent");
        assert_eq!(format!("{}", HealthLevel::Critical), "Critical");
    }

    #[test]
    fn test_quick_action_display() {
        assert_eq!(format!("{}", QuickAction::Export), "Export Settings");
        assert_eq!(format!("{}", QuickAction::Backup), "Create Backup");
    }

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

    #[test]
    fn test_recent_change_new() {
        let change = RecentChange::new(
            SettingsCategory::Personality,
            "mode",
            "Casual",
            "Professional",
        );
        assert_eq!(change.category, SettingsCategory::Personality);
        assert_eq!(change.setting, "mode");
    }

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

    #[test]
    fn test_format_dashboard() {
        let dashboard = SettingsDashboard::new();
        let output = format_dashboard(&dashboard);
        assert!(output.contains("Dashboard"));
    }

    #[test]
    fn test_is_dashboard_query() {
        assert!(is_dashboard_query("show dashboard"));
        assert!(is_dashboard_query("settings overview"));
        assert!(!is_dashboard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_dashboard_fun_fact();
        assert!(fact.contains("dashboard"));
    }
}
