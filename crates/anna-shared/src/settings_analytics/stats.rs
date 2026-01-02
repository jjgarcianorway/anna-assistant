// v0.0.577: Settings Analytics - Stats (Phase 153)

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
