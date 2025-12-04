//! Unix socket server for annad.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anna_shared::ledger::{Ledger, LedgerEntry, LedgerEntryKind};
use anna_shared::rpc::RpcRequest;
use anna_shared::{SOCKET_PATH, STATE_DIR, UPDATE_CHECK_INTERVAL};
use anyhow::Result;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::{interval, Duration};
use tracing::{error, info};

use crate::hardware::{probe_hardware, select_model};
use crate::ollama;
use crate::rpc_handler::handle_request;
use crate::state::{create_shared_state, SharedState};

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

        // Start background tasks
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            update_check_loop(state_clone).await;
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
            let cpu_status = if state.hardware.cpu_cores >= 4 { "ok" } else { "limited" };
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

        // Select model
        let model_name = select_model(&hardware);

        // Phase: Pulling model
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("pulling_models");
        }

        if !ollama::has_model(&model_name).await {
            ollama::pull_model(&model_name).await?;
            let mut state = self.state.write().await;
            state.ledger.add(LedgerEntry::new(
                LedgerEntryKind::ModelPulled,
                model_name.clone(),
                false,
            ));
        }

        // Add model to status
        {
            let mut state = self.state.write().await;
            state.add_model(&model_name, "general", 0);
        }

        // Run benchmark
        let _throughput = ollama::benchmark(&model_name).await.unwrap_or(0.0);

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

        // Set socket permissions
        fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o660))?;

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
    let mut interval = interval(Duration::from_secs(UPDATE_CHECK_INTERVAL));

    loop {
        interval.tick().await;

        info!("Running update check...");

        // TODO: Implement actual update check against GitHub releases
        // For v0.0.1, just record the check time

        {
            let mut state = state.write().await;
            state.last_update_check = Some(Utc::now());
            state.next_update_check =
                Some(Utc::now() + chrono::Duration::seconds(UPDATE_CHECK_INTERVAL as i64));
        }

        info!("Update check complete");
    }
}
