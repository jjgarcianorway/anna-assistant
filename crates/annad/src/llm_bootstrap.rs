//! LLM Bootstrap Manager v0.0.5
//!
//! Manages the LLM model setup lifecycle:
//! - Detects Ollama availability
//! - Selects models based on hardware tier
//! - Pulls models if needed (with progress tracking)
//! - Runs benchmarks to validate models
//! - Updates BootstrapState for status reporting

use anna_common::{
    AnnaConfig,
    OllamaClient, OllamaError,
    model_selection::{
        BootstrapPhase, BootstrapState, HardwareProfile, LlmRole,
        ModelCandidate, ModelSelection, DownloadProgress,
        default_candidates, select_model_for_role,
    },
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Shared bootstrap state for status reporting
pub type SharedBootstrapState = Arc<RwLock<BootstrapState>>;

/// Create a new shared bootstrap state
pub fn new_shared_state() -> SharedBootstrapState {
    Arc::new(RwLock::new(BootstrapState::default()))
}

/// Run the LLM bootstrap process
/// This runs once on startup and handles the entire bootstrap lifecycle
pub async fn run_bootstrap(
    state: SharedBootstrapState,
    config: &AnnaConfig,
) -> Result<(), OllamaError> {
    info!("[+]  LLM Bootstrap: Starting...");

    // Phase 1: Detect hardware
    {
        let mut s = state.write().await;
        s.phase = BootstrapPhase::DetectingOllama;
        s.touch();
        let _ = s.save();
    }

    let hardware = HardwareProfile::detect();
    info!(
        "[+]  Hardware detected: {} RAM, {} VRAM, tier: {}",
        HardwareProfile::format_memory(hardware.total_ram_bytes),
        if hardware.gpu_vram_bytes > 0 {
            HardwareProfile::format_memory(hardware.gpu_vram_bytes)
        } else {
            "no GPU".to_string()
        },
        hardware.tier
    );

    {
        let mut s = state.write().await;
        s.hardware = Some(hardware.clone());
        s.touch();
        let _ = s.save();
    }

    // Phase 2: Check Ollama availability
    let ollama_url = config.llm.ollama_url.clone();
    let client = OllamaClient::with_url(&ollama_url);

    if !client.is_available().await {
        warn!("[!]  Ollama not available at {}", ollama_url);
        {
            let mut s = state.write().await;
            s.phase = BootstrapPhase::Error;
            s.error = Some(format!("Ollama not available at {}", ollama_url));
            s.touch();
            let _ = s.save();
        }
        return Err(OllamaError::NotAvailable(ollama_url));
    }

    info!("[+]  Ollama available at {}", ollama_url);

    // Phase 3: Get available models
    let available_models = match client.list_models().await {
        Ok(models) => models.into_iter().map(|m| m.name).collect::<Vec<_>>(),
        Err(e) => {
            warn!("[!]  Failed to list Ollama models: {}", e);
            let mut s = state.write().await;
            s.phase = BootstrapPhase::Error;
            s.error = Some(format!("Failed to list models: {}", e));
            s.touch();
            let _ = s.save();
            return Err(e);
        }
    };

    info!("[+]  Found {} models available", available_models.len());

    // Phase 4: Select models for each role
    let translator_candidates = if config.llm.translator_candidates.is_empty() {
        default_candidates(LlmRole::Translator)
    } else {
        config.llm.translator_candidates.iter().enumerate().map(|(i, name)| {
            ModelCandidate {
                name: name.clone(),
                size_bytes: 2 * 1024 * 1024 * 1024, // Assume 2GB for custom
                priority: i as u32,
                min_tier: anna_common::model_selection::HardwareTier::Low,
                description: "Custom candidate".to_string(),
            }
        }).collect()
    };

    let junior_candidates = if config.llm.junior_candidates.is_empty() {
        default_candidates(LlmRole::Junior)
    } else {
        config.llm.junior_candidates.iter().enumerate().map(|(i, name)| {
            ModelCandidate {
                name: name.clone(),
                size_bytes: 4 * 1024 * 1024 * 1024, // Assume 4GB for custom
                priority: i as u32,
                min_tier: anna_common::model_selection::HardwareTier::Low,
                description: "Custom candidate".to_string(),
            }
        }).collect()
    };

    // Use config-specified model if set, otherwise auto-select
    let translator_selection = if !config.llm.translator.model.is_empty() {
        Some(ModelSelection {
            role: LlmRole::Translator,
            model: config.llm.translator.model.clone(),
            reason: "Configured in config.toml".to_string(),
            benchmark: None,
            hardware_tier: hardware.tier,
            timestamp: now_epoch(),
        })
    } else {
        select_model_for_role(LlmRole::Translator, &hardware, &available_models, &translator_candidates)
    };

    let junior_selection = if !config.llm.junior.model.is_empty() {
        Some(ModelSelection {
            role: LlmRole::Junior,
            model: config.llm.junior.model.clone(),
            reason: "Configured in config.toml".to_string(),
            benchmark: None,
            hardware_tier: hardware.tier,
            timestamp: now_epoch(),
        })
    } else {
        select_model_for_role(LlmRole::Junior, &hardware, &available_models, &junior_candidates)
    };

    if translator_selection.is_none() && junior_selection.is_none() {
        warn!("[!]  No suitable models found for either role");
        let mut s = state.write().await;
        s.phase = BootstrapPhase::Error;
        s.error = Some("No suitable models found for either role".to_string());
        s.touch();
        let _ = s.save();
        return Err(OllamaError::ModelNotFound("No suitable models".to_string()));
    }

    // Phase 5: Pull missing models
    let mut models_to_pull = Vec::new();

    if let Some(ref sel) = translator_selection {
        if !available_models.iter().any(|m| model_matches(m, &sel.model)) {
            models_to_pull.push(sel.model.clone());
        }
    }

    if let Some(ref sel) = junior_selection {
        if !available_models.iter().any(|m| model_matches(m, &sel.model)) {
            if !models_to_pull.contains(&sel.model) {
                models_to_pull.push(sel.model.clone());
            }
        }
    }

    if !models_to_pull.is_empty() {
        info!("[+]  Need to pull {} models: {:?}", models_to_pull.len(), models_to_pull);

        {
            let mut s = state.write().await;
            s.phase = BootstrapPhase::PullingModels;
            s.touch();
            let _ = s.save();
        }

        for model in &models_to_pull {
            info!("[+]  Pulling model: {}", model);

            // Update state with download info
            {
                let mut s = state.write().await;
                s.download_progress = Some(DownloadProgress {
                    model: model.clone(),
                    total_bytes: 0,
                    downloaded_bytes: 0,
                    speed_bytes_per_sec: 0.0,
                    eta_seconds: None,
                    status: "starting".to_string(),
                });
                s.touch();
                let _ = s.save();
            }

            // Pull with progress tracking
            let mut rx = client.pull_model(model).await?;
            let mut last_completed: u64 = 0;
            let mut last_time = std::time::Instant::now();

            while let Some(progress) = rx.recv().await {
                if let Some(ref err) = progress.error {
                    error!("[!]  Pull error for {}: {}", model, err);
                    let mut s = state.write().await;
                    s.phase = BootstrapPhase::Error;
                    s.error = Some(format!("Failed to pull {}: {}", model, err));
                    s.download_progress = None;
                    s.touch();
                    let _ = s.save();
                    return Err(OllamaError::HttpError(err.clone()));
                }

                if progress.is_complete() {
                    info!("[+]  Model {} pulled successfully", model);
                    break;
                }

                if progress.total > 0 {
                    // Calculate speed
                    let now = std::time::Instant::now();
                    let elapsed = now.duration_since(last_time).as_secs_f64();
                    let bytes_delta = progress.completed.saturating_sub(last_completed);
                    let speed = if elapsed > 0.0 {
                        bytes_delta as f64 / elapsed
                    } else {
                        0.0
                    };

                    // Calculate ETA
                    let remaining = progress.total.saturating_sub(progress.completed);
                    let eta = if speed > 0.0 {
                        Some((remaining as f64 / speed) as u64)
                    } else {
                        None
                    };

                    // Update state
                    let mut s = state.write().await;
                    s.download_progress = Some(DownloadProgress {
                        model: model.clone(),
                        total_bytes: progress.total,
                        downloaded_bytes: progress.completed,
                        speed_bytes_per_sec: speed,
                        eta_seconds: eta,
                        status: progress.status.clone(),
                    });
                    s.touch();
                    let _ = s.save();

                    last_completed = progress.completed;
                    last_time = now;
                }
            }

            // Clear download progress
            {
                let mut s = state.write().await;
                s.download_progress = None;
                s.touch();
                let _ = s.save();
            }
        }
    }

    // Phase 6: Mark as ready
    {
        let mut s = state.write().await;
        s.phase = BootstrapPhase::Ready;
        s.translator = translator_selection;
        s.junior = junior_selection;
        s.error = None;
        s.touch();
        let _ = s.save();
    }

    info!("[+]  LLM Bootstrap complete");
    {
        let s = state.read().await;
        if let Some(ref translator) = s.translator {
            info!("[+]  Translator model: {}", translator.model);
        }
        if let Some(ref junior) = s.junior {
            info!("[+]  Junior model: {}", junior.model);
        }
    }

    Ok(())
}

/// Check if model name matches (handles tags like :latest)
fn model_matches(available: &str, target: &str) -> bool {
    let available_base = available.split(':').next().unwrap_or(available);
    let target_base = target.split(':').next().unwrap_or(target);
    available_base == target_base || available.starts_with(target_base) || target.starts_with(available_base)
}

/// Get current epoch timestamp
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_matches() {
        assert!(model_matches("qwen2.5:0.5b", "qwen2.5:0.5b"));
        assert!(model_matches("qwen2.5:latest", "qwen2.5"));
        assert!(model_matches("llama3.2:1b-instruct", "llama3.2"));
    }
}
