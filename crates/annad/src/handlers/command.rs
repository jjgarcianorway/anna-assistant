//! Command execution handler.
//! v0.0.312: Runs user-approved shell commands with daemon privileges

use super::types::*;
use anna_shared::rpc::{CommandExecutionResult, ExecuteCommandParams};
use std::process::Command;
use std::time::Instant;

/// Handle ExecuteCommand request - runs a user-approved shell command
/// This runs as the daemon (with elevated privileges) so commands like sudo work
pub async fn handle_execute_command(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: ExecuteCommandParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    info!(
        "Executing user-approved command: {} (request_id: {})",
        params.command, params.request_id
    );

    let start = Instant::now();

    // Execute the command via sh -c to support pipes, sudo, etc.
    let output = Command::new("sh").arg("-c").arg(&params.command).output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(output) => {
            let result = CommandExecutionResult {
                success: output.status.success(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms,
            };

            if result.success {
                info!("Command completed successfully in {}ms", duration_ms);
            } else {
                warn!(
                    "Command failed with exit code {} in {}ms",
                    result.exit_code, duration_ms
                );
            }

            RpcResponse::success(id, serde_json::to_value(&result).unwrap())
        }
        Err(e) => {
            error!("Failed to execute command: {}", e);
            let result = CommandExecutionResult {
                success: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Failed to spawn command: {}", e),
                duration_ms,
            };
            RpcResponse::success(id, serde_json::to_value(&result).unwrap())
        }
    }
}
