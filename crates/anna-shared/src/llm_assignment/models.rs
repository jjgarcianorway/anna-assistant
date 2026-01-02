//! Common LLM Models and Tier Detection
//!
//! Model definitions and tier assignment logic.

use super::types::ModelTier;

/// Common model names
pub const COMMON_MODELS: &[(&str, ModelTier)] = &[
    ("llama3.2:1b", ModelTier::Light),
    ("llama3.2:3b", ModelTier::Light),
    ("llama3.1:8b", ModelTier::Standard),
    ("llama3.1:70b", ModelTier::Heavy),
    ("qwen2.5:0.5b", ModelTier::Light),
    ("qwen2.5:7b", ModelTier::Standard),
    ("qwen2.5:32b", ModelTier::Heavy),
    ("deepseek-r1:8b", ModelTier::DeepThinking),
    ("deepseek-r1:32b", ModelTier::DeepThinking),
];

/// Get tier for model
pub fn get_model_tier(model: &str) -> ModelTier {
    for (name, tier) in COMMON_MODELS {
        if model.contains(name) {
            return *tier;
        }
    }
    ModelTier::Standard
}
