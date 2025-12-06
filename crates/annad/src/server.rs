//! Unix socket server for annad.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anna_shared::ledger::{Ledger, LedgerEntry, LedgerEntryKind};
use anna_shared::rpc::RpcRequest;
use anna_shared::{SOCKET_PATH, STATE_DIR, VERSION};
use anyhow::Result;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use crate::hardware::probe_hardware;
use crate::health::health_check_loop;
use crate::ollama;
use crate::rpc_handler::handle_request;
use crate::state::{create_shared_state, SharedState};
use crate::update::{check_latest_version, is_newer_version, perform_update};

pub struct Server {
    state: SharedState,
}

impl Server {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            state: create_shared_state(),
        })
    }

    pub async fn run(&self) -> Result<()> {
        // Ensure directories exist
        self.setup_directories().await?;

        // Initialize daemon
        self.initialize().await?;

        // Start update check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            update_check_loop(state_clone).await;
        });

        // Start health check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            health_check_loop(state_clone).await;
        });

        // Start socket server
        self.run_socket_server().await
    }

    async fn setup_directories(&self) -> Result<()> {
        // Create state directory
        fs::create_dir_all(STATE_DIR)?;

        // Create run directory for socket
        let socket_dir = Path::new(SOCKET_PATH).parent().unwrap();
        fs::create_dir_all(socket_dir)?;

        // Remove stale socket
        if Path::new(SOCKET_PATH).exists() {
            fs::remove_file(SOCKET_PATH)?;
        }

        // Record in ledger
        let mut state = self.state.write().await;
        state.ledger.add(LedgerEntry::new(
            LedgerEntryKind::DirectoryCreated,
            STATE_DIR.to_string(),
            true,
        ));

        Ok(())
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing daemon...");

        // Load existing ledger if available
        {
            let mut state = self.state.write().await;
            if let Ok(ledger) = Ledger::load() {
                state.ledger = ledger;
                info!("Loaded existing ledger");
            }
        }

        // Phase: Installing Ollama
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("installing_ollama");
        }

        if !ollama::is_installed() {
            ollama::install().await?;
            let mut state = self.state.write().await;
            state.ledger.add(LedgerEntry::new(
                LedgerEntryKind::PackageInstalled,
                "ollama".to_string(),
                true,
            ));
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

        // Get required models from config
        let required_models = {
            let state = self.state.read().await;
            state.config.required_models()
        };

        // Get model roles from config
        let (translator_model, specialist_model, supervisor_model) = {
            let state = self.state.read().await;
            (
                state.config.llm.translator_model.clone(),
                state.config.llm.specialist_model.clone(),
                state.config.llm.supervisor_model.clone(),
            )
        };

        // Phase: Pulling models
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("pulling_models");
        }

        for model_name in &required_models {
            if !ollama::has_model(model_name).await {
                info!("Pulling model: {}", model_name);
                ollama::pull_model(model_name).await?;
                let mut state = self.state.write().await;
                state.ledger.add(LedgerEntry::new(
                    LedgerEntryKind::ModelPulled,
                    model_name.clone(),
                    false,
                ));
            } else {
                info!("Model already available: {}", model_name);
            }
        }

        // Add models to status with their roles
        {
            let mut state = self.state.write().await;
            state.add_model(&translator_model, "translator", 0);
            state.add_model(&specialist_model, "specialist", 0);
            if supervisor_model != translator_model && supervisor_model != specialist_model {
                state.add_model(&supervisor_model, "supervisor", 0);
            }
        }

        // Run benchmark on specialist model (primary inference model)
        let _throughput = ollama::benchmark(&specialist_model).await.unwrap_or(0.0);

        // Save ledger
        {
            let state = self.state.read().await;
            state.ledger.save()?;
        }

        // Mark ready
        {
            let mut state = self.state.write().await;
            state.set_llm_ready();
        }

        info!("Daemon initialized and ready");
        Ok(())
    }

    async fn run_socket_server(&self) -> Result<()> {
        let listener = UnixListener::bind(SOCKET_PATH)?;
        info!("Listening on {}", SOCKET_PATH);

        // Set socket permissions: world accessible for zero-friction UX
        // The anna group is used for directory permissions, not socket
        fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666))?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(state, stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(state: SharedState, stream: UnixStream) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32700, "message": format!("Parse error: {}", e)},
                    "id": null
                });
                writer
                    .write_all(format!("{}\n", error_response).as_bytes())
                    .await?;
                line.clear();
                continue;
            }
        };

        let response = handle_request(state.clone(), request).await;
        let response_json = serde_json::to_string(&response)?;
        writer
            .write_all(format!("{}\n", response_json).as_bytes())
            .await?;

        line.clear();
    }

    Ok(())
}

async fn update_check_loop(state: SharedState) {
    use anna_shared::update_ledger::{
        load_update_ledger, save_update_ledger, UpdateCheckEntry, UpdateCheckResult,
    };
    use std::time::Instant;

    // Get check interval from state
    let check_interval = {
        let state = state.read().await;
        state.update.check_interval_secs
    };

    let mut interval = interval(Duration::from_secs(check_interval));

    // Set initial next_check time
    {
        let mut state = state.write().await;
        state.update.next_check_at =
            Some(Utc::now() + chrono::Duration::seconds(check_interval as i64));
    }

    loop {
        interval.tick().await;

        info!("Checking for updates...");
        let check_start = Instant::now();

        // Check GitHub for latest version
        match check_latest_version().await {
            Ok(latest_version) => {
                let duration_ms = check_start.elapsed().as_millis() as u64;
                let should_update = is_newer_version(VERSION, &latest_version);

                // Write to update ledger (v0.0.29)
                let ledger_result = if should_update {
                    UpdateCheckResult::UpdateAvailable {
                        version: latest_version.clone(),
                    }
                } else {
                    UpdateCheckResult::UpToDate
                };
                let entry = UpdateCheckEntry::new(VERSION, ledger_result, duration_ms)
                    .with_remote_tag(format!("v{}", latest_version));
                let mut ledger = load_update_ledger();
                ledger.push(entry);
                if let Err(e) = save_update_ledger(&ledger) {
                    warn!("Failed to save update ledger: {}", e);
                }

                {
                    let mut state = state.write().await;
                    let now = Utc::now();
                    state.update.last_check_at = Some(now);
                    state.update.next_check_at =
                        Some(now + chrono::Duration::seconds(check_interval as i64));
                    state.update.latest_version = Some(latest_version.clone());
                    state.update.latest_checked_at = Some(now);
                    state.update.update_available = should_update;
                    state.update.check_state = anna_shared::status::UpdateCheckState::Success;
                }

                if should_update {
                    info!("New version available: {} -> {}", VERSION, latest_version);

                    // Check if auto-update is enabled
                    let auto_update_enabled = {
                        let state = state.read().await;
                        state.update.enabled
                    };

                    if auto_update_enabled {
                        info!("Auto-update enabled, performing update...");
                        match perform_update(&latest_version).await {
                            Ok(()) => {
                                info!("Update initiated, daemon will restart");
                                // Record successful install in ledger
                                let entry = UpdateCheckEntry::new(
                                    VERSION,
                                    UpdateCheckResult::Installed {
                                        version: latest_version.clone(),
                                    },
                                    0,
                                );
                                let mut ledger = load_update_ledger();
                                ledger.push(entry);
                                let _ = save_update_ledger(&ledger);
                            }
                            Err(e) => {
                                error!("Auto-update failed: {}", e);
                                // Record failure in ledger
                                let entry = UpdateCheckEntry::new(
                                    VERSION,
                                    UpdateCheckResult::Failed {
                                        reason: e.to_string(),
                                    },
                                    0,
                                );
                                let mut ledger = load_update_ledger();
                                ledger.push(entry);
                                let _ = save_update_ledger(&ledger);

                                let mut state = state.write().await;
                                state.last_error = Some(format!("Auto-update failed: {}", e));
                            }
                        }
                    } else {
                        info!("Auto-update disabled, skipping");
                    }
                } else {
                    info!("Already on latest version: {}", VERSION);
                }
            }
            Err(e) => {
                let duration_ms = check_start.elapsed().as_millis() as u64;
                warn!("Failed to check for updates: {}", e);

                // Record failure in ledger
                let entry = UpdateCheckEntry::new(
                    VERSION,
                    UpdateCheckResult::Failed {
                        reason: e.to_string(),
                    },
                    duration_ms,
                );
                let mut ledger = load_update_ledger();
                ledger.push(entry);
                let _ = save_update_ledger(&ledger);

                // v0.0.72: On failure, preserve last known version but mark as failed
                let mut state = state.write().await;
                let now = Utc::now();
                state.update.last_check_at = Some(now);
                state.update.next_check_at =
                    Some(now + chrono::Duration::seconds(check_interval as i64));
                state.update.check_state = anna_shared::status::UpdateCheckState::Failed;
                // NOTE: We do NOT clear latest_version - preserve last known good value
            }
        }
    }
}
