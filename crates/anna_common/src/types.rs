//! Core types for Anna v0.0.1

use serde::{Deserialize, Serialize};

/// Cache policy for probes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CachePolicy {
    /// Never expires (hardware info)
    Static,
    /// Expires slowly (configs, packages)
    Slow,
    /// Expires quickly (memory, CPU load)
    Volatile,
}

impl CachePolicy {
    /// Get TTL in seconds
    pub fn ttl_seconds(&self) -> u64 {
        match self {
            CachePolicy::Static => 0,    // Infinite
            CachePolicy::Slow => 3600,   // 1 hour
            CachePolicy::Volatile => 5,  // 5 seconds
        }
    }
}

/// Probe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDefinition {
    pub id: String,
    pub cmd: Vec<String>,
    pub parser: String,
    pub cache_policy: CachePolicy,
    #[serde(default)]
    pub ttl: u64,
}

/// Probe execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub id: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub cached: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// LLM-B verdict on LLM-A's reasoning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Approved,
    Revise,
    NotPossible,
}

/// LLM-B response to LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertResponse {
    pub verdict: Verdict,
    pub explanation: String,
    #[serde(default)]
    pub required_probes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_reasoning: Option<String>,
    pub confidence: f64,
}

/// LLM model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    pub orchestrator: String,  // LLM-A
    pub expert: String,        // LLM-B
}

impl Default for ModelSelection {
    fn default() -> Self {
        Self {
            orchestrator: "llama3.2:3b".to_string(),
            expert: "qwen2.5:7b".to_string(),
        }
    }
}

/// Hardware detection for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub ram_gb: u64,
    pub cpu_cores: usize,
    pub has_gpu: bool,
    pub vram_gb: Option<u64>,
}

impl HardwareInfo {
    /// Select appropriate models based on hardware
    pub fn select_models(&self) -> ModelSelection {
        let orchestrator = if self.has_gpu {
            "mistral-nemo-instruct".to_string()
        } else if self.cpu_cores >= 8 {
            "qwen2.5:3b".to_string()
        } else {
            "llama3.2:3b".to_string()
        };

        let expert = if self.ram_gb >= 32 && self.has_gpu {
            "qwen2.5:32b".to_string()
        } else if self.ram_gb >= 16 {
            "qwen2.5:14b".to_string()
        } else {
            "qwen2.5:7b".to_string()
        };

        ModelSelection { orchestrator, expert }
    }
}

/// Anna configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaConfig {
    pub version: String,
    pub models: ModelSelection,
    pub daemon_socket: String,
    pub ollama_url: String,
}

impl Default for AnnaConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            models: ModelSelection::default(),
            daemon_socket: "/run/anna/annad.sock".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_policy_ttl() {
        assert_eq!(CachePolicy::Static.ttl_seconds(), 0);
        assert_eq!(CachePolicy::Slow.ttl_seconds(), 3600);
        assert_eq!(CachePolicy::Volatile.ttl_seconds(), 5);
    }

    #[test]
    fn test_model_selection_low_ram() {
        let hw = HardwareInfo {
            ram_gb: 8,
            cpu_cores: 4,
            has_gpu: false,
            vram_gb: None,
        };
        let models = hw.select_models();
        assert_eq!(models.orchestrator, "llama3.2:3b");
        assert_eq!(models.expert, "qwen2.5:7b");
    }

    #[test]
    fn test_model_selection_high_ram_gpu() {
        let hw = HardwareInfo {
            ram_gb: 64,
            cpu_cores: 16,
            has_gpu: true,
            vram_gb: Some(24),
        };
        let models = hw.select_models();
        assert_eq!(models.orchestrator, "mistral-nemo-instruct");
        assert_eq!(models.expert, "qwen2.5:32b");
    }
}
