//! Configuration snapshot and model mappings.

use serde::{Deserialize, Serialize};

use crate::config::AnnaConfig;

/// v0.3.21: Full config snapshot for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Auto-install helpers enabled
    pub auto_install_helpers: bool,
    /// Ask clarification enabled
    pub ask_clarification: bool,
    /// Use Ralph loop
    pub use_ralph_loop: bool,
    /// Ollama URL
    pub ollama_url: String,
    /// Ollama model
    pub ollama_model: String,
    /// Max iterations
    pub max_iterations: u32,
    /// LLM timeout (seconds)
    pub llm_timeout_secs: u64,
    /// Command timeout (seconds)
    pub command_timeout_secs: u64,
    /// Wiki cache path
    pub wiki_cache_path: String,
    /// Use embeddings
    pub use_embeddings: bool,
    /// High confidence threshold
    pub high_confidence_threshold: f32,
}

impl ConfigSnapshot {
    /// Create from AnnaConfig
    pub fn from_config(config: &AnnaConfig) -> Self {
        Self {
            debug_mode: config.debug_mode,
            auto_install_helpers: config.auto_install_helpers,
            ask_clarification: config.ask_clarification,
            use_ralph_loop: config.use_ralph_loop,
            ollama_url: config.ollama.url.clone(),
            ollama_model: config.ollama.model.clone(),
            max_iterations: config.performance.max_iterations,
            llm_timeout_secs: config.performance.llm_timeout_secs,
            command_timeout_secs: config.performance.command_timeout_secs,
            wiki_cache_path: config.wiki.cache_path.display().to_string(),
            use_embeddings: config.wiki.use_embeddings,
            high_confidence_threshold: config.performance.high_confidence_threshold,
        }
    }

    /// Create from current config
    pub fn current() -> Self {
        match AnnaConfig::load() {
            Ok(config) => Self::from_config(&config),
            Err(_) => Self::default(),
        }
    }
}

/// v0.3.21: Model role mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    /// Role name (e.g., "intent", "command", "validation", "answer")
    pub role: String,
    /// Model used for this role
    pub model: String,
    /// Whether this is the default model
    pub is_default: bool,
}

impl ModelMapping {
    /// Get default mappings (all roles use the configured model)
    pub fn defaults(model: &str) -> Vec<Self> {
        vec![
            Self {
                role: "intent".to_string(),
                model: model.to_string(),
                is_default: true,
            },
            Self {
                role: "command".to_string(),
                model: model.to_string(),
                is_default: true,
            },
            Self {
                role: "validation".to_string(),
                model: model.to_string(),
                is_default: true,
            },
            Self {
                role: "answer".to_string(),
                model: model.to_string(),
                is_default: true,
            },
            Self {
                role: "clarification".to_string(),
                model: model.to_string(),
                is_default: true,
            },
        ]
    }
}
