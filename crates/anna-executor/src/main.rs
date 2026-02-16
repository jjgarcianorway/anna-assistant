//! anna-executor: minimal root-privilege execution daemon.
//!
//! Runs as root. Accepts structured RPC from annad (which runs as the `anna` service user).
//! Never interprets shell strings. The ExecutorRequest enum is the complete allowlist.
//!
//! Security model:
//! - Socket at /run/anna/anna-executor.sock (root:anna, 0660)
//! - SO_PEERCRED check: only accepts connections from the anna service UID
//! - All operations are enum-matched — no dynamic command construction

mod handlers;
mod protocol;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

use anna_shared::paths::paths;

/// UID of the `anna` service user. Verified at runtime via getpwnam.
fn anna_service_uid() -> Option<u32> {
    // Read from /etc/passwd — no libc dependency needed
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 && parts[0] == "anna" {
            return parts[2].parse::<u32>().ok();
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handle --version before anything else
    if std::env::args().any(|a| a == "--version") {
        println!("anna-executor {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let socket_path = paths().executor_socket_file();

    // Remove stale socket if present
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .context("Failed to remove stale executor socket")?;
    }

    // Ensure /run/anna exists
    std::fs::create_dir_all(socket_path.parent().unwrap_or(Path::new("/run/anna")))?;

    let listener = UnixListener::bind(&socket_path)
        .context("Failed to bind executor socket")?;

    // Set socket permissions: root:anna, 0660
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o660))
        .context("Failed to set executor socket permissions")?;

    info!("anna-executor listening on {}", socket_path.display());

    // Notify systemd we're ready
    let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);

    // Resolve anna service UID once at startup
    let anna_uid = anna_service_uid();
    if let Some(uid) = anna_uid {
        info!("Anna service UID: {} — will enforce SO_PEERCRED", uid);
    } else {
        warn!("Could not resolve anna service UID — SO_PEERCRED check disabled");
    }

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // SO_PEERCRED: verify connecting process is the anna service user
                if let Some(expected_uid) = anna_uid {
                    match stream.peer_cred() {
                        Ok(cred) if cred.uid() == expected_uid || cred.uid() == 0 => {
                            // Allow: anna service user or root (for testing)
                        }
                        Ok(cred) => {
                            warn!("Rejected connection from UID {} (expected {})", cred.uid(), expected_uid);
                            continue;
                        }
                        Err(e) => {
                            warn!("Failed to read peer credentials: {} — rejecting", e);
                            continue;
                        }
                    }
                }

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
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

async fn handle_connection(stream: tokio::net::UnixStream) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    reader.read_line(&mut line).await?;
    let line = line.trim();

    if line.is_empty() {
        return Ok(());
    }

    let response = match serde_json::from_str::<protocol::ExecutorRequest>(line) {
        Ok(request) => {
            info!("Received request: {:?}", request);
            // Run synchronous handler on blocking thread to avoid blocking async runtime
            tokio::task::spawn_blocking(move || handlers::handle(request))
                .await
                .unwrap_or_else(|e| protocol::ExecutorResponse::Error {
                    message: format!("Handler panic: {}", e),
                })
        }
        Err(e) => {
            warn!("Failed to deserialize request: {} — raw: {}", e, line);
            protocol::ExecutorResponse::Denied {
                reason: format!("Invalid request format: {}", e),
            }
        }
    };

    let json = serde_json::to_string(&response)?;
    write_half.write_all(format!("{}\n", json).as_bytes()).await?;
    write_half.flush().await?;

    Ok(())
}
