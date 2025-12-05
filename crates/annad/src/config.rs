//! Configuration management for annad.
//!
//! Loads settings from /etc/anna/config.toml or uses defaults.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Config file path
pub const CONFIG_PATH: &str = "/etc/anna/config.toml";

/// Default config file path for fallback
pub const DEFAULT_CONFIG_PATH: &str = "/var/lib/anna/config.toml";

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Model for translator (query classification) - fast, small
    #[serde(default = "default_translator_model")]
    pub translator_model: String,

    /// Model for specialist (domain expert) - capable, accurate
    #[serde(default = "default_specialist_model")]
    pub specialist_model: String,

    /// Model for supervisor (validation) - same as translator
    #[serde(default = "default_supervisor_model")]
    pub supervisor_model: String,

    /// Translator timeout in seconds
    #[serde(default = "default_translator_timeout")]
    pub translator_timeout_secs: u64,

    /// Specialist timeout in seconds (v0.0.30: reduced from 12 to 8 with fallback)
    #[serde(default = "default_specialist_timeout")]
    pub specialist_timeout_secs: u64,

    /// Maximum specialist prompt size in bytes (v0.0.30: cap to prevent slow inference)
    #[serde(default = "default_max_specialist_prompt")]
    pub max_specialist_prompt_bytes: usize,

    /// Supervisor timeout in seconds
    #[serde(default = "default_supervisor_timeout")]
    pub supervisor_timeout_secs: u64,

    /// Per-probe timeout in seconds
    #[serde(default = "default_probe_timeout")]
    pub probe_timeout_secs: u64,

    /// Total probe stage timeout
    #[serde(default = "default_probes_total_timeout")]
    pub probes_total_timeout_secs: u64,
}

fn default_translator_model() -> String {
    // v0.0.32: Use smallest fast model for translator to avoid timeouts
    "qwen2.5:0.5b-instruct".to_string()
}

fn default_specialist_model() -> String {
    "qwen2.5:7b-instruct".to_string()
}

fn default_supervisor_model() -> String {
    // v0.0.32: Use same small model as translator for speed
    "qwen2.5:0.5b-instruct".to_string()
}

fn default_translator_timeout() -> u64 {
    2 // v0.0.32: reduced from 4 - fast translator should be quick
}

fn default_specialist_timeout() -> u64 {
    6 // v0.0.32: reduced from 8 - bias toward deterministic fallback
}

fn default_max_specialist_prompt() -> usize {
    16_384 // 16KB cap to prevent slow inference
}

fn default_supervisor_timeout() -> u64 {
    6
}

fn default_probe_timeout() -> u64 {
    4
}

fn default_probes_total_timeout() -> u64 {
    10
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            translator_model: default_translator_model(),
            specialist_model: default_specialist_model(),
            supervisor_model: default_supervisor_model(),
            translator_timeout_secs: default_translator_timeout(),
            specialist_timeout_secs: default_specialist_timeout(),
            max_specialist_prompt_bytes: default_max_specialist_prompt(),
            supervisor_timeout_secs: default_supervisor_timeout(),
            probe_timeout_secs: default_probe_timeout(),
            probes_total_timeout_secs: default_probes_total_timeout(),
        }
    }
}

/// Stage budget configuration (METER phase)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Translator stage budget in milliseconds
    #[serde(default = "default_translator_budget")]
    pub translator_ms: u64,

    /// Probes stage budget in milliseconds
    #[serde(default = "default_probes_budget")]
    pub probes_ms: u64,

    /// Specialist stage budget in milliseconds
    #[serde(default = "default_specialist_budget")]
    pub specialist_ms: u64,

    /// Supervisor stage budget in milliseconds
    #[serde(default = "default_supervisor_budget")]
    pub supervisor_ms: u64,

    /// Total request budget in milliseconds
    #[serde(default = "default_total_budget")]
    pub total_ms: u64,

    /// Margin for orchestration overhead in milliseconds
    #[serde(default = "default_margin_budget")]
    pub margin_ms: u64,
}

fn default_translator_budget() -> u64 {
    1_500 // v0.0.32: 1.5s - fast translator with 0.5b model
}

fn default_probes_budget() -> u64 {
    8_000 // v0.0.32: 8s - reasonable probe window
}

fn default_specialist_budget() -> u64 {
    6_000 // v0.0.32: 6s - bias toward deterministic fallback
}

fn default_supervisor_budget() -> u64 {
    4_000 // v0.0.32: 4s - review gate
}

fn default_total_budget() -> u64 {
    18_000 // v0.0.32: 18s total - faster response bias
}

fn default_margin_budget() -> u64 {
    1_000 // 1 second
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            translator_ms: default_translator_budget(),
            probes_ms: default_probes_budget(),
            specialist_ms: default_specialist_budget(),
            supervisor_ms: default_supervisor_budget(),
            total_ms: default_total_budget(),
            margin_ms: default_margin_budget(),
        }
    }
}

impl BudgetConfig {
    /// Convert to StageBudget for use with BudgetEnforcer
    pub fn to_stage_budget(&self) -> anna_shared::budget::StageBudget {
        anna_shared::budget::StageBudget {
            translator_ms: self.translator_ms,
            probes_ms: self.probes_ms,
            specialist_ms: self.specialist_ms,
            supervisor_ms: self.supervisor_ms,
            total_ms: self.total_ms,
            margin_ms: self.margin_ms,
        }
    }
}

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Debug mode shows detailed pipeline output
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,

    /// Auto-update enabled
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,

    /// Update check interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,

    /// Global request timeout in seconds (entire pipeline)
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
}

fn default_debug_mode() -> bool {
    true // Debug ON by default
}

fn default_auto_update() -> bool {
    true
}

fn default_update_interval() -> u64 {
    600
}

fn default_request_timeout() -> u64 {
    20 // 20 second total budget
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            debug_mode: default_debug_mode(),
            auto_update: default_auto_update(),
            update_interval: default_update_interval(),
            request_timeout_secs: default_request_timeout(),
        }
    }
}

/// Full daemon configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,

    #[serde(default)]
    pub llm: LlmConfig,

    /// Stage budget configuration (METER phase)
    #[serde(default)]
    pub budget: BudgetConfig,
}

impl Config {
    /// Get debug mode setting
    pub fn debug_mode(&self) -> bool {
        self.daemon.debug_mode
    }

    /// Load config from file, or return defaults
    pub fn load() -> Self {
        Self::load_from_path(CONFIG_PATH)
            .or_else(|_| Self::load_from_path(DEFAULT_CONFIG_PATH))
            .unwrap_or_else(|e| {
                warn!("Config not found, using defaults: {}", e);
                Config::default()
            })
    }

    /// Load config from specific path
    fn load_from_path(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        info!("Loaded config from {}", path);
        Ok(config)
    }

    /// Save default config to path (for init)
    #[allow(dead_code)]
    pub fn save_default(path: &str) -> Result<()> {
        let config = Config::default();
        let content = toml::to_string_pretty(&config)?;
        let parent = Path::new(path).parent().unwrap();
        fs::create_dir_all(parent)?;
        fs::write(path, content)?;
        info!("Saved default config to {}", path);
        Ok(())
    }

    /// Get list of unique models needed (for pulling)
    pub fn required_models(&self) -> Vec<String> {
        let mut models = vec![
            self.llm.translator_model.clone(),
            self.llm.specialist_model.clone(),
        ];
        // Add supervisor only if different
        if self.llm.supervisor_model != self.llm.translator_model
            && self.llm.supervisor_model != self.llm.specialist_model
        {
            models.push(self.llm.supervisor_model.clone());
        }
        models.sort();
        models.dedup();
        models
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        // v0.0.32: fast translator with 0.5b model
        assert_eq!(config.llm.translator_model, "qwen2.5:0.5b-instruct");
        assert_eq!(config.llm.specialist_model, "qwen2.5:7b-instruct");
        assert_eq!(config.llm.translator_timeout_secs, 2);
    }

    #[test]
    fn test_required_models_dedup() {
        let config = Config::default();
        // translator and supervisor are the same by default
        let models = config.required_models();
        assert_eq!(models.len(), 2); // translator/supervisor (same) + specialist
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[llm]
translator_model = "custom:1b"
specialist_model = "custom:7b"
translator_timeout_secs = 5
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.llm.translator_model, "custom:1b");
        assert_eq!(config.llm.specialist_model, "custom:7b");
        assert_eq!(config.llm.translator_timeout_secs, 5);
        // Defaults for missing fields (v0.0.32: specialist timeout reduced to 6)
        assert_eq!(config.llm.specialist_timeout_secs, 6);
    }
}
