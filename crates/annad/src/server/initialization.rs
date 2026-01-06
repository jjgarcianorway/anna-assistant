//! Server initialization logic.
//! Handles directory setup, Ollama installation, hardware probing, and model selection.
//! v0.0.825: Background model setup extracted to model_setup.rs

use std::fs;
use std::path::Path;

use anna_shared::ledger::{LedgerEntry, LedgerEntryKind};
use anna_shared::{socket_path, state_dir};
use anyhow::Result;
use tracing::{error, info, warn};

use std::process::Command as StdCommand;

use crate::gpu_setup;
use crate::hardware::probe_hardware;
use crate::ollama;
use crate::state::SharedState;

use super::model_setup::setup_models_background;
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
        self.init_ollama().await?;

        // Phase: Configure GPU acceleration
        self.init_gpu().await;

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
        self.init_hardware().await?;

        // Phase: Benchmarking
        self.init_benchmark().await;

        // v0.0.310: Mark daemon as ready EARLY with PullingModels state
        self.mark_daemon_ready().await;

        info!("Daemon ready for deterministic queries (models loading in background)");

        // v0.0.310: Model selection and pulling now happens in background
        let state_for_models = self.state.clone();
        tokio::spawn(async move {
            info!("Background model setup starting");
            if let Err(e) = setup_models_background(state_for_models).await {
                error!("Background model setup failed: {}", e);
            }
        });

        // v0.0.310: Daemon is already marked Running - initialization complete
        info!("Daemon initialized (model setup in background)");
        Ok(())
    }

    /// Install Ollama if not already installed.
    async fn init_ollama(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.set_llm_phase("installing_ollama");
        drop(state);

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
        Ok(())
    }

    /// Configure GPU acceleration if available.
    async fn init_gpu(&self) {
        // v0.0.818: Ensure GPU acceleration is configured before starting Ollama
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
    }

    /// Probe hardware and store results.
    async fn init_hardware(&self) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("probing_hardware");
        }

        let hardware = probe_hardware()?;
        {
            let mut state = self.state.write().await;
            state.hardware = hardware;
        }
        Ok(())
    }

    /// Run benchmark based on hardware.
    async fn init_benchmark(&self) {
        let mut state = self.state.write().await;
        state.set_llm_phase("benchmarking");

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

    /// Mark daemon as ready for queries while models load in background.
    async fn mark_daemon_ready(&self) {
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
}
