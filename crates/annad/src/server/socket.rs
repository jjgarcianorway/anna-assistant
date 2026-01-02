//! Unix socket server implementation.
//! Handles socket creation, connection acceptance, and RPC request processing.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anna_shared::rpc::RpcRequest;
use anna_shared::socket_path;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

use crate::rpc_handler::handle_request;
use crate::state::SharedState;

use super::types::Server;

impl Server {
    /// v0.0.298: Static method so it can run before initialization completes.
    /// v0.0.386: Added socket health monitoring loop for resilience.
    pub(super) async fn run_socket_server_impl(state: SharedState) -> Result<()> {
        let socket_file = socket_path();

        // Initial socket setup
        Self::setup_socket(&socket_file)?;

        // v0.0.386: Run socket server with automatic recovery on socket loss
        loop {
            match Self::run_socket_accept_loop(&socket_file, state.clone()).await {
                Ok(()) => {
                    // Normal shutdown (shouldn't happen)
                    break;
                }
                Err(e) => {
                    error!("Socket server error: {}, attempting recovery...", e);
                    // Small delay before retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    // Try to recreate socket
                    if let Err(setup_err) = Self::setup_socket(&socket_file) {
                        error!("Failed to recreate socket: {}", setup_err);
                        // Wait longer before next retry
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }
        Ok(())
    }

    /// v0.0.386: Setup socket file with proper permissions.
    fn setup_socket(socket_file: &str) -> Result<()> {
        // Remove stale socket if exists
        let socket_path = Path::new(socket_file);
        if socket_path.exists() {
            fs::remove_file(socket_path)?;
        }
        Ok(())
    }

    /// v0.0.386: Accept loop with socket health monitoring.
    async fn run_socket_accept_loop(socket_file: &str, state: SharedState) -> Result<()> {
        let listener = UnixListener::bind(socket_file)?;
        info!(
            "Socket available at {} (daemon still initializing)",
            socket_file
        );

        // Set socket permissions: world accessible for zero-friction UX
        fs::set_permissions(socket_file, fs::Permissions::from_mode(0o666))?;

        // v0.0.386: Spawn health monitor that checks socket file exists
        let socket_file_clone = socket_file.to_string();
        let (health_tx, mut health_rx) = tokio::sync::mpsc::channel::<()>(1);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if !Path::new(&socket_file_clone).exists() {
                    warn!("Socket file disappeared! Triggering recovery...");
                    let _ = health_tx.send(()).await;
                    return;
                }
            }
        });

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let state = state.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(state, stream).await {
                                    error!("Connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                            // Check if socket file still exists
                            if !Path::new(socket_file).exists() {
                                return Err(anyhow::anyhow!("Socket file deleted"));
                            }
                        }
                    }
                }
                _ = health_rx.recv() => {
                    // Health monitor detected socket loss
                    return Err(anyhow::anyhow!("Socket file disappeared"));
                }
            }
        }
    }
}

/// Handle a single client connection.
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
