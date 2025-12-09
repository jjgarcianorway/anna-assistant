//! Model catalog (v0.0.223).

use super::types::{ModelCandidate, ModelFamily, ModelRole};

/// Central model catalog - maps model names to candidates
/// v0.0.74: Centralized to avoid scattered model names
pub fn model_catalog() -> Vec<ModelCandidate> {
    vec![
        // Qwen3-VL family (preferred)
        ModelCandidate {
            name: "qwen3-vl:4b".to_string(),
            family: ModelFamily::Qwen3VL,
            size_gb: 2.5,
            priority: 1,
            roles: vec![ModelRole::Specialist],
        },
        ModelCandidate {
            name: "qwen3-vl:2b".to_string(),
            family: ModelFamily::Qwen3VL,
            size_gb: 1.5,
            priority: 1,
            roles: vec![ModelRole::Translator],
        },
        ModelCandidate {
            name: "qwen3-vl:1b".to_string(),
            family: ModelFamily::Qwen3VL,
            size_gb: 0.8,
            priority: 2,
            roles: vec![ModelRole::Translator],
        },
        // Qwen2.5 family (fallback)
        ModelCandidate {
            name: "qwen2.5:3b".to_string(),
            family: ModelFamily::Qwen25,
            size_gb: 2.0,
            priority: 2,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        ModelCandidate {
            name: "qwen2.5:1.5b".to_string(),
            family: ModelFamily::Qwen25,
            size_gb: 1.0,
            priority: 1,
            roles: vec![ModelRole::Translator],
        },
        ModelCandidate {
            name: "qwen2.5:0.5b".to_string(),
            family: ModelFamily::Qwen25,
            size_gb: 0.4,
            priority: 3,
            roles: vec![ModelRole::Translator],
        },
        // Llama3.2 family (fallback)
        ModelCandidate {
            name: "llama3.2:3b".to_string(),
            family: ModelFamily::Llama32,
            size_gb: 2.0,
            priority: 2,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        ModelCandidate {
            name: "llama3.2:1b".to_string(),
            family: ModelFamily::Llama32,
            size_gb: 0.8,
            priority: 3,
            roles: vec![ModelRole::Translator],
        },
    ]
}
