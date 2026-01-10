//! RPC client utilities for communicating with the Anna daemon.

use anna_shared::rpc::{AskResult, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::socket_path;
use anna_shared::status::DaemonStatus;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

pub const RPC_TIMEOUT_SECS: u64 = 120; // 2 minutes for LLM operations

/// Connect to the daemon
pub async fn connect() -> Result<UnixStream> {
    let socket_file = socket_path();
    let socket_path = Path::new(&socket_file);

    if !socket_path.exists() {
        return Err(anyhow!(
            "Anna daemon not running.\n\
             The socket at {} does not exist.\n\n\
             Start the daemon with: sudo systemctl start annad",
            socket_file
        ));
    }

    UnixStream::connect(socket_path).await.map_err(|e| {
        anyhow!(
            "Cannot connect to Anna daemon: {}\n\n\
             Try: sudo systemctl restart annad",
            e
        )
    })
}

/// Send an RPC request and get the response
pub async fn call(method: RpcMethod, params: Option<serde_json::Value>) -> Result<RpcResponse> {
    let mut stream = connect().await?;
    let request = RpcRequest::new(method, params);
    let request_json = serde_json::to_string(&request)?;

    // Send request
    timeout(Duration::from_secs(5), async {
        stream
            .write_all(format!("{}\n", request_json).as_bytes())
            .await
    })
    .await
    .map_err(|_| anyhow!("Timeout writing to daemon"))?
    .map_err(|e| anyhow!("Failed to write to daemon: {}", e))?;

    // Read response
    let (reader, _) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    timeout(Duration::from_secs(RPC_TIMEOUT_SECS), reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("Request timed out after {}s", RPC_TIMEOUT_SECS))?
        .map_err(|e| anyhow!("Failed to read from daemon: {}", e))?;

    serde_json::from_str(&line).map_err(|e| anyhow!("Invalid response: {}", e))
}

/// Get daemon status
pub async fn get_status() -> Result<DaemonStatus> {
    let response = call(RpcMethod::Status, None).await?;
    if let Some(error) = response.error {
        return Err(anyhow!("Status error: {}", error.message));
    }
    let result = response.result.ok_or_else(|| anyhow!("No result"))?;
    serde_json::from_value(result).map_err(|e| anyhow!("Parse error: {}", e))
}

/// Send a question and get the answer (non-streaming)
#[allow(dead_code)]
pub async fn ask(question: &str) -> Result<AskResult> {
    let params = serde_json::json!({ "question": question });
    let response = call(RpcMethod::Ask, Some(params)).await?;

    if let Some(error) = response.error {
        return Err(anyhow!("{}", error.message));
    }

    let result = response.result.ok_or_else(|| anyhow!("No result"))?;
    serde_json::from_value(result).map_err(|e| anyhow!("Parse error: {}", e))
}
