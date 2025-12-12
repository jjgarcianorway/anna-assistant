//! Model configuration (v0.0.434).
//!
//! User-configurable options for model behavior.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Auto-install policy for models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoInstallPolicy {
    /// Always install required models automatically.
    Always,
    /// Never install models automatically.
    Never,
    /// Ask for each model before installing.
    AskPerModel,
}

impl AutoInstallPolicy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::AskPerModel => "ask-per-model",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            "ask-per-model" | "ask" => Some(Self::AskPerModel),
            _ => None,
        }
    }
}

impl Default for AutoInstallPolicy {
    fn default() -> Self {
        Self::AskPerModel
    }
}

/// Prefer small models setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferSmallSetting {
    /// Always prefer small models.
    Yes,
    /// Never prefer small (prefer best for tier).
    No,
    /// Automatic based on hardware.
    Auto,
}

impl PreferSmallSetting {
    /// Resolve to boolean based on hardware.
    pub fn resolve(&self, ram_gb: f32) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
            Self::Auto => ram_gb < 16.0, // Prefer small if less than 16GB
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "yes" | "true" | "1" => Some(Self::Yes),
            "no" | "false" | "0" => Some(Self::No),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

impl Default for PreferSmallSetting {
    fn default() -> Self {
        Self::Auto
    }
}

/// Per-model decision record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDecision {
    /// Model name.
    pub model: String,
    /// Whether user approved installation.
    pub approved: bool,
    /// When the decision was made.
    pub decided_at: String,
    /// Optional reason.
    pub reason: Option<String>,
}

/// Model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Auto-install policy.
    pub auto_install: AutoInstallPolicy,
    /// Maximum disk usage for models in GB.
    pub max_model_disk_gb: u32,
    /// Prefer small models setting.
    pub prefer_small: PreferSmallSetting,
    /// Per-model decisions (for ask-per-model policy).
    pub model_decisions: HashMap<String, ModelDecision>,
    /// Whether to verify models on startup.
    pub verify_on_startup: bool,
    /// Custom Ollama URL (if not default).
    pub ollama_url: Option<String>,
}

impl ModelConfig {
    /// Create with defaults.
    pub fn new() -> Self {
        Self {
            auto_install: AutoInstallPolicy::AskPerModel,
            max_model_disk_gb: super::DEFAULT_MAX_MODEL_DISK_GB,
            prefer_small: PreferSmallSetting::Auto,
            model_decisions: HashMap::new(),
            verify_on_startup: true,
            ollama_url: None,
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

    /// Check if a model can be installed.
    pub fn can_install(&self, model: &str) -> InstallDecision {
        match self.auto_install {
            AutoInstallPolicy::Always => InstallDecision::Allowed,
            AutoInstallPolicy::Never => {
                InstallDecision::Denied("Auto-install disabled".to_string())
            }
            AutoInstallPolicy::AskPerModel => {
                if let Some(decision) = self.model_decisions.get(model) {
                    if decision.approved {
                        InstallDecision::Allowed
                    } else {
                        InstallDecision::Denied("Previously declined".to_string())
                    }
                } else {
                    InstallDecision::NeedsApproval
                }
            }
        }
    }

    /// Check if total disk would exceed limit.
    pub fn would_exceed_disk_limit(&self, current_gb: u32, new_model_gb: u32) -> bool {
        current_gb + new_model_gb > self.max_model_disk_gb
    }

    /// Record a user's decision about a model.
    pub fn record_decision(&mut self, model: &str, approved: bool, reason: Option<&str>) {
        self.model_decisions.insert(
            model.to_string(),
            ModelDecision {
                model: model.to_string(),
                approved,
                decided_at: timestamp_now(),
                reason: reason.map(|s| s.to_string()),
            },
        );
    }

    /// Clear a decision for a model.
    pub fn clear_decision(&mut self, model: &str) {
        self.model_decisions.remove(model);
    }

    /// Get Ollama URL (default or custom).
    pub fn ollama_url(&self) -> &str {
        self.ollama_url
            .as_deref()
            .unwrap_or("http://localhost:11434")
    }

    /// Format for display.
    pub fn format_summary(&self) -> String {
        format!(
            "auto_install={}, max_disk={}GB, prefer_small={:?}",
            self.auto_install.label(),
            self.max_model_disk_gb,
            self.prefer_small
        )
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of checking if install is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallDecision {
    /// Installation is allowed.
    Allowed,
    /// Installation is denied with reason.
    Denied(String),
    /// User approval is needed.
    NeedsApproval,
}

impl InstallDecision {
    /// Whether installation can proceed.
    pub fn can_proceed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Install request for user prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    /// Model to install.
    pub model: String,
    /// Estimated size in GB.
    pub size_gb: u32,
    /// Why this model is needed.
    pub reason: String,
    /// Role it will fill.
    pub role: String,
}

impl InstallRequest {
    /// Format as natural language prompt.
    pub fn format_prompt(&self) -> String {
        format!(
            "I would like to install {} (about {} GB) for {}. {}. Install now? [y/N]",
            self.model, self.size_gb, self.role, self.reason
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
    fn test_auto_install_policy() {
        assert_eq!(
            AutoInstallPolicy::from_str("always"),
            Some(AutoInstallPolicy::Always)
        );
        assert_eq!(
            AutoInstallPolicy::from_str("never"),
            Some(AutoInstallPolicy::Never)
        );
        assert_eq!(
            AutoInstallPolicy::from_str("ask"),
            Some(AutoInstallPolicy::AskPerModel)
        );
    }

    #[test]
    fn test_prefer_small_resolve() {
        assert!(PreferSmallSetting::Yes.resolve(64.0));
        assert!(!PreferSmallSetting::No.resolve(4.0));
        assert!(PreferSmallSetting::Auto.resolve(8.0)); // < 16GB
        assert!(!PreferSmallSetting::Auto.resolve(32.0)); // >= 16GB
    }

    #[test]
    fn test_can_install_always() {
        let mut config = ModelConfig::new();
        config.auto_install = AutoInstallPolicy::Always;

        assert_eq!(config.can_install("any_model"), InstallDecision::Allowed);
    }

    #[test]
    fn test_can_install_never() {
        let mut config = ModelConfig::new();
        config.auto_install = AutoInstallPolicy::Never;

        let decision = config.can_install("any_model");
        assert!(!decision.can_proceed());
    }

    #[test]
    fn test_can_install_ask() {
        let mut config = ModelConfig::new();
        config.auto_install = AutoInstallPolicy::AskPerModel;

        // No decision yet
        assert_eq!(
            config.can_install("new_model"),
            InstallDecision::NeedsApproval
        );

        // After approval
        config.record_decision("new_model", true, None);
        assert_eq!(config.can_install("new_model"), InstallDecision::Allowed);

        // After denial
        config.record_decision("denied_model", false, Some("Too large"));
        assert!(!config.can_install("denied_model").can_proceed());
    }

    #[test]
    fn test_disk_limit() {
        let config = ModelConfig::new(); // Default 25GB

        assert!(!config.would_exceed_disk_limit(10, 10)); // 20 < 25
        assert!(config.would_exceed_disk_limit(20, 10)); // 30 > 25
    }

    #[test]
    fn test_install_request_prompt() {
        let request = InstallRequest {
            model: "qwen2.5:7b-instruct".to_string(),
            size_gb: 5,
            reason: "Needed for complex tickets".to_string(),
            role: "senior specialist".to_string(),
        };

        let prompt = request.format_prompt();
        assert!(prompt.contains("qwen2.5:7b-instruct"));
        assert!(prompt.contains("5 GB"));
        assert!(prompt.contains("[y/N]"));
    }

    #[test]
    fn test_serialization() {
        let mut config = ModelConfig::new();
        config.record_decision("test_model", true, Some("Approved"));

        let json = serde_json::to_string(&config).unwrap();
        let restored: ModelConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.auto_install, config.auto_install);
        assert!(restored.model_decisions.contains_key("test_model"));
    }
}
