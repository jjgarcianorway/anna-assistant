//! Server initialization logic.
//! Handles directory setup, Ollama installation, hardware probing, and model selection.

use std::fs;
use std::path::Path;

use anna_shared::ledger::{LedgerEntry, LedgerEntryKind};
use anna_shared::{socket_path, state_dir};
use anyhow::Result;
use tracing::{error, info, warn};

use std::process::Command as StdCommand;

use crate::auto_select;
use crate::gpu_setup;
use crate::hardware::probe_hardware;
use crate::ollama;
use crate::state::SharedState;

use super::types::Server;

/// v0.0.823: Clean up manually installed Ollama files FIRST before any install attempts
/// This is critical for systems where curl-installed Ollama blocks pacman packages
fn cleanup_manual_ollama_install() {
    // Check if ollama binary exists but isn't owned by pacman (manual install)
    let is_manual = {
        let binary_exists = std::path::Path::new("/usr/bin/ollama").exists();
        if !binary_exists {
            // Also check for leftover lib files even if binary was deleted
            std::path::Path::new("/usr/lib/ollama").exists()
        } else {
            StdCommand::new("pacman")
                .args(["-Qo", "/usr/bin/ollama"])
                .output()
                .map(|o| !o.status.success())
                .unwrap_or(false)
        }
    };

    if !is_manual {
        return;
    }

    info!("Cleaning up manually installed Ollama files...");

    // Stop and kill ollama first
    let _ = StdCommand::new("systemctl").args(["stop", "ollama"]).output();
    let _ = StdCommand::new("pkill").args(["-9", "ollama"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Remove all files that conflict with pacman
    let paths = [
        "/usr/bin/ollama",
        "/usr/lib/ollama",
        "/usr/lib/systemd/system/ollama.service",
        "/usr/lib/sysusers.d/ollama.conf",
        "/usr/lib/tmpfiles.d/ollama.conf",
        "/usr/share/licenses/ollama",
        "/usr/share/ollama",
    ];

    for path in &paths {
        let p = std::path::Path::new(path);
        if p.exists() {
            if p.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(p) {
                    warn!("Failed to remove dir {}: {}", path, e);
                } else {
                    info!("Removed {}", path);
                }
            } else if let Err(e) = std::fs::remove_file(p) {
                warn!("Failed to remove file {}: {}", path, e);
            } else {
                info!("Removed {}", path);
            }
        }
    }
}

impl Server {
    /// Create directories for state and socket file.
    pub(super) async fn setup_directories(&self) -> Result<()> {
        let state_path = state_dir();
        let socket_file = socket_path();

        // Create state directory
        fs::create_dir_all(&state_path)?;

        // Create run directory for socket
        // v0.0.291: Safe path handling - avoid panic on malformed paths
        let socket_dir = Path::new(&socket_file)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid socket path: {}", socket_file))?;
        fs::create_dir_all(socket_dir)?;

        // Remove stale socket
        if Path::new(&socket_file).exists() {
            fs::remove_file(&socket_file)?;
        }

        // Record in ledger
        let mut state = self.state.write().await;
        state.ledger.add(LedgerEntry::new(
            LedgerEntryKind::DirectoryCreated,
            state_path,
            true,
        ));

        Ok(())
    }

    /// Initialize the daemon: install Ollama, probe hardware, select models.
    pub(super) async fn initialize(&self) -> Result<()> {
        info!("Initializing daemon...");

        // v0.0.823: FIRST cleanup any manual Ollama install files before anything else
        // This prevents crashes from conflicting files blocking pacman install
        cleanup_manual_ollama_install();

        // Phase: Installing Ollama
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("installing_ollama");
        }

        // v0.0.822: Make Ollama install non-fatal so daemon can start and auto-update
        if !ollama::is_installed() {
            match ollama::install().await {
                Ok(()) => {
                    let mut state = self.state.write().await;
                    state.ledger.add(LedgerEntry::new(
                        LedgerEntryKind::PackageInstalled,
                        "ollama".to_string(),
                        true,
                    ));
                }
                Err(e) => {
                    error!("Failed to install Ollama (will retry): {}", e);
                    // Don't crash - continue startup so auto-update can run
                }
            }
        }

        // v0.0.818: Ensure GPU acceleration is configured before starting Ollama
        // This checks for NVIDIA GPU, installs cuda/ollama-cuda if needed,
        // and adds ollama user to video/render groups
        // Track which packages were actually installed for proper uninstall
        let cuda_was_installed = gpu_setup::is_cuda_installed();
        let ollama_cuda_was_installed = gpu_setup::is_ollama_cuda_installed();

        match gpu_setup::ensure_gpu_acceleration() {
            Ok(true) => {
                info!("GPU acceleration configured");
                let mut state = self.state.write().await;
                // v0.0.818: Track each package separately for proper uninstall
                if !cuda_was_installed && gpu_setup::is_cuda_installed() {
                    state.ledger.add(LedgerEntry::new(
                        LedgerEntryKind::PackageInstalled,
                        "cuda".to_string(),
                        true,
                    ));
                }
                if !ollama_cuda_was_installed && gpu_setup::is_ollama_cuda_installed() {
                    state.ledger.add(LedgerEntry::new(
                        LedgerEntryKind::PackageInstalled,
                        "ollama-cuda".to_string(),
                        true,
                    ));
                }
            }
            Ok(false) => {
                info!("No GPU detected, using CPU-only mode");
            }
            Err(e) => {
                warn!("GPU setup failed: {} - continuing with CPU-only mode", e);
            }
        }

        // Start Ollama if not running
        if !ollama::is_running().await {
            ollama::start_service().await?;
        }

        // Update Ollama status
        {
            let mut state = self.state.write().await;
            state.ollama = ollama::get_status().await;
        }

        // Phase: Probing hardware
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("probing_hardware");
        }

        let hardware = probe_hardware()?;
        {
            let mut state = self.state.write().await;
            state.hardware = hardware.clone();
        }

        // Phase: Benchmarking
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("benchmarking");
        }

        // Set benchmark result based on hardware
        {
            let mut state = self.state.write().await;
            let cpu_status = if state.hardware.cpu_cores >= 4 {
                "ok"
            } else {
                "limited"
            };
            let ram_status = if state.hardware.ram_bytes >= 8 * 1024 * 1024 * 1024 {
                "ok"
            } else {
                "limited"
            };
            let gpu_status = state
                .hardware
                .gpu
                .as_ref()
                .map(|_| "detected")
                .unwrap_or("none");
            state.set_benchmark_result(cpu_status, ram_status, gpu_status);
        }

        // v0.0.310: Mark daemon as ready EARLY with PullingModels state
        // This allows deterministic answers while models load in background
        {
            let mut state = self.state.write().await;
            state.llm.state = anna_shared::status::LlmState::PullingModels;
            state.state = anna_shared::status::DaemonState::Running;
            state.set_llm_phase("starting_model_setup");

            // Set default models from config (will be updated when auto-select completes)
            state.llm.translator_model = Some(state.config.llm.translator_model.clone());
            state.llm.specialist_model = Some(state.config.llm.specialist_model.clone());
            state.llm.junior_model = Some(state.config.llm.specialist_model.clone());
            state.llm.senior_model = Some(state.config.llm.senior_model.clone());
        }

        info!("Daemon ready for deterministic queries (models loading in background)");

        // v0.0.310: Model selection and pulling now happens in background
        let state_for_models = self.state.clone();
        tokio::spawn(async move {
            info!("Background model setup starting");
            if let Err(e) = Self::setup_models_background(state_for_models).await {
                error!("Background model setup failed: {}", e);
            }
        });

        // v0.0.310: Daemon is already marked Running - initialization complete
        // Model setup continues in background
        info!("Daemon initialized (model setup in background)");
        Ok(())
    }

    /// v0.0.310: Background model selection and pulling.
    /// Runs after daemon is already marked Ready, so deterministic answers work immediately.
    pub(super) async fn setup_models_background(state: SharedState) -> Result<()> {
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

        // Mark models as fully ready
        {
            let mut state_write = state.write().await;
            state_write.set_llm_ready();
        }

        info!("Background model setup complete - LLM fully ready");
        Ok(())
    }
}
