//! Model catalog (v0.0.223).
//! v0.0.267: Added DeepSeek-R1 family for reasoning tasks.
//! v0.0.393: Translator requires 3B+ - smaller models can't reliably output JSON.

use super::types::{ModelCandidate, ModelFamily, ModelRole};

/// Central model catalog - maps model names to candidates
/// v0.0.74: Centralized to avoid scattered model names
/// v0.0.267: Added DeepSeek-R1 models (excellent reasoning, based on Qwen)
/// v0.0.393: IMPORTANT - Translator needs 3B+ for reliable JSON output!
///           Models < 3B produce malformed JSON causing routing failures.
pub fn model_catalog() -> Vec<ModelCandidate> {
    vec![
        // Qwen3-VL family (preferred for multimodal)
        // v0.0.393: 4B can be translator too since it's reliable
        ModelCandidate {
            name: "qwen3-vl:4b".to_string(),
            family: ModelFamily::Qwen3VL,
            size_gb: 2.5,
            priority: 1,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        // v0.0.393: REMOVED 2B and 1B from Translator - they produce garbage JSON
        // qwen3-vl:2b and qwen3-vl:1b are too small for structured output

        // DeepSeek-R1 family (v0.0.267: strong reasoning capability)
        ModelCandidate {
            name: "deepseek-r1:7b".to_string(),
            family: ModelFamily::DeepSeekR1,
            size_gb: 4.7,
            priority: 1,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        // v0.0.393: REMOVED deepseek-r1:1.5b from Translator - too small

        // Qwen2.5 family
        ModelCandidate {
            name: "qwen2.5:7b-instruct".to_string(),
            family: ModelFamily::Qwen25,
            size_gb: 4.7,
            priority: 1,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        ModelCandidate {
            name: "qwen2.5:3b-instruct".to_string(),
            family: ModelFamily::Qwen25,
            size_gb: 2.0,
            priority: 2,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        // v0.0.393: REMOVED 1.5b and 0.5b from Translator - too small for JSON

        // Llama3.2 family
        ModelCandidate {
            name: "llama3.2:3b".to_string(),
            family: ModelFamily::Llama32,
            size_gb: 2.0,
            priority: 2,
            roles: vec![ModelRole::Specialist, ModelRole::Translator],
        },
        // v0.0.393: REMOVED llama3.2:1b from Translator - too small
    ]
}
