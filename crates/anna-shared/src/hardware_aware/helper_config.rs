//! Helper configuration (v0.0.434).
//!
//! User-configurable options for helper tool behavior.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Auto-install policy for helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelperInstallPolicy {
    /// Always install requested helpers automatically.
    Always,
    /// Never install helpers automatically.
    Never,
    /// Ask for each helper before installing.
    AskPerHelper,
}

impl HelperInstallPolicy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::AskPerHelper => "ask-per-helper",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            "ask-per-helper" | "ask" => Some(Self::AskPerHelper),
            _ => None,
        }
    }
}

impl Default for HelperInstallPolicy {
    fn default() -> Self {
        Self::AskPerHelper
    }
}

/// Per-helper decision record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperDecision {
    /// Helper ID.
    pub helper_id: String,
    /// Whether user approved installation.
    pub approved: bool,
    /// When the decision was made.
    pub decided_at: String,
    /// Optional reason.
    pub reason: Option<String>,
}

/// Helper configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperConfig {
    /// Auto-install policy.
    pub auto_install: HelperInstallPolicy,
    /// Whether to remove Anna-installed helpers on uninstall.
    pub remove_on_uninstall: bool,
    /// Per-helper decisions (for ask-per-helper policy).
    pub helper_decisions: HashMap<String, HelperDecision>,
    /// Helpers to never suggest (user blocked).
    pub blocked_helpers: Vec<String>,
    /// Maximum number of helpers Anna can install.
    pub max_helpers: u32,
}

impl HelperConfig {
    /// Create with defaults.
    pub fn new() -> Self {
        Self {
            auto_install: HelperInstallPolicy::AskPerHelper,
            remove_on_uninstall: true,
            helper_decisions: HashMap::new(),
            blocked_helpers: Vec::new(),
            max_helpers: 10,
        }
    }

    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Check if a helper can be installed.
    pub fn can_install(&self, helper_id: &str) -> HelperInstallDecision {
        // Check if blocked
        if self.blocked_helpers.contains(&helper_id.to_string()) {
            return HelperInstallDecision::Blocked;
        }

        match self.auto_install {
            HelperInstallPolicy::Always => HelperInstallDecision::Allowed,
            HelperInstallPolicy::Never => {
                HelperInstallDecision::Denied("Helper auto-install disabled".to_string())
            }
            HelperInstallPolicy::AskPerHelper => {
                if let Some(decision) = self.helper_decisions.get(helper_id) {
                    if decision.approved {
                        HelperInstallDecision::Allowed
                    } else {
                        HelperInstallDecision::Denied("Previously declined".to_string())
                    }
                } else {
                    HelperInstallDecision::NeedsApproval
                }
            }
        }
    }

    /// Record a user's decision about a helper.
    pub fn record_decision(&mut self, helper_id: &str, approved: bool, reason: Option<&str>) {
        self.helper_decisions.insert(
            helper_id.to_string(),
            HelperDecision {
                helper_id: helper_id.to_string(),
                approved,
                decided_at: timestamp_now(),
                reason: reason.map(|s| s.to_string()),
            },
        );
    }

    /// Clear a decision for a helper.
    pub fn clear_decision(&mut self, helper_id: &str) {
        self.helper_decisions.remove(helper_id);
    }

    /// Block a helper from being suggested.
    pub fn block_helper(&mut self, helper_id: &str) {
        if !self.blocked_helpers.contains(&helper_id.to_string()) {
            self.blocked_helpers.push(helper_id.to_string());
        }
    }

    /// Unblock a helper.
    pub fn unblock_helper(&mut self, helper_id: &str) {
        self.blocked_helpers.retain(|h| h != helper_id);
    }

    /// Format for display.
    pub fn format_summary(&self) -> String {
        format!(
            "auto_install={}, remove_on_uninstall={}",
            self.auto_install.label(),
            self.remove_on_uninstall
        )
    }
}

impl Default for HelperConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of checking if helper install is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperInstallDecision {
    /// Installation is allowed.
    Allowed,
    /// Installation is denied with reason.
    Denied(String),
    /// User approval is needed.
    NeedsApproval,
    /// Helper is blocked.
    Blocked,
}

impl HelperInstallDecision {
    /// Whether installation can proceed.
    pub fn can_proceed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Install request for user prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperInstallRequest {
    /// Helper ID.
    pub helper_id: String,
    /// Helper name.
    pub name: String,
    /// Purpose.
    pub purpose: String,
    /// Why it's being requested.
    pub reason: String,
    /// Packages to install.
    pub packages: Vec<String>,
}

impl HelperInstallRequest {
    /// Format as natural language prompt.
    pub fn format_prompt(&self) -> String {
        format!(
            "I would like to install {} to {}. {}. This will install: {}. Install now? [y/N]",
            self.name,
            self.purpose.to_lowercase(),
            self.reason,
            self.packages.join(", ")
        )
    }
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_parsing() {
        assert_eq!(
            HelperInstallPolicy::from_str("always"),
            Some(HelperInstallPolicy::Always)
        );
        assert_eq!(
            HelperInstallPolicy::from_str("never"),
            Some(HelperInstallPolicy::Never)
        );
        assert_eq!(
            HelperInstallPolicy::from_str("ask"),
            Some(HelperInstallPolicy::AskPerHelper)
        );
    }

    #[test]
    fn test_can_install_always() {
        let mut config = HelperConfig::new();
        config.auto_install = HelperInstallPolicy::Always;

        assert_eq!(
            config.can_install("lm_sensors"),
            HelperInstallDecision::Allowed
        );
    }

    #[test]
    fn test_can_install_never() {
        let mut config = HelperConfig::new();
        config.auto_install = HelperInstallPolicy::Never;

        assert!(!config.can_install("lm_sensors").can_proceed());
    }

    #[test]
    fn test_can_install_ask() {
        let mut config = HelperConfig::new();
        config.auto_install = HelperInstallPolicy::AskPerHelper;

        // No decision yet
        assert_eq!(
            config.can_install("lm_sensors"),
            HelperInstallDecision::NeedsApproval
        );

        // After approval
        config.record_decision("lm_sensors", true, None);
        assert_eq!(
            config.can_install("lm_sensors"),
            HelperInstallDecision::Allowed
        );
    }

    #[test]
    fn test_blocked_helpers() {
        let mut config = HelperConfig::new();
        config.auto_install = HelperInstallPolicy::Always;

        config.block_helper("lm_sensors");
        assert_eq!(
            config.can_install("lm_sensors"),
            HelperInstallDecision::Blocked
        );

        config.unblock_helper("lm_sensors");
        assert_eq!(
            config.can_install("lm_sensors"),
            HelperInstallDecision::Allowed
        );
    }

    #[test]
    fn test_install_request_prompt() {
        let request = HelperInstallRequest {
            helper_id: "lm_sensors".to_string(),
            name: "lm-sensors".to_string(),
            purpose: "Read CPU temperatures".to_string(),
            reason: "Needed to diagnose overheating".to_string(),
            packages: vec!["lm_sensors".to_string()],
        };

        let prompt = request.format_prompt();
        assert!(prompt.contains("lm-sensors"));
        assert!(prompt.contains("[y/N]"));
    }
}
