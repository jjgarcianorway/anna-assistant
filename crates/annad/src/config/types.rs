//! Configuration type definitions for annad.
//!
//! Defines all configuration structures used by the daemon.

use serde::{Deserialize, Serialize};

// Re-export ModelRegistryConfig for backwards compatibility
pub use crate::config_registry::ModelRegistryConfig;

/// Config file path
pub const CONFIG_PATH: &str = "/etc/anna/config.toml";

/// Default config file path for fallback
pub const DEFAULT_CONFIG_PATH: &str = "/var/lib/anna/config.toml";

/// LLM configuration
/// v0.0.277: Added junior_model and senior_model for tiered expertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Model for translator (query classification + formatting) - smallest, fastest
    #[serde(default = "super::defaults::translator_model")]
    pub translator_model: String,

    /// Model for junior specialist (regular queries) - mid-size, capable
    #[serde(default = "super::defaults::junior_model")]
    pub junior_model: String,

    /// Model for senior specialist (complex/escalated queries) - largest, smartest
    #[serde(default = "super::defaults::senior_model")]
    pub senior_model: String,

    /// Legacy: Model for specialist - maps to junior_model
    #[serde(default = "super::defaults::specialist_model")]
    pub specialist_model: String,

    /// Model for supervisor (validation) - same as translator
    #[serde(default = "super::defaults::supervisor_model")]
    pub supervisor_model: String,

    /// Translator timeout in seconds
    #[serde(default = "super::defaults::translator_timeout")]
    pub translator_timeout_secs: u64,

    /// Specialist timeout in seconds (v0.0.30: reduced from 12 to 8 with fallback)
    #[serde(default = "super::defaults::specialist_timeout")]
    pub specialist_timeout_secs: u64,

    /// Maximum specialist prompt size in bytes (v0.0.30: cap to prevent slow inference)
    #[serde(default = "super::defaults::max_specialist_prompt")]
    pub max_specialist_prompt_bytes: usize,

    /// Supervisor timeout in seconds
    #[serde(default = "super::defaults::supervisor_timeout")]
    pub supervisor_timeout_secs: u64,

    /// Per-probe timeout in seconds
    #[serde(default = "super::defaults::probe_timeout")]
    pub probe_timeout_secs: u64,

    /// Total probe stage timeout
    #[serde(default = "super::defaults::probes_total_timeout")]
    pub probes_total_timeout_secs: u64,
    // v0.0.406: Removed use_json_specialist - JSON is now the only architecture
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            translator_model: super::defaults::translator_model(),
            junior_model: super::defaults::junior_model(),
            senior_model: super::defaults::senior_model(),
            specialist_model: super::defaults::specialist_model(),
            supervisor_model: super::defaults::supervisor_model(),
            translator_timeout_secs: super::defaults::translator_timeout(),
            specialist_timeout_secs: super::defaults::specialist_timeout(),
            max_specialist_prompt_bytes: super::defaults::max_specialist_prompt(),
            supervisor_timeout_secs: super::defaults::supervisor_timeout(),
            probe_timeout_secs: super::defaults::probe_timeout(),
            probes_total_timeout_secs: super::defaults::probes_total_timeout(),
        }
    }
}

/// Stage budget configuration (METER phase)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Translator stage budget in milliseconds
    #[serde(default = "super::defaults::translator_budget")]
    pub translator_ms: u64,

    /// Probes stage budget in milliseconds
    #[serde(default = "super::defaults::probes_budget")]
    pub probes_ms: u64,

    /// Specialist stage budget in milliseconds
    #[serde(default = "super::defaults::specialist_budget")]
    pub specialist_ms: u64,

    /// Supervisor stage budget in milliseconds
    #[serde(default = "super::defaults::supervisor_budget")]
    pub supervisor_ms: u64,

    /// Total request budget in milliseconds
    #[serde(default = "super::defaults::total_budget")]
    pub total_ms: u64,

    /// Margin for orchestration overhead in milliseconds
    #[serde(default = "super::defaults::margin_budget")]
    pub margin_ms: u64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            translator_ms: super::defaults::translator_budget(),
            probes_ms: super::defaults::probes_budget(),
            specialist_ms: super::defaults::specialist_budget(),
            supervisor_ms: super::defaults::supervisor_budget(),
            total_ms: super::defaults::total_budget(),
            margin_ms: super::defaults::margin_budget(),
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
    #[serde(default = "super::defaults::debug_mode")]
    pub debug_mode: bool,

    /// Auto-update enabled
    #[serde(default = "super::defaults::auto_update")]
    pub auto_update: bool,

    /// Update check interval in seconds
    #[serde(default = "super::defaults::update_interval")]
    pub update_interval: u64,

    /// Global request timeout in seconds (entire pipeline)
    #[serde(default = "super::defaults::request_timeout")]
    pub request_timeout_secs: u64,

    /// Snapshot max age in seconds before considered stale (v0.0.36)
    #[serde(default = "super::defaults::snapshot_max_age")]
    pub snapshot_max_age_secs: u64,

    /// Fast path enabled (v0.0.39)
    #[serde(default = "super::defaults::fast_path_enabled")]
    pub fast_path_enabled: bool,

    /// Fast path fallback on translator timeout (v0.0.39)
    #[serde(default = "super::defaults::fast_path_fallback")]
    pub fast_path_fallback_on_timeout: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            debug_mode: super::defaults::debug_mode(),
            auto_update: super::defaults::auto_update(),
            update_interval: super::defaults::update_interval(),
            request_timeout_secs: super::defaults::request_timeout(),
            snapshot_max_age_secs: super::defaults::snapshot_max_age(),
            fast_path_enabled: super::defaults::fast_path_enabled(),
            fast_path_fallback_on_timeout: super::defaults::fast_path_fallback(),
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

    /// Model registry (v0.0.76)
    #[serde(default)]
    pub model_registry: ModelRegistryConfig,
}
