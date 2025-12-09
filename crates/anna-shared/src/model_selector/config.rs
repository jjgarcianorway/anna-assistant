//! Model selector configuration (v0.0.223).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ModelBenchmark, ModelSelection};

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
        Self {
            prefer_qwen3_vl: true,
            min_translator_tps: 10.0,
            min_specialist_tps: 5.0,
            enable_benchmark: true,
            benchmark_interval_secs: 604800,
        }
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
