//! RPC Server - Unix socket server for daemon-client communication

use anna_common::ipc::{ConfigData, Method, Request, Response, ResponseData, StatusData};
use anna_common::{Advice, SystemFacts};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// Daemon state shared across connections
pub struct DaemonState {
    pub version: String,
    pub start_time: std::time::Instant,
    pub facts: RwLock<SystemFacts>,
    pub advice: RwLock<Vec<Advice>>,
    pub config: RwLock<ConfigData>,
}

impl DaemonState {
    pub fn new(version: String, facts: SystemFacts, advice: Vec<Advice>) -> Self {
        Self {
            version,
            start_time: std::time::Instant::now(),
            facts: RwLock::new(facts),
            advice: RwLock::new(advice),
            config: RwLock::new(ConfigData::default()),
        }
    }
}

/// Start the RPC server
pub async fn start_server(state: Arc<DaemonState>) -> Result<()> {
    // Ensure socket directory exists
    let socket_dir = Path::new(SOCKET_PATH).parent().unwrap();
    tokio::fs::create_dir_all(socket_dir)
        .await
        .context("Failed to create socket directory")?;

    // Remove old socket if it exists
    let _ = tokio::fs::remove_file(SOCKET_PATH).await;

    // Bind to Unix socket
    let listener = UnixListener::bind(SOCKET_PATH)
        .context("Failed to bind Unix socket")?;

    info!("RPC server listening on {}", SOCKET_PATH);

    // Set socket permissions (readable/writable by all users for now)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))?;
    }

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state).await {
                        error!("Connection handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read from socket")?;

        if bytes_read == 0 {
            // Connection closed
            break;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid request JSON: {}", e);
                continue;
            }
        };

        let response = handle_request(request.id, request.method, &state).await;

        let response_json = serde_json::to_string(&response)? + "\n";
        writer
            .write_all(response_json.as_bytes())
            .await
            .context("Failed to write response")?;
    }

    Ok(())
}

/// Handle a single request
async fn handle_request(id: u64, method: Method, state: &DaemonState) -> Response {
    let result = match method {
        Method::Ping => Ok(ResponseData::Ok),

        Method::Status => {
            let advice = state.advice.read().await;
            let status = StatusData {
                version: state.version.clone(),
                uptime_seconds: state.start_time.elapsed().as_secs(),
                last_telemetry_check: "Just now".to_string(), // TODO: track actual last check
                pending_recommendations: advice.len(),
            };
            Ok(ResponseData::Status(status))
        }

        Method::GetFacts => {
            let facts = state.facts.read().await.clone();
            Ok(ResponseData::Facts(facts))
        }

        Method::GetAdvice => {
            let advice = state.advice.read().await.clone();
            Ok(ResponseData::Advice(advice))
        }

        Method::ApplyAction { advice_id, dry_run } => {
            if dry_run {
                Ok(ResponseData::ActionResult {
                    success: true,
                    message: format!("Would apply action: {}", advice_id),
                })
            } else {
                // TODO: Actually execute actions
                Err("Action execution not yet implemented".to_string())
            }
        }

        Method::GetConfig => {
            let config = state.config.read().await.clone();
            Ok(ResponseData::Config(config))
        }

        Method::SetConfig { key, value } => {
            let mut config = state.config.write().await;
            match key.as_str() {
                "autonomy_tier" => {
                    if let Ok(tier) = value.parse::<u8>() {
                        if tier <= 3 {
                            config.autonomy_tier = tier;
                            Ok(ResponseData::Ok)
                        } else {
                            Err("Invalid autonomy tier (must be 0-3)".to_string())
                        }
                    } else {
                        Err("Invalid value for autonomy_tier".to_string())
                    }
                }
                "auto_update_check" => {
                    if let Ok(enabled) = value.parse::<bool>() {
                        config.auto_update_check = enabled;
                        Ok(ResponseData::Ok)
                    } else {
                        Err("Invalid value for auto_update_check".to_string())
                    }
                }
                "wiki_cache_path" => {
                    config.wiki_cache_path = value;
                    Ok(ResponseData::Ok)
                }
                _ => Err(format!("Unknown config key: {}", key)),
            }
        }
    };

    Response { id, result }
}
