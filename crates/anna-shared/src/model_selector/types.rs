//! Model selector types (v0.0.223).
//! v0.0.267: Added DeepSeekR1 family for reasoning-focused tasks.

use serde::{Deserialize, Serialize};

/// Model family for preference ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    Qwen3VL,    // Preferred for vision/multimodal
    DeepSeekR1, // v0.0.267: Strong reasoning capability
    Qwen25,     // General fallback
    Llama32,    // Fallback
    Other,
}

/// Model role for selection
/// v0.0.277: Added Junior and Senior roles for tiered expertise
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Translator, // Query classification + formatting (smallest, fastest)
    Junior,     // Regular specialist work (mid-size, capable)
    Senior,     // Complex/escalated queries (largest, smartest)
    Specialist, // Legacy: maps to Junior for backwards compatibility
}

/// Model candidate with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCandidate {
    pub name: String, // Full model name (e.g., "qwen3-vl:4b")
    pub family: ModelFamily,
    pub size_gb: f32,  // Estimated VRAM/RAM needed (GB)
    pub priority: u32, // Lower = better for role
    pub roles: Vec<ModelRole>,
}

impl ModelCandidate {
    pub fn family_display(&self) -> &'static str {
        match self.family {
            ModelFamily::Qwen3VL => "Qwen3-VL",
            ModelFamily::DeepSeekR1 => "DeepSeek-R1",
            ModelFamily::Qwen25 => "Qwen2.5",
            ModelFamily::Llama32 => "Llama3.2",
            ModelFamily::Other => "Other",
        }
    }
}

/// Model selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    pub model: String,
    pub family: ModelFamily,
    pub reason: String,
    pub is_preferred: bool,
    pub is_fallback: bool,
}

/// Benchmark result for a model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelBenchmark {
    pub model: String,
    pub tokens_per_sec: f32, // Tokens per second (inference)
    pub ttft_ms: u64,        // Time to first token (ms)
    pub timestamp: u64,
}
