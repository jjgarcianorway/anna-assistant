//! Background model setup logic.
//! v0.0.825: Extracted from initialization.rs for modularity.

use anna_shared::ledger::{LedgerEntry, LedgerEntryKind};
use anyhow::Result;
use tracing::{error, info, warn};

use crate::auto_select;
use crate::ollama;
use crate::state::SharedState;

/// v0.0.310: Background model selection and pulling.
/// Runs after daemon is already marked Ready, so deterministic answers work immediately.
pub async fn setup_models_background(state: SharedState) -> Result<()> {
    // Get hardware info for model selection
    let (available_ram_gb, has_gpu) = {
        let state_read = state.read().await;
        let ram_gb = state_read.hardware.ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
        let has_gpu = state_read.hardware.gpu.is_some();
        (ram_gb, has_gpu)
    };

    // Update phase
    {
        let mut state_write = state.write().await;
        state_write.set_llm_phase("selecting_models");
    }

    // Try auto-selection first, fall back to config defaults
    let (translator_model, specialist_model) =
        match auto_select::auto_select_models(available_ram_gb, has_gpu).await {
            Ok(result) => {
                // Record models pulled by Anna
                for model_name in &result.models_pulled {
                    let mut state_write = state.write().await;
                    state_write.ledger.add(LedgerEntry::new(
                        LedgerEntryKind::ModelPulled,
                        model_name.clone(),
                        true,
                    ));
                }

                // Update state with benchmark results
                {
                    let mut state_write = state.write().await;
                    for (model, bench) in &result.benchmarks {
                        state_write.add_model(
                            model,
                            "benchmarked",
                            bench.tokens_per_sec as u64,
                        );
                    }
                }

                info!(
                    "Auto-selected: translator={} specialist={}",
                    result.translator.model, result.specialist.model
                );
                (result.translator.model, result.specialist.model)
            }
            Err(e) => {
                warn!("Auto-selection failed: {}, using config defaults", e);
                let state_read = state.read().await;
                (
                    state_read.config.llm.translator_model.clone(),
                    state_read.config.llm.specialist_model.clone(),
                )
            }
        };

    // Phase: Pulling models (ensure selected models are available)
    {
        let mut state_write = state.write().await;
        state_write.set_llm_phase("pulling_models");
    }

    let required_models = vec![translator_model.clone(), specialist_model.clone()];
    for model_name in &required_models {
        if !ollama::has_model(model_name).await {
            info!("Pulling model: {}", model_name);
            if let Err(e) = ollama::pull_model(model_name).await {
                error!("Failed to pull model {}: {}", model_name, e);
                continue;
            }
            let mut state_write = state.write().await;
            state_write.ledger.add(LedgerEntry::new(
                LedgerEntryKind::ModelPulled,
                model_name.clone(),
                true,
            ));
        } else {
            info!("Model already available: {}", model_name);
        }
    }

    // Use translator as supervisor (fast model)
    let supervisor_model = translator_model.clone();

    // Add models to status with their roles
    {
        let mut state_write = state.write().await;
        state_write.add_model(&translator_model, "translator", 0);
        state_write.add_model(&specialist_model, "junior", 0);
        if supervisor_model != translator_model && supervisor_model != specialist_model {
            state_write.add_model(&supervisor_model, "supervisor", 0);
        }
        state_write.llm.translator_model = Some(translator_model.clone());
        state_write.llm.junior_model = Some(specialist_model.clone());
        state_write.llm.specialist_model = Some(specialist_model.clone());
        state_write.llm.senior_model = Some(state_write.config.llm.senior_model.clone());
        let family = anna_shared::model_selector::detect_family(&specialist_model);
        state_write.llm.preferred_family = Some(format!("{:?}", family));
    }

    // Run benchmark on specialist model
    let _throughput = ollama::benchmark(&specialist_model).await.unwrap_or(0.0);

    // v0.0.818: Enhanced model cleanup - remove duplicates and unused models
    cleanup_unused_models(&state, &required_models).await;

    // Mark models as fully ready
    {
        let mut state_write = state.write().await;
        state_write.set_llm_ready();
    }

    info!("Background model setup complete - LLM fully ready");
    Ok(())
}

/// Clean up unused and duplicate models to free disk space.
async fn cleanup_unused_models(state: &SharedState, required_models: &[String]) {
    let installed = ollama::list_models().await.unwrap_or_default();
    let anna_pulled: Vec<String> = {
        let state_read = state.read().await;
        state_read
            .ledger
            .entries
            .iter()
            .filter(|e| matches!(e.kind, LedgerEntryKind::ModelPulled))
            .map(|e| e.target.clone())
            .collect()
    };

    // Group models by base name to detect duplicates
    let mut model_groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for model in &installed {
        // Extract base model name (before :)
        let base = model.split(':').next().unwrap_or(model);
        model_groups
            .entry(base.to_string())
            .or_default()
            .push(model.clone());
    }

    // Keep only needed models and one version of each family
    for model in &installed {
        // Check if this exact model is needed
        let is_needed = required_models
            .iter()
            .any(|r| model.contains(r) || r.contains(model));
        if is_needed {
            continue;
        }

        // Check if Anna owns this model
        let anna_owns = anna_pulled
            .iter()
            .any(|p| model.contains(p) || p.contains(model));

        // v0.0.818: Clean up duplicates even if Anna doesn't own them
        // Extract base name
        let base = model.split(':').next().unwrap_or(model);
        let variants = model_groups.get(base).map(|v| v.len()).unwrap_or(0);

        // Only clean up if:
        // 1. Anna owns it AND it's not needed, OR
        // 2. It's a duplicate (multiple variants of same base) AND not needed
        let should_cleanup = anna_owns || variants > 1;

        if !should_cleanup {
            continue;
        }

        // For duplicates, check if any variant is needed
        if variants > 1 {
            let any_variant_needed = model_groups
                .get(base)
                .map(|variants| {
                    variants.iter().any(|v| {
                        required_models
                            .iter()
                            .any(|r| v.contains(r) || r.contains(v))
                    })
                })
                .unwrap_or(false);

            // If a variant is needed, skip cleaning unless this is an extra duplicate
            if any_variant_needed {
                continue;
            }
        }

        info!("Cleaning up unused model: {}", model);
        if let Ok(()) = ollama::delete_model(model).await {
            let mut state_write = state.write().await;
            state_write.ledger.add(LedgerEntry::new(
                LedgerEntryKind::ModelDeleted,
                model.clone(),
                true,
            ));
        }
    }
}
