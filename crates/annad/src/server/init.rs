//! Daemon initialization - ollama setup and model management.

use anna_shared::status::DaemonState;
use anyhow::Result;
use tracing::{info, warn};

use crate::core_loop::init_system_profile;
use crate::ollama;
use crate::state::SharedState;

/// Initialize the daemon (install/start ollama, pull model)
pub async fn initialize(state: SharedState) -> Result<()> {
    info!("Initializing...");

    // Initialize system profile first (scans hardware, configs, preferences)
    init_system_profile();

    // Detect hardware for GPU/VRAM
    let hw = ollama::detect_hardware();
    let best_model = ollama::select_best_model(&hw);
    info!("Best model for this hardware: {}", best_model);

    // Store hardware info in state
    {
        let mut s = state.write().await;
        s.gpu = Some(format!("{:?}", hw.gpu_type));
        s.vram_mb = if hw.vram_mb > 0 {
            Some(hw.vram_mb)
        } else {
            None
        };
    }

    // Install ollama if needed (will pick cuda/rocm variant based on GPU)
    if !ollama::is_installed() {
        info!("Installing Ollama...");
        ollama::install().await?;
    }

    // Start ollama if not running
    if !ollama::is_running().await {
        info!("Starting Ollama...");
        ollama::start_service().await?;
    }

    // Check what models are available
    let models = ollama::list_models().await.unwrap_or_default();
    info!("Available models: {:?}", models);

    // Check if we already have the best model or a suitable alternative
    let model = if models
        .iter()
        .any(|m| m.starts_with(best_model.split(':').next().unwrap_or(best_model)))
    {
        // We have a version of the best model family
        models
            .iter()
            .find(|m| m.starts_with(best_model.split(':').next().unwrap_or(best_model)))
            .cloned()
            .unwrap_or_else(|| best_model.to_string())
    } else if !models.is_empty() {
        // Use best available model (prefer larger ones)
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| {
            let size_a = extract_model_size(a);
            let size_b = extract_model_size(b);
            size_b.cmp(&size_a) // Larger first
        });
        sorted[0].clone()
    } else {
        // No models - pull the best one for this hardware
        info!("No models found, pulling {}...", best_model);
        ollama::pull_model(best_model).await?;
        best_model.to_string()
    };

    // If current model is smaller than best and we have resources, upgrade
    let current_size = extract_model_size(&model);
    let best_size = extract_model_size(best_model);
    if current_size < best_size && !models.iter().any(|m| m == best_model) {
        info!(
            "Upgrading from {}B to {}B model for better performance...",
            current_size, best_size
        );
        if let Err(e) = ollama::pull_model(best_model).await {
            warn!(
                "Failed to pull better model, continuing with {}: {}",
                model, e
            );
        } else {
            // Use the newly pulled model
            let model = best_model.to_string();
            info!("Using upgraded model: {}", model);

            // Update state with upgraded model
            {
                let mut state = state.write().await;
                state.ollama_running = true;
                state.model = Some(model);
                state.state = DaemonState::Ready;
            }

            info!("Initialization complete - daemon ready");
            return Ok(());
        }
    }

    info!("Using model: {}", model);

    // Update state
    {
        let mut state = state.write().await;
        state.ollama_running = true;
        state.model = Some(model);
        state.state = DaemonState::Ready;
    }

    info!("Initialization complete - daemon ready");
    Ok(())
}

/// Extract model size in billions from model name (e.g., "qwen2.5:7b" -> 7)
pub fn extract_model_size(model: &str) -> u32 {
    // Look for patterns like "7b", "14b", "3b", etc.
    let model_lower = model.to_lowercase();
    for part in model_lower.split(|c: char| !c.is_alphanumeric()) {
        if part.ends_with('b') {
            if let Ok(size) = part.trim_end_matches('b').parse::<u32>() {
                return size;
            }
        }
    }
    // Check for size in the model name itself
    if model_lower.contains("14b") {
        return 14;
    }
    if model_lower.contains("13b") {
        return 13;
    }
    if model_lower.contains("7b") {
        return 7;
    }
    if model_lower.contains("8b") {
        return 8;
    }
    if model_lower.contains("3b") {
        return 3;
    }
    if model_lower.contains("1.5b") {
        return 1;
    }
    1 // Default to smallest
}
