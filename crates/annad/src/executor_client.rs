//! Synchronous RPC client for anna-executor.
//!
//! Connects to /run/anna/anna-executor.sock and sends a single request,
//! reads the response, then closes the connection.
//!
//! Uses blocking std I/O — appropriate for callers that are already on
//! a std::thread (scheduler, startup self-healing).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use anna_shared::paths::paths;
use tracing::warn;

// Re-export the request/response types from anna-executor's protocol.
// We duplicate them here as a local copy to avoid a workspace dependency
// on the anna-executor crate (which is a binary crate, not a library).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorRequest {
    RestartService { name: String },
    CleanJournal { keep_days: u32 },
    CleanPackageCache { keep_versions: u32 },
    CleanTmpFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorResponse {
    Ok { output: String },
    Denied { reason: String },
    Error { message: String },
}

/// Send a single request to anna-executor and return the response.
///
/// Returns `Err` if the socket is unavailable (executor not running) or
/// if the wire protocol fails. The caller decides how to degrade.
pub fn executor_rpc(request: &ExecutorRequest) -> Result<ExecutorResponse, String> {
    let socket_path = paths().executor_socket_file();

    let stream = UnixStream::connect(&socket_path).map_err(|e| {
        format!("anna-executor unavailable ({}): {}", socket_path.display(), e)
    })?;

    // 5 second timeout — executor should respond in milliseconds
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("set_read_timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("set_write_timeout: {}", e))?;

    let mut stream_write = stream.try_clone().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(request).map_err(|e| e.to_string())?;
    stream_write
        .write_all(format!("{}\n", json).as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    drop(stream_write);

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("read: {}", e))?;
    if response_line.is_empty() {
        return Err("executor closed connection without response".to_string());
    }

    serde_json::from_str::<ExecutorResponse>(&response_line)
        .map_err(|e| format!("deserialize response: {}", e))
}

/// Convenience wrapper: log a warning on failure, return bool success.
pub fn executor_rpc_logged(request: &ExecutorRequest) -> bool {
    match executor_rpc(request) {
        Ok(ExecutorResponse::Ok { .. }) => true,
        Ok(ExecutorResponse::Denied { reason }) => {
            warn!("anna-executor denied request: {}", reason);
            false
        }
        Ok(ExecutorResponse::Error { message }) => {
            warn!("anna-executor error: {}", message);
            false
        }
        Err(e) => {
            warn!("anna-executor RPC failed: {}", e);
            false
        }
    }
}
