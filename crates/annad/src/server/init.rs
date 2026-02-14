//! Daemon initialization - ollama setup and model management.

use anna_shared::status::DaemonState;
use anyhow::{anyhow, Result};
use tracing::{info, warn};

use crate::core_loop::init_system_profile;
use crate::ollama;
use crate::state::SharedState;

/// Remove the v0.3.216 regression drop-in if it exists.
/// That drop-in set OLLAMA_MODELS=/var/lib/anna/models on the systemd service,
/// making all pre-existing models in /var/lib/ollama invisible.
async fn remove_v0316_dropin() {
    const DROPIN: &str = "/etc/systemd/system/ollama.service.d/anna.conf";
    if !std::path::Path::new(DROPIN).exists() {
        return;
    }
    info!("Found v0.3.216 ollama drop-in — removing to restore default model path");
    if let Err(e) = std::fs::remove_file(DROPIN) {
        warn!("Failed to remove drop-in {}: {}", DROPIN, e);
        return;
    }
    // Remove the now-empty directory if possible
    let _ = std::fs::remove_dir("/etc/systemd/system/ollama.service.d");
    // daemon-reload so systemd forgets the drop-in
    let _ = tokio::task::spawn_blocking(|| {
        std::process::Command::new("/usr/bin/systemctl")
            .args(["daemon-reload"])
            .output()
    }).await;
    // Restart ollama only if it was already running — try-restart is a no-op if stopped
    let _ = tokio::task::spawn_blocking(|| {
        std::process::Command::new("/usr/bin/systemctl")
            .args(["try-restart", "ollama"])
            .output()
    }).await;
    // Wait for it to come back
    for _ in 0..15 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if ollama::is_running().await {
            info!("Ollama restarted with default model path");
            return;
        }
    }
    warn!("Ollama did not restart cleanly after drop-in removal");
}

/// Initialize the daemon (install/start ollama, pull model)
pub async fn initialize(state: SharedState) -> Result<()> {
    info!("Initializing...");

    // v0.3.218: Remove the v0.3.216 OLLAMA_MODELS drop-in if present
    remove_v0316_dropin().await;

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

    // Determine total steps so user sees "step X of Y"
    let ollama_missing = !ollama::is_installed();
    let total_steps: u8 = if ollama_missing { 4 } else { 3 };
    let mut step: u8 = 1;

    // Step: clear previous error before retry
    {
        let mut s = state.write().await;
        s.last_error = None;
    }

    // Step: Install ollama if needed
    if ollama_missing {
        info!("Installing Ollama...");
        {
            let mut s = state.write().await;
            s.init_status = format!(
                "[{}/{}] Installing ollama via pacman — takes 2–5 min...",
                step, total_steps
            );
        }
        ollama::install().await?;
        step += 1;
        {
            let mut s = state.write().await;
            s.init_status = format!(
                "[{}/{}] Ollama installed — starting service...",
                step, total_steps
            );
        }
    }

    // Step: Upgrade to GPU variant if needed (e.g., ollama -> ollama-cuda)
    if let Some(pkg) = ollama::needs_gpu_variant_upgrade() {
        info!("GPU detected but {} not installed - upgrading...", pkg);
        {
            let mut s = state.write().await;
            s.init_status = format!(
                "[{}/{}] GPU detected — upgrading to {} for acceleration...",
                step, total_steps, pkg
            );
        }
        if let Err(e) = ollama::upgrade_to_gpu_variant().await {
            warn!("Failed to upgrade to {}: {}", pkg, e);
        }
    }

    // Step: Start ollama if not running
    if !ollama::is_running().await {
        info!("Starting Ollama...");
        {
            let mut s = state.write().await;
            s.init_status = format!(
                "[{}/{}] Starting ollama service — usually takes a few seconds...",
                step, total_steps
            );
        }
        ollama::start_service().await?;
        step += 1;
    } else {
        step += 1;
    }

    // Check what models are available
    let models = ollama::list_models().await.unwrap_or_default();
    info!("Available models: {:?}", models);

    // Pick the best model that fits in available memory, preferring already-installed.
    // select_from_installed respects CPU/GPU limits — never picks a model too large to respond in time.
    let model = if !models.is_empty() {
        let selected = ollama::select_from_installed(&hw, &models);
        if models.iter().any(|m| *m == selected) {
            info!("Using installed model: {}", selected);
            selected
        } else {
            // selected is a download target — fall through to pull
            String::new()
        }
    } else { String::new() };

    let model = if !model.is_empty() {
        model
    } else {
        let best_model = best_model; // shadow to keep borrow
        // No models - pull the best one for this hardware
        info!("No models found, pulling {}...", best_model);
        let model_size_gb = extract_model_size(best_model);
        let eta = if model_size_gb >= 7 { "5–15 min" } else { "2–5 min" };
        {
            let mut s = state.write().await;
            s.init_status = format!(
                "[{}/{}] Downloading {} — estimated {} on first install...",
                step, total_steps, best_model, eta
            );
        }
        let pull_state = state.clone();
        let step_total = format!("[{}/{}] ", step, total_steps);
        ollama::pull_model_with_progress(best_model, move |msg| {
            let s = pull_state.clone();
            let prefix = step_total.clone();
            tokio::spawn(async move {
                s.write().await.init_status = format!("{}{}", prefix, msg);
            });
        }).await?;
        best_model.to_string()
    };

    info!("Using model: {}", model);

    // Final step: load model into memory and verify it responds
    {
        let mut s = state.write().await;
        s.init_status = format!(
            "[{}/{}] Loading {} into memory — cold start takes 30–120 sec...",
            total_steps, total_steps, model
        );
    }
    ollama::test_model(&model).await
        .map_err(|e| anyhow!("Model health check failed: {}", e))?;

    // Mark Ready immediately with whatever model works — don't block on upgrade
    {
        let mut state = state.write().await;
        state.ollama_running = true;
        state.model = Some(model.clone());
        state.state = DaemonState::Ready;
        state.init_status = "Ready".to_string();
    }

    // If a better model exists for this hardware, pull it in the background
    // and silently switch once it's ready — user keeps working the whole time
    let current_size = extract_model_size(&model);
    let best_size = extract_model_size(best_model);
    if current_size < best_size && !models.iter().any(|m| m == best_model) {
        let best_model_owned = best_model.to_string();
        let upgrade_state = state.clone();
        tokio::spawn(async move {
            info!(
                "Background upgrade: pulling {}B model (currently on {}B)...",
                best_size, current_size
            );
            if let Ok(()) = ollama::pull_model(&best_model_owned).await {
                // Verify it works before switching
                if ollama::test_model(&best_model_owned).await.is_ok() {
                    let mut s = upgrade_state.write().await;
                    // Only switch if we're still Ready (not mid-recovery)
                    if s.state == DaemonState::Ready {
                        info!("Background upgrade complete — switching to {}", best_model_owned);
                        s.model = Some(best_model_owned);
                    }
                }
            }
        });
    }

    // v0.0.999: Ensure GPU acceleration is active (restart Ollama if needed)
    if let Err(e) = ollama::ensure_gpu_acceleration().await {
        warn!("GPU acceleration check failed: {}", e);
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
        return 2; // round up so it's distinguishable from 1b models
    }
    1 // Default to smallest
}
