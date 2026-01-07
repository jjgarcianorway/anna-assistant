//! Unix socket server for handling client requests.

use anna_shared::rpc::{RpcMethod, RpcRequest, RpcResponse};
use anna_shared::socket_path;
use anna_shared::status::DaemonState;
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

use crate::core_loop::execute_question;
use crate::ollama;
use crate::state::SharedState;
use crate::update_loop::update_check_loop;

/// The daemon server
pub struct Server {
    state: SharedState,
}

impl Server {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<()> {
        // Setup socket
        let socket_path = socket_path();
        self.setup_socket(&socket_path).await?;

        // Start initialization in background
        let init_state = self.state.clone();
        tokio::spawn(async move {
            if let Err(e) = initialize(init_state).await {
                error!("Initialization failed: {}", e);
            }
        });

        // Start update check loop
        let update_state = self.state.clone();
        tokio::spawn(async move {
            update_check_loop(update_state).await;
        });

        // Run socket server
        self.run_socket_server(&socket_path).await
    }

    async fn setup_socket(&self, socket_path: &str) -> Result<()> {
        let path = Path::new(socket_path);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }

        // Remove old socket if it exists
        if path.exists() {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    async fn run_socket_server(&self, socket_path: &str) -> Result<()> {
        let listener = UnixListener::bind(socket_path)?;
        info!("Listening on {}", socket_path);

        // Set socket permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o666);
            std::fs::set_permissions(socket_path, perms)?;
        }

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state).await {
                            warn!("Connection error: {}", e);
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

/// Initialize the daemon (install/start ollama, pull model)
async fn initialize(state: SharedState) -> Result<()> {
    info!("Initializing...");

    // Install ollama if needed
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

    // Use any available model, prefer qwen/llama
    let model = if let Some(m) = models.iter().find(|m| m.contains("qwen")) {
        m.clone()
    } else if let Some(m) = models.iter().find(|m| m.contains("llama")) {
        m.clone()
    } else if let Some(m) = models.iter().find(|m| m.contains("mistral")) {
        m.clone()
    } else if let Some(m) = models.iter().find(|m| m.contains("gemma")) {
        m.clone()
    } else if !models.is_empty() {
        models[0].clone()
    } else {
        // No models - need to pull one
        info!("No models found, pulling qwen2.5:3b...");
        ollama::pull_model("qwen2.5:3b").await?;
        "qwen2.5:3b".to_string()
    };

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

/// Handle a single client connection
async fn handle_connection(stream: UnixStream, state: SharedState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader.read_line(&mut line).await?;

    let request: RpcRequest = match serde_json::from_str(&line) {
        Ok(r) => r,
        Err(e) => {
            let response = RpcResponse::error("", -32700, &format!("Parse error: {}", e));
            let response_json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", response_json).as_bytes()).await?;
            return Ok(());
        }
    };

    let response = handle_request(request, state).await;
    let response_json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", response_json).as_bytes()).await?;

    Ok(())
}

/// Handle an RPC request
async fn handle_request(request: RpcRequest, state: SharedState) -> RpcResponse {
    match request.method {
        RpcMethod::Status => {
            let state = state.read().await;
            let status = state.to_status();
            match serde_json::to_value(&status) {
                Ok(result) => RpcResponse::success(&request.id, result),
                Err(e) => RpcResponse::error(&request.id, -32603, &format!("Internal error: {}", e)),
            }
        }
        RpcMethod::Ask => {
            // Extract the question from params
            let question = request
                .params
                .as_ref()
                .and_then(|p| p.get("question"))
                .and_then(|q| q.as_str())
                .unwrap_or("");

            if question.is_empty() {
                return RpcResponse::error(&request.id, -32602, "Missing 'question' parameter");
            }

            // Get model from state
            let model = {
                let state = state.read().await;
                match &state.model {
                    Some(m) => m.clone(),
                    None => {
                        return RpcResponse::error(
                            &request.id,
                            -32603,
                            "Daemon not ready - no model available",
                        );
                    }
                }
            };

            // Execute the question
            match execute_question(&model, question).await {
                Ok(result) => match serde_json::to_value(&result) {
                    Ok(v) => RpcResponse::success(&request.id, v),
                    Err(e) => {
                        RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                    }
                },
                Err(e) => RpcResponse::error(&request.id, -32603, &format!("Execution error: {}", e)),
            }
        }
    }
}
