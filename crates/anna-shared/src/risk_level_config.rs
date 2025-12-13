// v0.0.543: Risk Level Config (Phase 119)
// Risk levels for confirmation skipping per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub enum RiskLevel {
    #[default]
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Action category for risk assessment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionCategory {
    ReadOnly,
    ConfigChange,
    ServiceRestart,
    PackageInstall,
    PackageRemove,
    FileModify,
    FileDelete,
    SystemChange,
    NetworkChange,
    UserChange,
    Custom(String),
}

impl Default for ActionCategory {
    fn default() -> Self {
        Self::ReadOnly
    }
}

impl std::fmt::Display for ActionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "Read Only"),
            Self::ConfigChange => write!(f, "Config Change"),
            Self::ServiceRestart => write!(f, "Service Restart"),
            Self::PackageInstall => write!(f, "Package Install"),
            Self::PackageRemove => write!(f, "Package Remove"),
            Self::FileModify => write!(f, "File Modify"),
            Self::FileDelete => write!(f, "File Delete"),
            Self::SystemChange => write!(f, "System Change"),
            Self::NetworkChange => write!(f, "Network Change"),
            Self::UserChange => write!(f, "User Change"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Confirmation requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConfirmationMode {
    Never,
    OnlyHigh,
    #[default]
    Normal,
    Always,
}

impl std::fmt::Display for ConfirmationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "Never"),
            Self::OnlyHigh => write!(f, "Only High Risk"),
            Self::Normal => write!(f, "Normal"),
            Self::Always => write!(f, "Always"),
        }
    }
}

/// Risk level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevelConfig {
    pub confirmation_mode: ConfirmationMode,
    pub auto_approve_up_to: RiskLevel,
    pub require_root_confirmation: bool,
    pub require_delete_confirmation: bool,
    pub category_overrides: HashMap<ActionCategory, RiskLevel>,
}

impl Default for RiskLevelConfig {
    fn default() -> Self {
        Self {
            confirmation_mode: ConfirmationMode::Normal,
            auto_approve_up_to: RiskLevel::Low,
            require_root_confirmation: true,
            require_delete_confirmation: true,
            category_overrides: HashMap::new(),
        }
    }
}

impl RiskLevelConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create permissive config (fewer confirmations)
    pub fn permissive() -> Self {
        Self {
            confirmation_mode: ConfirmationMode::OnlyHigh,
            auto_approve_up_to: RiskLevel::Medium,
            require_root_confirmation: true,
            require_delete_confirmation: true,
            category_overrides: HashMap::new(),
        }
    }

    /// Create strict config (more confirmations)
    pub fn strict() -> Self {
        Self {
            confirmation_mode: ConfirmationMode::Always,
            auto_approve_up_to: RiskLevel::None,
            require_root_confirmation: true,
            require_delete_confirmation: true,
            category_overrides: HashMap::new(),
        }
    }

    /// Should require confirmation for this risk level?
    pub fn requires_confirmation(&self, risk: RiskLevel) -> bool {
        match self.confirmation_mode {
            ConfirmationMode::Never => false,
            ConfirmationMode::Always => true,
            ConfirmationMode::OnlyHigh => risk >= RiskLevel::High,
            ConfirmationMode::Normal => risk > self.auto_approve_up_to,
        }
    }

    /// Get risk level for action category
    pub fn risk_for_category(&self, category: &ActionCategory) -> RiskLevel {
        if let Some(&override_level) = self.category_overrides.get(category) {
            return override_level;
        }

        match category {
            ActionCategory::ReadOnly => RiskLevel::None,
            ActionCategory::ConfigChange => RiskLevel::Low,
            ActionCategory::ServiceRestart => RiskLevel::Medium,
            ActionCategory::PackageInstall => RiskLevel::Low,
            ActionCategory::PackageRemove => RiskLevel::Medium,
            ActionCategory::FileModify => RiskLevel::Low,
            ActionCategory::FileDelete => RiskLevel::High,
            ActionCategory::SystemChange => RiskLevel::High,
            ActionCategory::NetworkChange => RiskLevel::Medium,
            ActionCategory::UserChange => RiskLevel::High,
            ActionCategory::Custom(_) => RiskLevel::Medium,
        }
    }

    /// Set override for category
    pub fn set_category_risk(&mut self, category: ActionCategory, risk: RiskLevel) {
        self.category_overrides.insert(category, risk);
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Confirmation mode changes
        if lower.contains("skip all confirmation") || lower.contains("never confirm") {
            self.confirmation_mode = ConfirmationMode::Never;
            return Some("All confirmations disabled. Be careful!".to_string());
        }
        if lower.contains("always confirm") || lower.contains("confirm everything") {
            self.confirmation_mode = ConfirmationMode::Always;
            return Some("All actions will now require confirmation.".to_string());
        }
        if lower.contains("only confirm high risk") || lower.contains("skip low risk") {
            self.confirmation_mode = ConfirmationMode::OnlyHigh;
            self.auto_approve_up_to = RiskLevel::Medium;
            return Some("Only high-risk actions will require confirmation.".to_string());
        }
        if lower.contains("normal confirmation") || lower.contains("default risk") {
            self.confirmation_mode = ConfirmationMode::Normal;
            self.auto_approve_up_to = RiskLevel::Low;
            return Some("Normal confirmation mode restored.".to_string());
        }

        // Auto-approve level changes
        if lower.contains("auto approve low") || lower.contains("skip low") {
            self.auto_approve_up_to = RiskLevel::Low;
            return Some("Low-risk actions will be auto-approved.".to_string());
        }
        if lower.contains("auto approve medium") {
            self.auto_approve_up_to = RiskLevel::Medium;
            return Some("Medium and lower risk actions will be auto-approved.".to_string());
        }

        // Root confirmation toggle
        if lower.contains("skip root confirm") {
            self.require_root_confirmation = false;
            return Some("Root action confirmations disabled.".to_string());
        }
        if lower.contains("require root confirm") {
            self.require_root_confirmation = true;
            return Some("Root actions will require confirmation.".to_string());
        }

        // Delete confirmation toggle
        if lower.contains("skip delete confirm") {
            self.require_delete_confirmation = false;
            return Some("Delete confirmations disabled.".to_string());
        }
        if lower.contains("require delete confirm") {
            self.require_delete_confirmation = true;
            return Some("Delete actions will require confirmation.".to_string());
        }

        None
    }

    /// Check if action needs confirmation
    pub fn needs_confirmation(&self, category: &ActionCategory, is_root: bool) -> bool {
        let risk = self.risk_for_category(category);

        // Special checks
        if is_root && self.require_root_confirmation {
            return true;
        }
        if matches!(category, ActionCategory::FileDelete) && self.require_delete_confirmation {
            return true;
        }

        self.requires_confirmation(risk)
    }
}

/// Format risk config
pub fn format_risk_config(config: &RiskLevelConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Risk Level Configuration ===\n\n");

    output.push_str(&format!("Confirmation Mode: {}\n", config.confirmation_mode));
    output.push_str(&format!("Auto-Approve Up To: {}\n", config.auto_approve_up_to));
    output.push_str(&format!("Require Root Confirmation: {}\n", config.require_root_confirmation));
    output.push_str(&format!("Require Delete Confirmation: {}\n", config.require_delete_confirmation));

    if !config.category_overrides.is_empty() {
        output.push_str("\nCategory Overrides:\n");
        for (cat, risk) in &config.category_overrides {
            output.push_str(&format!("  {}: {}\n", cat, risk));
        }
    }

    output
}

/// Check if query is risk-related
pub fn is_risk_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("risk")
        || lower.contains("confirm")
        || lower.contains("auto approve")
        || lower.contains("skip")
        || lower.contains("permission")
}

/// Fun fact about risk levels
pub fn risk_level_fun_fact() -> &'static str {
    "Anna's risk levels help balance convenience and safety. Low-risk actions can be auto-approved while critical changes always require your confirmation."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_default() {
        let risk = RiskLevel::default();
        assert_eq!(risk, RiskLevel::None);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_config_default() {
        let config = RiskLevelConfig::default();
        assert_eq!(config.confirmation_mode, ConfirmationMode::Normal);
        assert!(config.require_root_confirmation);
    }

    #[test]
    fn test_requires_confirmation() {
        let config = RiskLevelConfig::default();
        assert!(!config.requires_confirmation(RiskLevel::None));
        assert!(!config.requires_confirmation(RiskLevel::Low));
        assert!(config.requires_confirmation(RiskLevel::Medium));
        assert!(config.requires_confirmation(RiskLevel::High));
    }

    #[test]
    fn test_risk_for_category() {
        let config = RiskLevelConfig::default();
        assert_eq!(config.risk_for_category(&ActionCategory::ReadOnly), RiskLevel::None);
        assert_eq!(config.risk_for_category(&ActionCategory::FileDelete), RiskLevel::High);
    }

    #[test]
    fn test_apply_change() {
        let mut config = RiskLevelConfig::default();
        let result = config.apply_change("skip all confirmations please");
        assert!(result.is_some());
        assert_eq!(config.confirmation_mode, ConfirmationMode::Never);
    }

    #[test]
    fn test_permissive_config() {
        let config = RiskLevelConfig::permissive();
        assert_eq!(config.confirmation_mode, ConfirmationMode::OnlyHigh);
        assert_eq!(config.auto_approve_up_to, RiskLevel::Medium);
    }

    #[test]
    fn test_needs_confirmation_root() {
        let config = RiskLevelConfig::default();
        assert!(config.needs_confirmation(&ActionCategory::ReadOnly, true));
    }

    #[test]
    fn test_is_risk_query() {
        assert!(is_risk_query("Change risk settings"));
        assert!(is_risk_query("Skip confirmations"));
        assert!(!is_risk_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = risk_level_fun_fact();
        assert!(fact.contains("risk") || fact.contains("confirm"));
    }
}
