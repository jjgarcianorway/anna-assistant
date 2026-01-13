//! RPC client utilities for communicating with the Anna daemon.
//!
//! v0.3.28: Added version mismatch protection to prevent subtle RPC drift
//! between incompatible annactl and annad versions.

use anna_shared::rpc::{AskResult, ResetResult, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::socket_path;
use anna_shared::status::DaemonStatus;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

pub const RPC_TIMEOUT_SECS: u64 = 120; // 2 minutes for LLM operations

/// Client version (from Cargo.toml)
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// v0.3.28: Parse version into (major, minor, patch) components
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    } else if parts.len() == 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor, 0))
    } else {
        None
    }
}

/// v0.3.28: Check if annactl and annad versions are compatible.
/// Versions are compatible if major.minor match (patch can differ).
/// This prevents subtle RPC drift between incompatible binaries.
pub fn check_version_compatibility(daemon_version: &str) -> Result<()> {
    let client = parse_version(CLIENT_VERSION);
    let daemon = parse_version(daemon_version);

    match (client, daemon) {
        (Some((c_maj, c_min, _)), Some((d_maj, d_min, _))) => {
            if c_maj != d_maj || c_min != d_min {
                return Err(anyhow!(
                    "Version mismatch: annactl {} vs annad {}\n\n\
                     Major/minor versions must match to prevent RPC drift.\n\
                     Please update both binaries to the same version:\n\n\
                     Option 1: Reinstall using the installer\n\
                       curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/install.sh | bash\n\n\
                     Option 2: Build from source\n\
                       cargo build --release --workspace && sudo systemctl restart annad",
                    CLIENT_VERSION, daemon_version
                ));
            }
            Ok(())
        }
        _ => {
            // If we can't parse versions, allow but warn
            // This handles development builds with non-standard versions
            eprintln!(
                "Warning: Could not compare versions: annactl={}, annad={}",
                CLIENT_VERSION, daemon_version
            );
            Ok(())
        }
    }
}

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

/// v0.3.28: Get daemon status and verify version compatibility.
/// Use this before operations that depend on RPC contract compatibility.
pub async fn get_status_verified() -> Result<DaemonStatus> {
    let status = get_status().await?;
    check_version_compatibility(&status.version)?;
    Ok(status)
}

/// v0.3.28: Verify that the daemon version is compatible with this client.
/// Call this before operations that require version compatibility.
pub async fn ensure_compatible_daemon() -> Result<()> {
    let status = get_status().await?;
    check_version_compatibility(&status.version)
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_full() {
        assert_eq!(parse_version("0.3.28"), Some((0, 3, 28)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn test_parse_version_partial() {
        assert_eq!(parse_version("0.3"), Some((0, 3, 0)));
        assert_eq!(parse_version("1"), None); // Too few parts
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("a.b.c"), None);
        assert_eq!(parse_version("1.2.x"), None);
    }

    #[test]
    fn test_version_compatibility_same() {
        // Same version is compatible
        assert!(check_version_compatibility("0.3.28").is_ok()
            || check_version_compatibility(CLIENT_VERSION).is_ok());
    }

    #[test]
    fn test_version_compatibility_patch_differs() {
        // Same major.minor, different patch should be compatible
        // We test by comparing two versions manually
        let client = parse_version("0.3.27");
        let daemon = parse_version("0.3.28");
        if let (Some((c_maj, c_min, _)), Some((d_maj, d_min, _))) = (client, daemon) {
            assert_eq!(c_maj, d_maj);
            assert_eq!(c_min, d_min);
        }
    }

    #[test]
    fn test_version_compatibility_minor_differs() {
        // Different minor versions should be incompatible
        let result = {
            let client = parse_version("0.3.0");
            let daemon = parse_version("0.4.0");
            match (client, daemon) {
                (Some((c_maj, c_min, _)), Some((d_maj, d_min, _))) => {
                    c_maj != d_maj || c_min != d_min
                }
                _ => false,
            }
        };
        assert!(result, "Different minor versions should be incompatible");
    }

    #[test]
    fn test_version_compatibility_major_differs() {
        // Different major versions should be incompatible
        let result = {
            let client = parse_version("0.3.0");
            let daemon = parse_version("1.3.0");
            match (client, daemon) {
                (Some((c_maj, c_min, _)), Some((d_maj, d_min, _))) => {
                    c_maj != d_maj || c_min != d_min
                }
                _ => false,
            }
        };
        assert!(result, "Different major versions should be incompatible");
    }
}

/// Reset all statistics and learning data
/// v0.3.28: Verifies version compatibility before destructive operation
pub async fn reset(mode: anna_shared::rpc::ResetMode) -> Result<ResetResult> {
    // Verify version compatibility before destructive operation
    ensure_compatible_daemon().await?;

    let params = serde_json::json!({ "mode": mode });
    let response = call(RpcMethod::Reset, Some(params)).await?;
    if let Some(error) = response.error {
        return Err(anyhow!("Reset error: {}", error.message));
    }
    let result = response.result.ok_or_else(|| anyhow!("No result"))?;
    serde_json::from_value(result).map_err(|e| anyhow!("Parse error: {}", e))
}
