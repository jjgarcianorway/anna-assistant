//! Persistent configuration for Anna.
//!
//! INVARIANT: Config is system-wide at /etc/anna/config.toml.
//! No per-user config. No home directory paths.

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

    /// v0.3.29: Teaching mode - explains why actions were taken with citations
    #[serde(default)]
    pub teaching_mode: bool,

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
    std::env::var("ANNA_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string())
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
            auto_install_helpers: true,
            ask_clarification: true,
            wiki: WikiConfig::default(),
            performance: PerformanceConfig::default(),
            ollama: OllamaConfig::default(),
            use_ralph_loop: true,
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
            "auto_install_helpers" | "auto_install" => self.auto_install_helpers = value,
            "ask_clarification" | "clarification" => self.ask_clarification = value,
            "use_embeddings" | "embeddings" => self.wiki.use_embeddings = value,
            _ => anyhow::bail!("Unknown setting: {}", key),
        }
        self.save()?;
        Ok(())
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
