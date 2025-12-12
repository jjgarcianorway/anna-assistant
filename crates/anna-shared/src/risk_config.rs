//! Risk Level Configuration via Natural Language.
//!
//! Allows users to configure risk tolerance through natural language:
//! - "set risk level to high" - allow high-risk operations without confirmation
//! - "be cautious" - require confirmation for all changes
//! - "auto-confirm low risk" - skip confirmation for safe operations
//! - "always ask before changes" - require confirmation for everything
//!
//! Per VISION.md: "Risk levels for confirmation skipping"

use crate::recipe_v3::RecipeRiskLevel;
use serde::{Deserialize, Serialize};

/// User's risk tolerance configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskTolerance {
    /// Maximum risk level that can be auto-confirmed (without asking)
    pub max_auto_confirm: RecipeRiskLevel,
    /// Whether to show warnings for risky operations
    pub show_warnings: bool,
    /// Whether to require explicit confirmation for destructive ops
    pub protect_destructive: bool,
}

impl Default for RiskTolerance {
    fn default() -> Self {
        Self {
            max_auto_confirm: RecipeRiskLevel::None, // Safe default: confirm everything
            show_warnings: true,
            protect_destructive: true,
        }
    }
}

impl RiskTolerance {
    /// Cautious preset - confirm everything
    pub fn cautious() -> Self {
        Self {
            max_auto_confirm: RecipeRiskLevel::None,
            show_warnings: true,
            protect_destructive: true,
        }
    }

    /// Balanced preset - auto-confirm low risk
    pub fn balanced() -> Self {
        Self {
            max_auto_confirm: RecipeRiskLevel::Low,
            show_warnings: true,
            protect_destructive: true,
        }
    }

    /// Confident preset - auto-confirm medium risk
    pub fn confident() -> Self {
        Self {
            max_auto_confirm: RecipeRiskLevel::Medium,
            show_warnings: true,
            protect_destructive: true,
        }
    }

    /// Expert preset - auto-confirm most operations
    pub fn expert() -> Self {
        Self {
            max_auto_confirm: RecipeRiskLevel::High,
            show_warnings: false,
            protect_destructive: true,
        }
    }

    /// Check if a risk level should be auto-confirmed
    pub fn should_auto_confirm(&self, risk: RecipeRiskLevel) -> bool {
        risk <= self.max_auto_confirm
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self.max_auto_confirm {
            RecipeRiskLevel::None => "Cautious - confirm all changes",
            RecipeRiskLevel::Low => "Balanced - auto-confirm safe changes",
            RecipeRiskLevel::Medium => "Confident - auto-confirm most changes",
            RecipeRiskLevel::High => "Expert - minimal confirmations",
        }
    }
}

/// Risk configuration change from natural language
#[derive(Debug, Clone, PartialEq)]
pub enum RiskConfigChange {
    /// Set risk tolerance preset
    SetPreset(RiskPreset),
    /// Set maximum auto-confirm level
    SetMaxAutoConfirm(RecipeRiskLevel),
    /// Enable/disable warnings
    ShowWarnings(bool),
    /// Enable/disable destructive protection
    ProtectDestructive(bool),
}

/// Risk tolerance presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskPreset {
    Cautious,
    Balanced,
    Confident,
    Expert,
}

impl RiskPreset {
    /// Get name
    pub fn name(&self) -> &'static str {
        match self {
            RiskPreset::Cautious => "cautious",
            RiskPreset::Balanced => "balanced",
            RiskPreset::Confident => "confident",
            RiskPreset::Expert => "expert",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            RiskPreset::Cautious => "Confirm all changes, show all warnings",
            RiskPreset::Balanced => "Auto-confirm safe (read-only) operations",
            RiskPreset::Confident => "Auto-confirm low and medium risk operations",
            RiskPreset::Expert => "Minimal confirmations, hide warnings",
        }
    }
}

impl RiskConfigChange {
    /// Human-readable description
    pub fn description(&self) -> String {
        match self {
            RiskConfigChange::SetPreset(preset) => {
                format!("Set risk tolerance to {} ({})", preset.name(), preset.description())
            }
            RiskConfigChange::SetMaxAutoConfirm(level) => {
                let level_name = match level {
                    RecipeRiskLevel::None => "none (confirm everything)",
                    RecipeRiskLevel::Low => "low (auto-confirm safe ops)",
                    RecipeRiskLevel::Medium => "medium (auto-confirm most ops)",
                    RecipeRiskLevel::High => "high (minimal confirmations)",
                };
                format!("Set max auto-confirm level to {}", level_name)
            }
            RiskConfigChange::ShowWarnings(true) => "Enabled risk warnings".to_string(),
            RiskConfigChange::ShowWarnings(false) => "Disabled risk warnings".to_string(),
            RiskConfigChange::ProtectDestructive(true) => {
                "Enabled protection for destructive operations".to_string()
            }
            RiskConfigChange::ProtectDestructive(false) => {
                "Disabled protection for destructive operations".to_string()
            }
        }
    }
}

/// Detect risk configuration changes from natural language
pub fn detect_risk_config(query: &str) -> Option<RiskConfigChange> {
    let lower = query.to_lowercase();

    // Preset changes
    if matches_any(&lower, &["be cautious", "cautious mode", "careful mode", "always confirm"]) {
        return Some(RiskConfigChange::SetPreset(RiskPreset::Cautious));
    }

    if matches_any(&lower, &["balanced risk", "normal risk", "default risk"]) {
        return Some(RiskConfigChange::SetPreset(RiskPreset::Balanced));
    }

    if matches_any(&lower, &["confident mode", "trust me", "less confirmations"]) {
        return Some(RiskConfigChange::SetPreset(RiskPreset::Confident));
    }

    if matches_any(&lower, &["expert mode", "no confirmations", "skip confirmations"]) {
        return Some(RiskConfigChange::SetPreset(RiskPreset::Expert));
    }

    // Auto-confirm level
    if matches_any(&lower, &["auto-confirm low", "auto confirm low", "skip low risk"]) {
        return Some(RiskConfigChange::SetMaxAutoConfirm(RecipeRiskLevel::Low));
    }

    if matches_any(&lower, &["auto-confirm medium", "auto confirm medium", "skip medium risk"]) {
        return Some(RiskConfigChange::SetMaxAutoConfirm(RecipeRiskLevel::Medium));
    }

    if matches_any(&lower, &["confirm everything", "no auto-confirm", "ask for everything"]) {
        return Some(RiskConfigChange::SetMaxAutoConfirm(RecipeRiskLevel::None));
    }

    // Warnings
    if matches_any(&lower, &["show warnings", "enable warnings", "warn me"]) {
        return Some(RiskConfigChange::ShowWarnings(true));
    }

    if matches_any(&lower, &["hide warnings", "disable warnings", "no warnings"]) {
        return Some(RiskConfigChange::ShowWarnings(false));
    }

    // Destructive protection
    if matches_any(&lower, &["protect destructive", "block destructive", "prevent delete"]) {
        return Some(RiskConfigChange::ProtectDestructive(true));
    }

    if matches_any(&lower, &["allow destructive", "enable destructive", "allow delete"]) {
        return Some(RiskConfigChange::ProtectDestructive(false));
    }

    None
}

/// Check if query is asking about risk settings
pub fn is_show_risk_settings(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "show risk", "risk settings", "risk level", "what risk",
        "current risk", "risk config", "confirmation settings"
    ])
}

/// Apply risk config change
pub fn apply_risk_change(config: &mut RiskTolerance, change: &RiskConfigChange) {
    match change {
        RiskConfigChange::SetPreset(preset) => {
            *config = match preset {
                RiskPreset::Cautious => RiskTolerance::cautious(),
                RiskPreset::Balanced => RiskTolerance::balanced(),
                RiskPreset::Confident => RiskTolerance::confident(),
                RiskPreset::Expert => RiskTolerance::expert(),
            };
        }
        RiskConfigChange::SetMaxAutoConfirm(level) => {
            config.max_auto_confirm = *level;
        }
        RiskConfigChange::ShowWarnings(enabled) => {
            config.show_warnings = *enabled;
        }
        RiskConfigChange::ProtectDestructive(enabled) => {
            config.protect_destructive = *enabled;
        }
    }
}

/// Format risk settings for display
pub fn format_risk_settings(config: &RiskTolerance) -> String {
    let mut lines = vec![
        format!("risk_tolerance      {}", config.description()),
        format!("max_auto_confirm    {}", risk_level_name(config.max_auto_confirm)),
        format!("show_warnings       {}", if config.show_warnings { "yes" } else { "no" }),
        format!("protect_destructive {}", if config.protect_destructive { "yes" } else { "no" }),
    ];

    lines.push(String::new());
    lines.push("Configure via natural language:".to_string());
    lines.push("  \"be cautious\" - confirm all changes".to_string());
    lines.push("  \"auto-confirm low risk\" - skip safe confirmations".to_string());
    lines.push("  \"expert mode\" - minimal confirmations".to_string());

    lines.join("\n")
}

fn risk_level_name(level: RecipeRiskLevel) -> &'static str {
    match level {
        RecipeRiskLevel::None => "none",
        RecipeRiskLevel::Low => "low",
        RecipeRiskLevel::Medium => "medium",
        RecipeRiskLevel::High => "high",
    }
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cautious() {
        let change = detect_risk_config("be cautious please");
        assert_eq!(change, Some(RiskConfigChange::SetPreset(RiskPreset::Cautious)));
    }

    #[test]
    fn test_detect_expert() {
        let change = detect_risk_config("enable expert mode");
        assert_eq!(change, Some(RiskConfigChange::SetPreset(RiskPreset::Expert)));
    }

    #[test]
    fn test_detect_auto_confirm() {
        let change = detect_risk_config("auto-confirm low risk");
        assert_eq!(change, Some(RiskConfigChange::SetMaxAutoConfirm(RecipeRiskLevel::Low)));
    }

    #[test]
    fn test_detect_warnings() {
        let change = detect_risk_config("hide warnings");
        assert_eq!(change, Some(RiskConfigChange::ShowWarnings(false)));

        let change = detect_risk_config("show warnings");
        assert_eq!(change, Some(RiskConfigChange::ShowWarnings(true)));
    }

    #[test]
    fn test_is_show_risk() {
        assert!(is_show_risk_settings("show risk settings"));
        assert!(is_show_risk_settings("what is my risk level"));
        assert!(!is_show_risk_settings("how much disk space"));
    }

    #[test]
    fn test_apply_preset() {
        let mut config = RiskTolerance::default();
        assert_eq!(config.max_auto_confirm, RecipeRiskLevel::None);

        apply_risk_change(&mut config, &RiskConfigChange::SetPreset(RiskPreset::Balanced));
        assert_eq!(config.max_auto_confirm, RecipeRiskLevel::Low);
    }

    #[test]
    fn test_apply_max_auto_confirm() {
        let mut config = RiskTolerance::default();
        apply_risk_change(&mut config, &RiskConfigChange::SetMaxAutoConfirm(RecipeRiskLevel::Medium));
        assert_eq!(config.max_auto_confirm, RecipeRiskLevel::Medium);
    }

    #[test]
    fn test_should_auto_confirm() {
        let balanced = RiskTolerance::balanced();
        assert!(balanced.should_auto_confirm(RecipeRiskLevel::None));
        assert!(balanced.should_auto_confirm(RecipeRiskLevel::Low));
        assert!(!balanced.should_auto_confirm(RecipeRiskLevel::Medium));
        assert!(!balanced.should_auto_confirm(RecipeRiskLevel::High));
    }

    #[test]
    fn test_format_settings() {
        let config = RiskTolerance::balanced();
        let output = format_risk_settings(&config);
        assert!(output.contains("risk_tolerance"));
        assert!(output.contains("Balanced"));
        assert!(output.contains("max_auto_confirm"));
    }

    #[test]
    fn test_description() {
        let change = RiskConfigChange::SetPreset(RiskPreset::Balanced);
        let desc = change.description();
        assert!(desc.contains("balanced"));
    }
}
