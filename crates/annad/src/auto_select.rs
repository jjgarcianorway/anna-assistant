//! Intelligent model auto-selection (v0.0.269).
//!
//! Benchmarks available models and selects the best for each role.
//! Uses hardware detection to make smart decisions.

use std::collections::HashMap;

use anna_shared::model_selector::{
    model_catalog, select_model, ModelBenchmark, ModelRole, ModelSelection, ModelSelectorConfig,
};
use tracing::{debug, info, warn};

use crate::benchmark::run_micro_benchmark;
use crate::ollama;

/// Result of auto-selection process
#[derive(Debug, Clone)]
pub struct AutoSelectResult {
    pub translator: ModelSelection,
    pub specialist: ModelSelection,
    pub benchmarks: HashMap<String, ModelBenchmark>,
    pub models_pulled: Vec<String>,
}

/// Auto-select best models for all roles
/// 1. Lists available models
/// 2. Identifies catalog models that need pulling
/// 3. Pulls missing models if needed
/// 4. Benchmarks available models
/// 5. Selects best models for each role
pub async fn auto_select_models(
    available_ram_gb: f32,
    has_gpu: bool,
) -> Result<AutoSelectResult, String> {
    info!("Starting intelligent model auto-selection");
    info!("Hardware: {:.1}GB RAM, GPU: {}", available_ram_gb, has_gpu);

    // Step 1: List available models
    let available = ollama::list_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;
    info!("Found {} locally available models", available.len());

    // Step 2: Get catalog and filter by hardware
    let catalog = model_catalog();
    let suitable_models: Vec<_> = catalog
        .iter()
        .filter(|c| c.size_gb <= available_ram_gb)
        .collect();
    info!(
        "Catalog has {} models suitable for hardware",
        suitable_models.len()
    );

    // Step 3: Identify models to pull
    let mut models_pulled = Vec::new();
    let mut final_available = available.clone();

    // Determine which catalog models to pull based on need
    let needed_models = determine_needed_models(&suitable_models, &available, available_ram_gb);

    for model_name in &needed_models {
        if !model_available(model_name, &final_available) {
            info!("Pulling needed model: {}", model_name);
            match ollama::pull_model(model_name).await {
                Ok(()) => {
                    models_pulled.push(model_name.clone());
                    final_available.push(model_name.clone());
                    info!("Successfully pulled: {}", model_name);
                }
                Err(e) => {
                    warn!("Failed to pull {}: {}", model_name, e);
                }
            }
        }
    }

    // Step 4: Benchmark available models
    let benchmarks = benchmark_models(&final_available).await;
    info!("Benchmarked {} models", benchmarks.len());

    // Step 5: Select best models
    let config = ModelSelectorConfig::default();

    let translator = select_model(ModelRole::Translator, &final_available, &config, &benchmarks)
        .ok_or("No suitable translator model found")?;

    let specialist = select_model(ModelRole::Specialist, &final_available, &config, &benchmarks)
        .ok_or("No suitable specialist model found")?;

    info!("Selected translator: {} ({})", translator.model, translator.reason);
    info!("Selected specialist: {} ({})", specialist.model, specialist.reason);

    Ok(AutoSelectResult {
        translator,
        specialist,
        benchmarks,
        models_pulled,
    })
}

/// Determine which models should be pulled based on needs
fn determine_needed_models(
    suitable: &[&anna_shared::model_selector::ModelCandidate],
    available: &[String],
    ram_gb: f32,
) -> Vec<String> {
    let mut needed = Vec::new();

    // Check if we have a suitable translator
    let has_translator = suitable
        .iter()
        .any(|c| c.roles.contains(&ModelRole::Translator) && model_available(&c.name, available));

    // Check if we have a suitable specialist
    let has_specialist = suitable
        .iter()
        .any(|c| c.roles.contains(&ModelRole::Specialist) && model_available(&c.name, available));

    // Find best translator to pull
    if !has_translator {
        if let Some(best) = suitable
            .iter()
            .filter(|c| c.roles.contains(&ModelRole::Translator))
            .min_by_key(|c| c.priority)
        {
            debug!("Need to pull translator: {}", best.name);
            needed.push(best.name.clone());
        }
    }

    // Find best specialist to pull
    if !has_specialist {
        if let Some(best) = suitable
            .iter()
            .filter(|c| c.roles.contains(&ModelRole::Specialist))
            .min_by_key(|c| c.priority)
        {
            // Don't pull if too big for system
            if best.size_gb <= ram_gb * 0.7 {
                debug!("Need to pull specialist: {}", best.name);
                needed.push(best.name.clone());
            }
        }
    }

    // Always try to pull preferred family if we have RAM
    if ram_gb >= 4.0 {
        // Try DeepSeek-R1 for reasoning
        let deepseek = "deepseek-r1:7b";
        if !model_available(deepseek, available) && !needed.contains(&deepseek.to_string()) {
            if ram_gb >= 8.0 {
                needed.push(deepseek.to_string());
            }
        }
        // Try Qwen3-VL for vision
        let qwen3vl = "qwen3-vl:4b";
        if !model_available(qwen3vl, available) && !needed.contains(&qwen3vl.to_string()) {
            needed.push(qwen3vl.to_string());
        }
    }

    needed
}

/// Check if a model is available (fuzzy match)
fn model_available(name: &str, available: &[String]) -> bool {
    let name_lower = name.to_lowercase();
    available.iter().any(|a| {
        let a_lower = a.to_lowercase();
        a_lower.contains(&name_lower) || name_lower.contains(&a_lower)
    })
}

/// Benchmark all available models
async fn benchmark_models(models: &[String]) -> HashMap<String, ModelBenchmark> {
    let mut results = HashMap::new();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap();
    let ollama_url = "http://127.0.0.1:11434";

    for model in models {
        debug!("Benchmarking: {}", model);
        match run_micro_benchmark(&client, ollama_url, model).await {
            Ok(bench) => {
                info!(
                    "  {} - {:.1} tok/s, {} ms TTFT",
                    model, bench.tokens_per_sec, bench.ttft_ms
                );
                results.insert(model.clone(), bench);
            }
            Err(e) => {
                debug!("Benchmark failed for {}: {}", model, e);
            }
        }
    }

    results
}

/// Quick selection without benchmarking (for faster startup)
pub async fn quick_select_models(available_ram_gb: f32) -> Result<(String, String), String> {
    let available = ollama::list_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;

    let catalog = model_catalog();
    let config = ModelSelectorConfig::default();
    let empty_benchmarks = HashMap::new();

    // Filter suitable models
    let suitable: Vec<String> = available
        .iter()
        .filter(|a| {
            catalog
                .iter()
                .any(|c| c.size_gb <= available_ram_gb && model_available(&c.name, &[a.to_string()]))
        })
        .cloned()
        .collect();

    let translator =
        select_model(ModelRole::Translator, &suitable, &config, &empty_benchmarks)
            .map(|s| s.model)
            .unwrap_or_else(|| "qwen2.5:0.5b-instruct".to_string());

    let specialist =
        select_model(ModelRole::Specialist, &suitable, &config, &empty_benchmarks)
            .map(|s| s.model)
            .unwrap_or_else(|| "qwen2.5:7b-instruct".to_string());

    Ok((translator, specialist))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_available_exact() {
        let available = vec!["qwen2.5:7b-instruct".to_string()];
        assert!(model_available("qwen2.5:7b", &available));
    }

    #[test]
    fn test_model_available_partial() {
        let available = vec!["deepseek-r1:7b-q4_K_M".to_string()];
        assert!(model_available("deepseek-r1:7b", &available));
    }

    #[test]
    fn test_determine_needed_models_empty() {
        let catalog = model_catalog();
        let suitable: Vec<_> = catalog.iter().collect();
        let available = vec![];

        let needed = determine_needed_models(&suitable, &available, 16.0);
        assert!(!needed.is_empty(), "Should suggest models to pull");
    }
}
