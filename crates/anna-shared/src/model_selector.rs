//! Model selector with Qwen3-VL preference (v0.0.74).
//! Selects best model from available inventory, prefers Qwen3-VL family.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model family for preference ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    Qwen3VL,  // Preferred
    Qwen25,   // Fallback
    Llama32,  // Fallback
    Other,
}

/// Model role for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Translator,  // Query classification (small)
    Specialist,  // Domain expert (capable)
}

/// Model candidate with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCandidate {
    pub name: String,          // Full model name (e.g., "qwen3-vl:4b")
    pub family: ModelFamily,
    pub size_gb: f32,          // Estimated VRAM/RAM needed (GB)
    pub priority: u32,         // Lower = better for role
    pub roles: Vec<ModelRole>,
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

/// Model selector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectorConfig {
    pub prefer_qwen3_vl: bool,
    pub min_translator_tps: f32,
    pub min_specialist_tps: f32,
    pub enable_benchmark: bool,
    pub benchmark_interval_secs: u64,
}

impl Default for ModelSelectorConfig {
    fn default() -> Self {
        Self { prefer_qwen3_vl: true, min_translator_tps: 10.0, min_specialist_tps: 5.0,
               enable_benchmark: true, benchmark_interval_secs: 604800 }
    }
}

/// Model selector state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelSelectorState {
    pub translator: Option<ModelSelection>,
    pub specialist: Option<ModelSelection>,
    pub available_models: Vec<String>,
    pub benchmarks: HashMap<String, ModelBenchmark>,
    pub last_selection_ts: u64,
    pub last_benchmark_ts: u64,
}

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

/// Select best model for a role from available models
pub fn select_model(
    role: ModelRole,
    available: &[String],
    config: &ModelSelectorConfig,
    benchmarks: &HashMap<String, ModelBenchmark>,
) -> Option<ModelSelection> {
    let catalog = model_catalog();

    // Filter to models that are available and support the role
    let mut candidates: Vec<&ModelCandidate> = catalog.iter()
        .filter(|c| c.roles.contains(&role))
        .filter(|c| available.iter().any(|a| model_matches(&c.name, a)))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by: family preference, then priority, then benchmark if available
    candidates.sort_by(|a, b| {
        // Family preference: Qwen3VL > Qwen25 > Llama32 > Other
        let family_order = |f: &ModelFamily| match f {
            ModelFamily::Qwen3VL if config.prefer_qwen3_vl => 0,
            ModelFamily::Qwen25 => 1,
            ModelFamily::Llama32 => 2,
            ModelFamily::Qwen3VL => 1,
            ModelFamily::Other => 3,
        };

        let a_family = family_order(&a.family);
        let b_family = family_order(&b.family);

        if a_family != b_family {
            return a_family.cmp(&b_family);
        }

        // Same family: check benchmark if available
        let a_tps = benchmarks.get(&a.name).map(|b| b.tokens_per_sec).unwrap_or(0.0);
        let b_tps = benchmarks.get(&b.name).map(|b| b.tokens_per_sec).unwrap_or(0.0);

        if a_tps > 0.0 && b_tps > 0.0 {
            // Higher TPS is better
            return b_tps.partial_cmp(&a_tps).unwrap_or(std::cmp::Ordering::Equal);
        }

        // Fallback to priority
        a.priority.cmp(&b.priority)
    });

    let selected = candidates.first()?;
    let is_preferred = selected.family == ModelFamily::Qwen3VL && config.prefer_qwen3_vl;
    let is_fallback = !is_preferred && config.prefer_qwen3_vl;

    let reason = if is_preferred {
        format!("preferred: {} available", selected.family_display())
    } else if is_fallback {
        format!("fallback: {} (Qwen3-VL not available)", selected.family_display())
    } else {
        format!("selected: {}", selected.family_display())
    };

    Some(ModelSelection {
        model: selected.name.clone(),
        family: selected.family,
        reason,
        is_preferred,
        is_fallback,
    })
}

/// Check if a catalog model name matches an available model
fn model_matches(catalog_name: &str, available_name: &str) -> bool {
    let normalize = |s: &str| s.to_lowercase().replace('-', "").replace('_', "");
    let c = normalize(catalog_name);
    let a = normalize(available_name);

    // Exact match
    if c == a {
        return true;
    }

    // Prefix match (qwen3vl:4b matches qwen3-vl:4b-q4_k_m)
    if a.starts_with(&c) {
        return true;
    }

    // Check if base model matches (without quantization suffix)
    let a_base = a.split(':').next().unwrap_or(&a);
    let c_base = c.split(':').next().unwrap_or(&c);

    a_base == c_base || a_base.starts_with(c_base)
}

impl ModelCandidate {
    fn family_display(&self) -> &'static str {
        match self.family {
            ModelFamily::Qwen3VL => "Qwen3-VL",
            ModelFamily::Qwen25 => "Qwen2.5",
            ModelFamily::Llama32 => "Llama3.2",
            ModelFamily::Other => "Other",
        }
    }
}

/// Micro-benchmark prompt for quick performance measurement
/// Short enough to measure quickly, long enough to be meaningful
pub const BENCHMARK_PROMPT: &str = "Classify: 'how much RAM do I have?' Reply: system/network/storage";

/// Expected response length for benchmark (tokens)
pub const BENCHMARK_EXPECTED_TOKENS: u32 = 10;

/// Parse ollama benchmark response into ModelBenchmark
/// Use this with the result from an ollama /api/generate call
pub fn parse_benchmark_response(
    model: &str,
    response: &serde_json::Value,
    fallback_duration_ns: u64,
) -> ModelBenchmark {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Extract timing from ollama response
    // ollama returns: total_duration, load_duration, prompt_eval_duration, eval_duration in nanoseconds
    let ttft_ns = response
        .get("prompt_eval_duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let eval_ns = response
        .get("eval_duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(fallback_duration_ns);
    let eval_count = response
        .get("eval_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    // Calculate tokens per second
    let eval_secs = eval_ns as f64 / 1_000_000_000.0;
    let tokens_per_sec = if eval_secs > 0.0 {
        eval_count as f32 / eval_secs as f32
    } else {
        0.0
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ModelBenchmark {
        model: model.to_string(),
        tokens_per_sec,
        ttft_ms: (ttft_ns / 1_000_000) as u64,
        timestamp,
    }
}

/// Detect model family from model name
pub fn detect_family(model_name: &str) -> ModelFamily {
    let name_lower = model_name.to_lowercase();

    if name_lower.contains("qwen3-vl") || name_lower.contains("qwen3vl") {
        ModelFamily::Qwen3VL
    } else if name_lower.contains("qwen2.5") || name_lower.contains("qwen25") {
        ModelFamily::Qwen25
    } else if name_lower.contains("llama3.2") || name_lower.contains("llama32") {
        ModelFamily::Llama32
    } else {
        ModelFamily::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_catalog_not_empty() {
        let catalog = model_catalog();
        assert!(!catalog.is_empty());
    }

    #[test]
    fn test_select_model_prefers_qwen3_vl() {
        let available = vec![
            "qwen3-vl:4b".to_string(),
            "qwen2.5:3b".to_string(),
            "llama3.2:3b".to_string(),
        ];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.family, ModelFamily::Qwen3VL);
        assert!(sel.is_preferred);
        assert!(!sel.is_fallback);
    }

    #[test]
    fn test_select_model_fallback() {
        let available = vec![
            "qwen2.5:3b".to_string(),
            "llama3.2:3b".to_string(),
        ];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert!(sel.is_fallback);
    }

    #[test]
    fn test_model_matches() {
        assert!(model_matches("qwen3-vl:4b", "qwen3-vl:4b"));
        assert!(model_matches("qwen3-vl:4b", "qwen3vl:4b"));
        assert!(model_matches("qwen3-vl:4b", "qwen3-vl:4b-q4_k_m"));
        assert!(!model_matches("qwen3-vl:4b", "qwen2.5:3b"));
    }

    #[test]
    fn test_select_translator_prefers_small() {
        let available = vec![
            "qwen3-vl:4b".to_string(),
            "qwen3-vl:2b".to_string(),
            "qwen3-vl:1b".to_string(),
        ];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Translator, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        // Should select 2b (smallest with priority 1 for translator)
        assert!(sel.model.contains("2b"));
    }
}
