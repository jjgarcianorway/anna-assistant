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

    // Detect hardware first
    let hw = ollama::detect_hardware();
    let best_model = ollama::select_best_model(&hw);
    info!("Best model for this hardware: {}", best_model);

    // Store hardware info in state
    {
        let mut s = state.write().await;
        s.gpu = Some(format!("{:?}", hw.gpu_type));
        s.vram_mb = if hw.vram_mb > 0 { Some(hw.vram_mb) } else { None };
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
    let model = if models.iter().any(|m| m.starts_with(best_model.split(':').next().unwrap_or(best_model))) {
        // We have a version of the best model family
        models.iter()
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
        info!("Upgrading from {}B to {}B model for better performance...", current_size, best_size);
        if let Err(e) = ollama::pull_model(best_model).await {
            warn!("Failed to pull better model, continuing with {}: {}", model, e);
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
fn extract_model_size(model: &str) -> u32 {
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
    if model_lower.contains("14b") { return 14; }
    if model_lower.contains("13b") { return 13; }
    if model_lower.contains("7b") { return 7; }
    if model_lower.contains("8b") { return 8; }
    if model_lower.contains("3b") { return 3; }
    if model_lower.contains("1.5b") { return 1; }
    1 // Default to smallest
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

    // Handle streaming requests separately
    if matches!(request.method, RpcMethod::AskStreaming) {
        return handle_streaming_request(request, state, writer).await;
    }

    let response = handle_request(request, state).await;
    let response_json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", response_json).as_bytes()).await?;

    Ok(())
}

/// Handle a streaming AskStreaming request
async fn handle_streaming_request(
    request: RpcRequest,
    state: SharedState,
    mut writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    use anna_shared::rpc::StreamingResponse;
    use crate::core_loop::execute_question_streaming;

    let question = request
        .params
        .as_ref()
        .and_then(|p| p.get("question"))
        .and_then(|q| q.as_str())
        .unwrap_or("");

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Get model from state
    let model = {
        let state = state.read().await;
        match &state.model {
            Some(m) => m.clone(),
            None => {
                let response = StreamingResponse::Error {
                    message: "Daemon not ready - no model available".to_string(),
                };
                let json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                return Ok(());
            }
        }
    };

    // Execute with streaming
    if let Err(e) = execute_question_streaming(&model, question, &mut writer).await {
        let response = StreamingResponse::Error {
            message: format!("Execution error: {}", e),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    }

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
        RpcMethod::AskStreaming => {
            // This is handled separately in handle_streaming_request
            // Should not reach here, but provide a fallback
            RpcResponse::error(&request.id, -32603, "Use streaming connection for AskStreaming")
        }
    }
}
