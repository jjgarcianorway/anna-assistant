//! RPC Client - Unix socket client for communicating with daemon

use anna_common::ipc::{Method, Request, Response, ResponseData};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/anna.sock";

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// RPC Client for communicating with the daemon
pub struct RpcClient {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl RpcClient {
    /// Connect to the daemon with retry logic (Beta.108, rc.13.1 enhanced errors)
    ///
    /// Problem: After daemon restart, socket recreation takes time
    /// Old behavior: Single connect attempt → 30s timeout → failure
    /// New behavior: Retry with exponential backoff for 5s max
    ///
    /// rc.13.1: Detect EACCES and provide group enrollment hint
    pub async fn connect() -> Result<Self> {
        use std::time::Duration;
        use tokio::time::sleep;

        let max_retries = 10;
        let mut retry_delay = Duration::from_millis(50); // Start with 50ms
        let mut last_error: Option<std::io::Error> = None;

        for attempt in 0..max_retries {
            match tokio::time::timeout(
                Duration::from_millis(500), // 500ms connect timeout per attempt
                UnixStream::connect(SOCKET_PATH),
            )
            .await
            {
                Ok(Ok(stream)) => {
                    // Success!
                    let (reader, writer) = stream.into_split();
                    let reader = BufReader::new(reader);
                    return Ok(Self { reader, writer });
                }
                Ok(Err(e)) if attempt == max_retries - 1 => {
                    // Last attempt failed - check for permission denied
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        return Err(e).context(
                            "Permission denied accessing /run/anna/anna.sock.\n\
                             Ensure your user is in group 'anna':\n  \
                             sudo usermod -aG anna \"$USER\" && newgrp anna"
                        );
                    }
                    // Other error - give up
                    return Err(e).context(
                        "Failed to connect to daemon after 10 retries. Is annad running?",
                    );
                }
                Ok(Err(e)) => {
                    // Store error for final check
                    last_error = Some(e);
                    // Connection failed - retry if not last attempt
                    if attempt < max_retries - 1 {
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(500));
                    }
                }
                Err(_) => {
                    // Timeout - retry if not last attempt
                    if attempt < max_retries - 1 {
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(500));
                    }
                }
            }
        }

        // Check if last error was permission denied
        if let Some(e) = last_error {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(anyhow::Error::new(e)).context(
                    "Permission denied accessing /run/anna/anna.sock.\n\
                     Ensure your user is in group 'anna':\n  \
                     sudo usermod -aG anna \"$USER\" && newgrp anna"
                );
            }
        }

        anyhow::bail!("Failed to connect to daemon. Is annad running?")
    }

    /// Send a request and get a response
    pub async fn call(&mut self, method: Method) -> Result<ResponseData> {
        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

        let request = Request { id, method };

        // Send request
        let request_json = serde_json::to_string(&request)? + "\n";
        self.writer
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send request")?;

        // Read response
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .await
            .context("Failed to read response")?;

        let response: Response = serde_json::from_str(&line).context("Failed to parse response")?;

        if response.id != id {
            anyhow::bail!("Response ID mismatch");
        }

        response
            .result
            .map_err(|e| anyhow::anyhow!("RPC error: {}", e))
    }

    /// Ping the daemon (health check)
    #[allow(dead_code)]
    pub async fn ping(&mut self) -> Result<()> {
        self.call(Method::Ping).await?;
        Ok(())
    }

    /// Get system state detection (Phase 0.3b)
    /// Citation: [archwiki:system_maintenance]
    pub async fn get_state(&mut self) -> Result<ResponseData> {
        self.call(Method::GetState).await
    }

    /// Get available capabilities for current state (Phase 0.3b)
    /// Citation: [archwiki:system_maintenance]
    pub async fn get_capabilities(&mut self) -> Result<ResponseData> {
        self.call(Method::GetCapabilities).await
    }

    /// Run health probes (Phase 0.5b)
    /// Citation: [archwiki:System_maintenance]
    pub async fn health_run(
        &mut self,
        timeout_ms: u64,
        probes: Vec<String>,
    ) -> Result<ResponseData> {
        self.call(Method::HealthRun { timeout_ms, probes }).await
    }

    /// Get health summary (Phase 0.5b)
    /// Citation: [archwiki:System_maintenance]
    #[allow(dead_code)]
    pub async fn health_summary(&mut self) -> Result<ResponseData> {
        self.call(Method::HealthSummary).await
    }

    /// Get recovery plans (Phase 0.5b)
    /// Citation: [archwiki:General_troubleshooting]
    pub async fn recovery_plans(&mut self) -> Result<ResponseData> {
        self.call(Method::RecoveryPlans).await
    }

    /// Call a method with streaming response support
    /// Creates a dedicated connection for streaming to avoid blocking the main client
    /// Returns a receiver that yields ResponseData chunks until StreamEnd
    #[allow(dead_code)]
    pub async fn call_streaming(
        &mut self,
        method: Method,
    ) -> Result<tokio::sync::mpsc::Receiver<ResponseData>> {
        // Create a dedicated connection for this streaming call
        let stream = UnixStream::connect(SOCKET_PATH)
            .await
            .context("Failed to connect for streaming")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        let request = Request { id, method };

        // Send request
        let request_json = serde_json::to_string(&request)? + "\n";
        writer
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send streaming request")?;

        // Create channel for responses
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Spawn task to read responses
        tokio::spawn(async move {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) | Ok(_) if line.is_empty() => break, // Connection closed
                    Ok(_) => {
                        let response: Response = match serde_json::from_str(&line) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("Failed to parse streaming response: {}", e);
                                break;
                            }
                        };

                        if response.id != id {
                            eprintln!("Response ID mismatch in streaming");
                            break;
                        }

                        match response.result {
                            Ok(data) => {
                                let is_end = matches!(data, ResponseData::StreamEnd { .. });

                                if tx.send(data).await.is_err() {
                                    break; // Receiver dropped
                                }

                                if is_end {
                                    break; // Streaming complete
                                }
                            }
                            Err(e) => {
                                eprintln!("RPC error in streaming: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read streaming response: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
