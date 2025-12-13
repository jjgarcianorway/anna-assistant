// v0.0.547: Confirmation Behavior Config (Phase 123)
// Configurable confirmation behavior per VISION.md

use serde::{Deserialize, Serialize};

/// Confirmation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConfirmationStyle {
    #[default]
    Inline,
    Prompt,
    Dialog,
    Silent,
}

impl std::fmt::Display for ConfirmationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline => write!(f, "Inline"),
            Self::Prompt => write!(f, "Prompt"),
            Self::Dialog => write!(f, "Dialog"),
            Self::Silent => write!(f, "Silent (auto-confirm)"),
        }
    }
}

/// Confirmation timeout behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TimeoutBehavior {
    #[default]
    Deny,
    Approve,
    AskAgain,
}

impl std::fmt::Display for TimeoutBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deny => write!(f, "Deny on timeout"),
            Self::Approve => write!(f, "Approve on timeout"),
            Self::AskAgain => write!(f, "Ask again on timeout"),
        }
    }
}

/// Action types that may require confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfirmableAction {
    FileModification,
    FileDeletion,
    PackageInstall,
    PackageRemove,
    ServiceControl,
    ConfigChange,
    SystemCommand,
    NetworkChange,
    RootAction,
}

impl std::fmt::Display for ConfirmableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileModification => write!(f, "File Modification"),
            Self::FileDeletion => write!(f, "File Deletion"),
            Self::PackageInstall => write!(f, "Package Install"),
            Self::PackageRemove => write!(f, "Package Remove"),
            Self::ServiceControl => write!(f, "Service Control"),
            Self::ConfigChange => write!(f, "Config Change"),
            Self::SystemCommand => write!(f, "System Command"),
            Self::NetworkChange => write!(f, "Network Change"),
            Self::RootAction => write!(f, "Root Action"),
        }
    }
}

/// Confirmation behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationBehaviorConfig {
    pub style: ConfirmationStyle,
    pub timeout_behavior: TimeoutBehavior,
    pub timeout_seconds: u32,
    pub show_command_preview: bool,
    pub show_risk_level: bool,
    pub require_explicit_yes: bool,
    pub remember_choices: bool,
    pub batch_confirmations: bool,
    pub auto_confirm_safe: bool,
    pub always_confirm_root: bool,
    pub always_confirm_delete: bool,
}

impl Default for ConfirmationBehaviorConfig {
    fn default() -> Self {
        Self {
            style: ConfirmationStyle::Inline,
            timeout_behavior: TimeoutBehavior::Deny,
            timeout_seconds: 30,
            show_command_preview: true,
            show_risk_level: true,
            require_explicit_yes: false,
            remember_choices: true,
            batch_confirmations: false,
            auto_confirm_safe: false,
            always_confirm_root: true,
            always_confirm_delete: true,
        }
    }
}

impl ConfirmationBehaviorConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Strict mode - always confirm, explicit yes
    pub fn strict() -> Self {
        Self {
            style: ConfirmationStyle::Prompt,
            timeout_behavior: TimeoutBehavior::Deny,
            timeout_seconds: 60,
            show_command_preview: true,
            show_risk_level: true,
            require_explicit_yes: true,
            remember_choices: false,
            batch_confirmations: false,
            auto_confirm_safe: false,
            always_confirm_root: true,
            always_confirm_delete: true,
        }
    }

    /// Lenient mode - auto-confirm safe operations
    pub fn lenient() -> Self {
        Self {
            style: ConfirmationStyle::Inline,
            timeout_behavior: TimeoutBehavior::Approve,
            timeout_seconds: 10,
            show_command_preview: false,
            show_risk_level: false,
            require_explicit_yes: false,
            remember_choices: true,
            batch_confirmations: true,
            auto_confirm_safe: true,
            always_confirm_root: true,
            always_confirm_delete: true,
        }
    }

    /// Silent mode - minimal confirmations
    pub fn silent() -> Self {
        Self {
            style: ConfirmationStyle::Silent,
            timeout_behavior: TimeoutBehavior::Approve,
            timeout_seconds: 5,
            show_command_preview: false,
            show_risk_level: false,
            require_explicit_yes: false,
            remember_choices: true,
            batch_confirmations: true,
            auto_confirm_safe: true,
            always_confirm_root: true,
            always_confirm_delete: false,
        }
    }

    /// Needs confirmation for action?
    pub fn needs_confirmation(&self, action: ConfirmableAction) -> bool {
        if self.style == ConfirmationStyle::Silent {
            return matches!(action, ConfirmableAction::RootAction)
                && self.always_confirm_root;
        }

        match action {
            ConfirmableAction::RootAction => self.always_confirm_root,
            ConfirmableAction::FileDeletion => self.always_confirm_delete,
            ConfirmableAction::FileModification
            | ConfirmableAction::ConfigChange => !self.auto_confirm_safe,
            _ => true,
        }
    }

    /// Is auto-confirm enabled for safe operations?
    pub fn is_auto_confirm_safe(&self) -> bool {
        self.auto_confirm_safe
    }

    /// Is silent mode?
    pub fn is_silent(&self) -> bool {
        self.style == ConfirmationStyle::Silent
    }

    /// Should show command preview?
    pub fn should_show_preview(&self) -> bool {
        self.show_command_preview && self.style != ConfirmationStyle::Silent
    }

    /// Should remember user choices?
    pub fn should_remember(&self) -> bool {
        self.remember_choices
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Style changes
        if lower.contains("strict confirmation") || lower.contains("always confirm") {
            *self = Self::strict();
            return Some("Strict confirmation mode - will always ask explicitly.".to_string());
        }
        if lower.contains("lenient confirmation") || lower.contains("less confirmation") {
            *self = Self::lenient();
            return Some("Lenient confirmation mode - auto-confirms safe operations.".to_string());
        }
        if lower.contains("silent") || lower.contains("no confirm") || lower.contains("auto confirm") {
            *self = Self::silent();
            return Some("Silent mode - minimal confirmations.".to_string());
        }

        // Individual toggles
        if lower.contains("require yes") || lower.contains("explicit yes") {
            self.require_explicit_yes = true;
            return Some("Now requiring explicit 'yes' for confirmations.".to_string());
        }
        if lower.contains("quick confirm") || lower.contains("enter to confirm") {
            self.require_explicit_yes = false;
            return Some("Quick confirmation enabled - Enter to confirm.".to_string());
        }
        if lower.contains("show preview") || lower.contains("show command") {
            self.show_command_preview = true;
            return Some("Command preview will be shown before execution.".to_string());
        }
        if lower.contains("hide preview") || lower.contains("no preview") {
            self.show_command_preview = false;
            return Some("Command preview hidden.".to_string());
        }
        if lower.contains("don't remember") || lower.contains("dont remember") || lower.contains("forget choice") {
            self.remember_choices = false;
            return Some("Choices will not be remembered.".to_string());
        }
        if lower.contains("remember") || lower.contains("save choice") {
            self.remember_choices = true;
            return Some("Choices will be remembered.".to_string());
        }
        if lower.contains("always confirm root") {
            self.always_confirm_root = true;
            return Some("Root actions will always require confirmation.".to_string());
        }
        if lower.contains("always confirm delete") {
            self.always_confirm_delete = true;
            return Some("Delete actions will always require confirmation.".to_string());
        }

        None
    }
}

/// Format confirmation behavior config
pub fn format_confirmation_config(config: &ConfirmationBehaviorConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Confirmation Behavior Configuration ===\n\n");

    output.push_str(&format!("Style: {}\n", config.style));
    output.push_str(&format!("Timeout Behavior: {}\n", config.timeout_behavior));
    output.push_str(&format!("Timeout: {}s\n", config.timeout_seconds));
    output.push_str(&format!("Show Command Preview: {}\n", config.show_command_preview));
    output.push_str(&format!("Show Risk Level: {}\n", config.show_risk_level));
    output.push_str(&format!("Require Explicit Yes: {}\n", config.require_explicit_yes));
    output.push_str(&format!("Remember Choices: {}\n", config.remember_choices));
    output.push_str(&format!("Batch Confirmations: {}\n", config.batch_confirmations));
    output.push_str(&format!("Auto-Confirm Safe: {}\n", config.auto_confirm_safe));
    output.push_str(&format!("Always Confirm Root: {}\n", config.always_confirm_root));
    output.push_str(&format!("Always Confirm Delete: {}\n", config.always_confirm_delete));

    output
}

/// Check if query is confirmation-related
pub fn is_confirmation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("confirmation")
        || lower.contains("confirm setting")
        || lower.contains("ask before")
        || lower.contains("auto confirm")
}

/// Fun fact about confirmation
pub fn confirmation_fun_fact() -> &'static str {
    "The famous 'Are you sure?' confirmation has prevented countless accidental deletions since the 1980s!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_display() {
        assert_eq!(format!("{}", ConfirmationStyle::Inline), "Inline");
        assert_eq!(format!("{}", ConfirmationStyle::Silent), "Silent (auto-confirm)");
    }

    #[test]
    fn test_default_config() {
        let config = ConfirmationBehaviorConfig::default();
        assert_eq!(config.style, ConfirmationStyle::Inline);
        assert!(config.always_confirm_root);
    }

    #[test]
    fn test_strict_preset() {
        let config = ConfirmationBehaviorConfig::strict();
        assert!(config.require_explicit_yes);
        assert!(!config.auto_confirm_safe);
    }

    #[test]
    fn test_lenient_preset() {
        let config = ConfirmationBehaviorConfig::lenient();
        assert!(config.auto_confirm_safe);
        assert!(config.batch_confirmations);
    }

    #[test]
    fn test_silent_preset() {
        let config = ConfirmationBehaviorConfig::silent();
        assert_eq!(config.style, ConfirmationStyle::Silent);
        assert!(config.is_silent());
    }

    #[test]
    fn test_needs_confirmation() {
        let config = ConfirmationBehaviorConfig::default();
        assert!(config.needs_confirmation(ConfirmableAction::RootAction));
        assert!(config.needs_confirmation(ConfirmableAction::FileDeletion));
    }

    #[test]
    fn test_silent_confirms_root() {
        let config = ConfirmationBehaviorConfig::silent();
        assert!(config.needs_confirmation(ConfirmableAction::RootAction));
        assert!(!config.needs_confirmation(ConfirmableAction::FileDeletion));
    }

    #[test]
    fn test_apply_strict() {
        let mut config = ConfirmationBehaviorConfig::default();
        let result = config.apply_change("use strict confirmation");
        assert!(result.is_some());
        assert!(config.require_explicit_yes);
    }

    #[test]
    fn test_apply_remember() {
        let mut config = ConfirmationBehaviorConfig::default();
        config.apply_change("don't remember my choices");
        assert!(!config.remember_choices);
    }

    #[test]
    fn test_is_confirmation_query() {
        assert!(is_confirmation_query("Change confirmation settings"));
        assert!(is_confirmation_query("Ask before executing?"));
        assert!(!is_confirmation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = confirmation_fun_fact();
        assert!(fact.contains("1980s"));
    }
}
