// v0.0.568: Settings Scheduler - Action Types
// Actions that can be scheduled

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Scheduled action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduledAction {
    /// Switch to a profile
    SwitchProfile(String),
    /// Apply specific settings
    ApplySettings(Box<UnifiedSettings>),
    /// Change single setting
    ChangeSetting { category: SettingsCategory, field: String, value: String },
    /// Reset category to defaults
    ResetCategory(SettingsCategory),
    /// Enable/disable sync
    SetSyncEnabled(bool),
}

impl std::fmt::Display for ScheduledAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SwitchProfile(p) => write!(f, "Switch to profile '{}'", p),
            Self::ApplySettings(_) => write!(f, "Apply settings"),
            Self::ChangeSetting { category, field, value } => {
                write!(f, "Set {}.{} = {}", category, field, value)
            }
            Self::ResetCategory(c) => write!(f, "Reset {} to defaults", c),
            Self::SetSyncEnabled(e) => write!(f, "Set sync enabled = {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        let a = ScheduledAction::SwitchProfile("work".to_string());
        assert!(format!("{}", a).contains("work"));
    }
}
