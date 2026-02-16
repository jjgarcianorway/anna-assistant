//! Persistent configuration for Anna.
//!
//! INVARIANT: Config is system-wide at /etc/anna/config.toml.
//! No per-user config. No home directory paths.

use crate::exposure::ExposureLevel;
use crate::paths::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Anna configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaConfig {
    /// Debug mode - shows full prompts
    #[serde(default = "default_true")]
    pub debug_mode: bool,

    /// v0.3.71: Teaching Mode - enables explanation and service desk reasoning.
    /// When true:
    /// - Explanation intents get conceptual teaching (why before how)
    /// - Service desk intents get diagnostic reasoning walkthrough
    /// - Still NO commands by default, NO guessing, NO hallucination
    /// When false (default):
    /// - Pure Observation Phase: data only, no interpretation
    #[serde(default)]
    pub teaching_mode: bool,

    /// v0.3.44: Show internal comms - fly-on-the-wall view (deprecated, use exposure_level)
    #[serde(default)]
    pub show_internal_comms: bool,

    /// v0.3.45: Exposure level - controls what internal information is shown
    /// Levels: silent, summary, dialogue, debug
    #[serde(default)]
    pub exposure_level: ExposureLevel,

    /// Auto-install helpers when needed
    #[serde(default = "default_true")]
    pub auto_install_helpers: bool,

    /// Ask for clarification on ambiguous queries
    #[serde(default = "default_true")]
    pub ask_clarification: bool,

    /// Wiki settings
    #[serde(default)]
    pub wiki: WikiConfig,

    /// Performance settings (timeouts, limits)
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Ollama settings (v0.0.895)
    #[serde(default)]
    pub ollama: OllamaConfig,

    /// v0.1.1: Use Ralph-style iteration loop (simpler, more robust)
    #[serde(default = "default_true")]
    pub use_ralph_loop: bool,

    /// v0.3.103: Multi-agent settings
    #[serde(default)]
    pub agents: AgentConfig,

    /// v0.3.103: Prediction settings
    #[serde(default)]
    pub prediction: PredictionConfig,

    /// v0.3.250: Update channel — controls which releases trigger auto-update.
    /// stable (default): tracks latest GitHub release
    /// beta: includes pre-releases
    /// pinned:<version>: never auto-update away from the specified version
    #[serde(default)]
    pub update_channel: UpdateChannel,

    /// v0.3.251: Minimum minutes since release publication before auto-installing.
    /// Prevents immediately installing a release that may be rolled back. Default: 0.
    #[serde(default)]
    pub update_delay_minutes: u32,

    /// v0.3.251: Each node adds a deterministic 0..N minute offset (derived from node_id).
    /// Prevents synchronized fleet updates from all machines installing at the same time.
    /// Capped internally at 60 minutes; values above 60 are treated as 60.
    #[serde(default)]
    pub update_stagger_minutes: u32,

    /// v0.3.251: Telegram user role configuration.
    #[serde(default)]
    pub telegram: TelegramRoleConfig,
}

/// v0.3.103: Multi-agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Enable multi-agent mode (parallel investigation)
    #[serde(default)]
    pub multi_agent_mode: bool,

    /// Enable parallel investigation for multi-domain questions
    #[serde(default)]
    pub parallel_investigation: bool,

    /// Maximum parallel agents
    #[serde(default = "default_max_parallel_agents")]
    pub max_parallel_agents: usize,

    /// Model for fast tier (simple queries)
    #[serde(default = "default_fast_model")]
    pub fast_model: String,

    /// Model for standard tier (balanced tasks)
    #[serde(default = "default_standard_model")]
    pub standard_model: String,

    /// Model for deep tier (complex debugging)
    #[serde(default = "default_deep_model")]
    pub deep_model: String,
}

fn default_max_parallel_agents() -> usize { 3 }
fn default_fast_model() -> String { "qwen2.5:7b".to_string() }
fn default_standard_model() -> String { "qwen2.5:14b".to_string() }
fn default_deep_model() -> String { "qwen2.5:32b".to_string() }

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            multi_agent_mode: false,  // v0.3.244: Off by default — produces noisy multi-specialist output
            parallel_investigation: false,  // v0.3.244: Off by default
            max_parallel_agents: default_max_parallel_agents(),
            fast_model: default_fast_model(),
            standard_model: default_standard_model(),
            deep_model: default_deep_model(),
        }
    }
}

/// v0.3.103: Prediction and alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// Enable predictive alerts
    #[serde(default)]
    pub enabled: bool,

    /// Days ahead to alert for disk space
    #[serde(default = "default_disk_alert_days")]
    pub disk_alert_days: u32,

    /// Days ahead to alert for critical disk
    #[serde(default = "default_disk_critical_days")]
    pub disk_critical_days: u32,

    /// Disk warning threshold (percentage)
    #[serde(default = "default_disk_warning_threshold")]
    pub disk_warning_threshold: f64,

    /// Disk critical threshold (percentage)
    #[serde(default = "default_disk_critical_threshold")]
    pub disk_critical_threshold: f64,
}

fn default_disk_alert_days() -> u32 { 14 }
fn default_disk_critical_days() -> u32 { 7 }
fn default_disk_warning_threshold() -> f64 { 85.0 }
fn default_disk_critical_threshold() -> f64 { 95.0 }

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // v0.3.121: Enabled by default
            disk_alert_days: default_disk_alert_days(),
            disk_critical_days: default_disk_critical_days(),
            disk_warning_threshold: default_disk_warning_threshold(),
            disk_critical_threshold: default_disk_critical_threshold(),
        }
    }
}

/// v0.0.895: Ollama configuration - centralized, no more hardcoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama API URL (default: http://127.0.0.1:11434)
    #[serde(default = "default_ollama_url")]
    pub url: String,
    /// Default model to use
    #[serde(default = "default_ollama_model")]
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: default_ollama_url(),
            model: default_ollama_model(),
        }
    }
}

fn default_ollama_url() -> String {
    std::env::var("ANNA_OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string())
}

fn default_ollama_model() -> String {
    std::env::var("ANNA_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:14b".to_string())
}

/// Performance configuration (timeouts, limits, etc.)
/// v0.0.893: Added wiki, session, and threshold settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum iterations for command discovery
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// Timeout for LLM calls in seconds
    #[serde(default = "default_llm_timeout")]
    pub llm_timeout_secs: u64,
    /// Fast timeout for simple queries in seconds
    #[serde(default = "default_fast_timeout")]
    pub fast_llm_timeout_secs: u64,
    /// Command execution timeout in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout_secs: u64,
    /// Cache TTL for answers in seconds
    #[serde(default = "default_answer_cache_ttl")]
    pub answer_cache_ttl_secs: u64,
    /// Cache TTL for command outputs in seconds
    #[serde(default = "default_command_cache_ttl")]
    pub command_cache_ttl_secs: u64,
    /// Cache TTL for static commands (uname, lscpu, etc.) in seconds
    #[serde(default = "default_static_cache_ttl")]
    pub static_command_cache_ttl_secs: u64,
    /// Wiki search timeout in seconds
    #[serde(default = "default_wiki_timeout")]
    pub wiki_search_timeout_secs: u64,
    /// Wiki circuit breaker threshold (failures before open)
    #[serde(default = "default_wiki_circuit_threshold")]
    pub wiki_circuit_threshold: u32,
    /// Wiki circuit breaker cooldown in seconds
    #[serde(default = "default_wiki_circuit_cooldown")]
    pub wiki_circuit_cooldown_secs: u64,
    /// High confidence threshold for skipping extra steps
    #[serde(default = "default_high_confidence")]
    pub high_confidence_threshold: f32,
    /// Maximum session history turns to keep
    #[serde(default = "default_max_session_history")]
    pub max_session_history: usize,
    /// v0.0.923: LLM retry settings
    /// Maximum number of retry attempts for LLM calls
    #[serde(default = "default_llm_max_retries")]
    pub llm_max_retries: u32,
    /// Base delay for exponential backoff in milliseconds
    #[serde(default = "default_llm_retry_delay_ms")]
    pub llm_retry_delay_ms: u64,
    /// LLM circuit breaker threshold (failures before open)
    #[serde(default = "default_llm_circuit_threshold")]
    pub llm_circuit_threshold: u32,
    /// LLM circuit breaker cooldown in seconds
    #[serde(default = "default_llm_circuit_cooldown")]
    pub llm_circuit_cooldown_secs: u64,
}

fn default_max_iterations() -> u32 { 3 }
fn default_llm_timeout() -> u64 { 120 }
fn default_fast_timeout() -> u64 { 30 }
fn default_command_timeout() -> u64 { 10 }
fn default_answer_cache_ttl() -> u64 { 600 }  // v0.0.924: Increased from 300s to 600s
fn default_command_cache_ttl() -> u64 { 60 }
fn default_static_cache_ttl() -> u64 { 300 }
fn default_wiki_timeout() -> u64 { 5 }
fn default_wiki_circuit_threshold() -> u32 { 4 } // v0.0.895: Increased from 2 (too aggressive)
fn default_wiki_circuit_cooldown() -> u64 { 60 }
fn default_high_confidence() -> f32 { 0.85 }
fn default_max_session_history() -> usize { 20 }
// v0.0.990: Increased retries to 3, reduced initial delay to 300ms
fn default_llm_max_retries() -> u32 { 3 }
fn default_llm_retry_delay_ms() -> u64 { 300 }
// v0.0.990: Increased threshold (5), decreased cooldown (15s) for better resilience
fn default_llm_circuit_threshold() -> u32 { 5 }
fn default_llm_circuit_cooldown() -> u64 { 15 }

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_iterations: default_max_iterations(),
            llm_timeout_secs: default_llm_timeout(),
            fast_llm_timeout_secs: default_fast_timeout(),
            command_timeout_secs: default_command_timeout(),
            answer_cache_ttl_secs: default_answer_cache_ttl(),
            command_cache_ttl_secs: default_command_cache_ttl(),
            static_command_cache_ttl_secs: default_static_cache_ttl(),
            wiki_search_timeout_secs: default_wiki_timeout(),
            wiki_circuit_threshold: default_wiki_circuit_threshold(),
            wiki_circuit_cooldown_secs: default_wiki_circuit_cooldown(),
            high_confidence_threshold: default_high_confidence(),
            max_session_history: default_max_session_history(),
            llm_max_retries: default_llm_max_retries(),
            llm_retry_delay_ms: default_llm_retry_delay_ms(),
            llm_circuit_threshold: default_llm_circuit_threshold(),
            llm_circuit_cooldown_secs: default_llm_circuit_cooldown(),
        }
    }
}

fn default_true() -> bool {
    true
}

pub use crate::update_channel::UpdateChannel;

/// v0.3.251: Role of a Telegram user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramUserRole {
    /// Full access (default for explicitly listed admins).
    Admin,
    /// Query/probe only — cannot trigger system actions.
    ReadOnly,
}

/// v0.3.251: Telegram role configuration (loaded from [telegram] in config.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramRoleConfig {
    /// Map of Telegram user_id → role. Unknown user IDs are silently ignored (ghosted).
    #[serde(default)]
    pub users: std::collections::HashMap<u64, TelegramUserRole>,
}

/// Wiki configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiConfig {
    /// Path to local wiki cache
    #[serde(default = "default_wiki_path")]
    pub cache_path: PathBuf,

    /// Use embeddings for semantic search
    #[serde(default = "default_true")]
    pub use_embeddings: bool,

    /// Last sync time
    pub last_sync: Option<String>,
}

fn default_wiki_path() -> PathBuf {
    anna_data_dir().join("wiki")
}

impl Default for AnnaConfig {
    fn default() -> Self {
        Self {
            debug_mode: true,
            teaching_mode: false, // v0.3.29: Off by default
            show_internal_comms: false, // v0.3.44: Deprecated, use exposure_level
            exposure_level: ExposureLevel::Silent, // v0.3.45: Silent by default
            auto_install_helpers: true,
            ask_clarification: true,
            wiki: WikiConfig::default(),
            performance: PerformanceConfig::default(),
            ollama: OllamaConfig::default(),
            use_ralph_loop: true,
            agents: AgentConfig::default(),
            prediction: PredictionConfig::default(),
            update_channel: UpdateChannel::default(),
            update_delay_minutes: 0,
            update_stagger_minutes: 0,
            telegram: TelegramRoleConfig::default(),
        }
    }
}

impl AnnaConfig {
    /// Load config from disk, or create default
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AnnaConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AnnaConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Update a setting by name (for natural language control)
    pub fn set(&mut self, key: &str, value: bool) -> Result<()> {
        match key {
            "debug_mode" | "debug" => self.debug_mode = value,
            "show_internal_comms" | "internal_comms" => {
                self.show_internal_comms = value;
                // Migrate to exposure_level
                self.exposure_level = if value {
                    ExposureLevel::Dialogue
                } else {
                    ExposureLevel::Silent
                };
            }
            "teaching_mode" | "teaching" => self.teaching_mode = value,
            "auto_install_helpers" | "auto_install" => self.auto_install_helpers = value,
            "ask_clarification" | "clarification" => self.ask_clarification = value,
            "use_embeddings" | "embeddings" => self.wiki.use_embeddings = value,
            _ => anyhow::bail!("Unknown setting: {}", key),
        }
        self.save()?;
        Ok(())
    }

    /// Set exposure level by name.
    pub fn set_exposure(&mut self, level: &str) -> Result<()> {
        self.exposure_level = ExposureLevel::from_str(level)
            .ok_or_else(|| anyhow::anyhow!("Unknown exposure level: {}", level))?;
        // Sync deprecated field for backward compatibility
        self.show_internal_comms = self.exposure_level >= ExposureLevel::Dialogue;
        self.save()
    }

    /// Get effective exposure level (handles migration from show_internal_comms).
    pub fn effective_exposure_level(&self) -> ExposureLevel {
        // If old show_internal_comms is set but new exposure_level is default,
        // migrate the setting
        if self.show_internal_comms && self.exposure_level == ExposureLevel::Silent {
            ExposureLevel::Dialogue
        } else {
            self.exposure_level
        }
    }
}

/// Get Anna data directory (/var/lib/anna)
pub fn anna_data_dir() -> PathBuf {
    paths().data_dir.clone()
}

/// Get config file path (/etc/anna/config.toml)
pub fn config_path() -> PathBuf {
    paths().config_file()
}

/// v0.0.895: Get Ollama URL from config or environment
pub fn get_ollama_url() -> String {
    AnnaConfig::load()
        .map(|c| c.ollama.url)
        .unwrap_or_else(|_| default_ollama_url())
}

/// v0.0.895: Get default Ollama model from config or environment
pub fn get_ollama_model() -> String {
    AnnaConfig::load()
        .map(|c| c.ollama.model)
        .unwrap_or_else(|_| default_ollama_model())
}
