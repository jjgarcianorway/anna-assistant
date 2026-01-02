// v0.0.579: Settings Dashboard Types (Phase 155)
// Enums and basic types for the settings dashboard

use serde::{Deserialize, Serialize};

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
}
