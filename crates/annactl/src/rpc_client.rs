//! RPC Client - Unix socket client for communicating with daemon

use anna_common::ipc::{Method, Request, Response, ResponseData};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// RPC Client for communicating with the daemon
pub struct RpcClient {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl RpcClient {
    /// Discover socket path with fallback chain
    ///
    /// Priority:
    /// 1. Explicit --socket flag (passed as argument)
    /// 2. $ANNAD_SOCKET environment variable
    /// 3. /run/anna/anna.sock (default)
    /// 4. /run/anna.sock (fallback)
    pub fn discover_socket_path(explicit_path: Option<&str>) -> String {
        if let Some(path) = explicit_path {
            return path.to_string();
        }

        if let Ok(path) = std::env::var("ANNAD_SOCKET") {
            return path;
        }

        // Try default first
        if std::path::Path::new("/run/anna/anna.sock").exists() {
            return "/run/anna/anna.sock".to_string();
        }

        // Fallback
        "/run/anna.sock".to_string()
    }

    /// Quick connect (single attempt, short timeout) - for availability checks
    pub async fn connect_quick(socket_path: Option<&str>) -> Result<Self> {
        use std::time::Duration;

        let path = Self::discover_socket_path(socket_path);

        match tokio::time::timeout(Duration::from_millis(200), UnixStream::connect(&path)).await {
            Ok(Ok(stream)) => {
                let (reader, writer) = stream.into_split();
                let reader = BufReader::new(reader);
                Ok(Self { reader, writer })
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Daemon unavailable: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Connection timeout")),
        }
    }

    /// Connect to the daemon with retry logic and errno-specific error messages
    ///
    /// v1.16.2-alpha.2: Socket discovery order and detailed error hints
    pub async fn connect_with_path(socket_path: Option<&str>) -> Result<Self> {
        use std::time::Duration;
        use tokio::time::sleep;

        let path = Self::discover_socket_path(socket_path);
        let max_retries = 10;
        let mut retry_delay = Duration::from_millis(50);
        let mut last_error: Option<std::io::Error> = None;

        for attempt in 0..max_retries {
            match tokio::time::timeout(Duration::from_millis(500), UnixStream::connect(&path)).await
            {
                Ok(Ok(stream)) => {
                    let (reader, writer) = stream.into_split();
                    let reader = BufReader::new(reader);
                    return Ok(Self { reader, writer });
                }
                Ok(Err(e)) if attempt == max_retries - 1 => {
                    // Last attempt - provide errno-specific hint
                    return Err(Self::socket_error_with_hint(&path, e));
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(500));
                    }
                }
                Err(_) => {
                    if attempt < max_retries - 1 {
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(500));
                    }
                }
            }
        }

        if let Some(e) = last_error {
            return Err(Self::socket_error_with_hint(&path, e));
        }

        anyhow::bail!("Failed to connect to daemon at {}. Is annad running?", path)
    }

    /// Backward compatibility: connect without explicit path
    pub async fn connect() -> Result<Self> {
        Self::connect_with_path(None).await
    }

    /// Generate errno-specific error hint
    fn socket_error_with_hint(path: &str, error: std::io::Error) -> anyhow::Error {
        use std::io::ErrorKind;

        let hint = match error.kind() {
            ErrorKind::NotFound => {
                format!(
                    "Socket not found at {}. Is annad running?\n\
                     Try: sudo systemctl status annad",
                    path
                )
            }
            ErrorKind::PermissionDenied => {
                // Phase 3.8: Enhanced permission error with current user
                let current_user =
                    std::env::var("USER").unwrap_or_else(|_| "YOUR_USERNAME".to_string());
                format!(
                    "❌ Permission denied accessing Anna daemon socket.\n\
                     \n\
                     Socket path: {}\n\
                     \n\
                     Your user account needs to be added to the 'anna' group.\n\
                     \n\
                     Fix (run these commands):\n\
                     \n\
                     1. Add your user to the 'anna' group:\n\
                        sudo usermod -aG anna {}\n\
                     \n\
                     2. Apply the group change (choose one):\n\
                        newgrp anna              # Apply immediately (current shell)\n\
                        # OR logout and login     # Apply permanently\n\
                     \n\
                     3. Verify the fix:\n\
                        groups | grep anna       # Should show 'anna' in output\n\
                        annactl status           # Should work now\n\
                     \n\
                     Debug info:\n\
                        ls -la {}        # Check socket permissions\n\
                        namei -l {}      # Trace path permissions",
                    path, current_user, path, path
                )
            }
            ErrorKind::ConnectionRefused | ErrorKind::TimedOut => {
                format!(
                    "Daemon not responding on {}.\n\
                     Socket exists but daemon not accepting connections.\n\
                     Try: sudo systemctl restart annad",
                    path
                )
            }
            _ => {
                format!(
                    "Failed to connect to daemon at {}: {}\n\
                     Check: sudo systemctl status annad",
                    path, error
                )
            }
        };

        anyhow::Error::new(error).context(hint)
    }

    /// Send a request and get a response
    /// Beta.235: Added timeout and exponential backoff for transient errors
    pub async fn call(&mut self, method: Method) -> Result<ResponseData> {
        use std::time::Duration;
        use tokio::time::sleep;

        // Beta.235: Wrap entire RPC call in timeout (not just connection)
        // Default timeout: 5 seconds for normal calls
        // Brain analysis and other expensive operations get more time
        let timeout = match &method {
            Method::BrainAnalysis => Duration::from_secs(10), // Brain needs more time
            Method::GetHistorianSummary => Duration::from_secs(10), // Historian too
            _ => Duration::from_secs(5), // Standard timeout for other calls
        };

        // Beta.235: Exponential backoff for transient errors
        let max_retries = 3;
        let mut retry_delay = Duration::from_millis(50);
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..=max_retries {
            match tokio::time::timeout(timeout, self.call_inner(method.clone())).await {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    // Check if error is transient (worth retrying)
                    let error_msg = e.to_string();
                    let is_transient = error_msg.contains("Failed to send request")
                        || error_msg.contains("Failed to read response")
                        || error_msg.contains("broken pipe");

                    if is_transient && attempt < max_retries {
                        last_error = Some(e);
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(800));
                        continue;
                    } else {
                        // Non-transient error or final retry - propagate immediately
                        return Err(e);
                    }
                }
                Err(_) => {
                    // Timeout error - transient, retry
                    let timeout_error = anyhow::anyhow!("RPC call timed out after {:?}", timeout);
                    if attempt < max_retries {
                        last_error = Some(timeout_error);
                        sleep(retry_delay).await;
                        retry_delay = (retry_delay * 2).min(Duration::from_millis(800));
                        continue;
                    } else {
                        return Err(timeout_error.context(format!("After {} retries", max_retries)));
                    }
                }
            }
        }

        // Should never reach here, but handle gracefully
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("RPC call failed after {} retries", max_retries)))
    }

    /// Inner call implementation (without timeout wrapper)
    async fn call_inner(&mut self, method: Method) -> Result<ResponseData> {
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

    /// Get system profile for adaptive intelligence (Phase 3.0)
    /// Citation: [linux:proc][systemd:detect-virt][xdg:session]
    pub async fn get_profile(&mut self) -> Result<ResponseData> {
        self.call(Method::GetProfile).await
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

    /// Repair failed probes (Phase 0.7)
    /// Citation: [archwiki:System_maintenance]
    pub async fn repair_probe(&mut self, probe: String, dry_run: bool) -> Result<ResponseData> {
        self.call(Method::RepairProbe { probe, dry_run }).await
    }

    /// Perform guided installation (Phase 0.8)
    /// Citation: [archwiki:Installation_guide]
    pub async fn perform_install(
        &mut self,
        config: anna_common::ipc::InstallConfigData,
        dry_run: bool,
    ) -> Result<ResponseData> {
        self.call(Method::PerformInstall { config, dry_run }).await
    }

    /// Check system health (Phase 0.9)
    /// Citation: [archwiki:System_maintenance]
    pub async fn system_health(&mut self) -> Result<ResponseData> {
        self.call(Method::SystemHealth).await
    }

    /// Perform system update (Phase 0.9)
    /// Citation: [archwiki:System_maintenance#Upgrading_the_system]
    pub async fn system_update(&mut self, dry_run: bool) -> Result<ResponseData> {
        self.call(Method::SystemUpdate { dry_run }).await
    }

    /// Perform system audit (Phase 0.9)
    /// Citation: [archwiki:Security]
    pub async fn system_audit(&mut self) -> Result<ResponseData> {
        self.call(Method::SystemAudit).await
    }

    /// Get sentinel status (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    pub async fn sentinel_status(&mut self) -> Result<ResponseData> {
        self.call(Method::SentinelStatus).await
    }

    /// Get sentinel metrics (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    pub async fn sentinel_metrics(&mut self) -> Result<ResponseData> {
        self.call(Method::SentinelMetrics).await
    }

    /// Get sentinel configuration (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    pub async fn sentinel_get_config(&mut self) -> Result<ResponseData> {
        self.call(Method::SentinelGetConfig).await
    }

    /// Set sentinel configuration (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    pub async fn sentinel_set_config(
        &mut self,
        config: anna_common::ipc::SentinelConfigData,
    ) -> Result<ResponseData> {
        self.call(Method::SentinelSetConfig { config }).await
    }

    /// Get conscience pending actions (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    pub async fn conscience_review(&mut self) -> Result<ResponseData> {
        self.call(Method::ConscienceReview).await
    }

    /// Get conscience decision explanation (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    pub async fn conscience_explain(&mut self, decision_id: String) -> Result<ResponseData> {
        self.call(Method::ConscienceExplain { decision_id }).await
    }

    /// Approve a flagged conscience action (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    pub async fn conscience_approve(&mut self, decision_id: String) -> Result<ResponseData> {
        self.call(Method::ConscienceApprove { decision_id }).await
    }

    /// Reject a flagged conscience action (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    pub async fn conscience_reject(&mut self, decision_id: String) -> Result<ResponseData> {
        self.call(Method::ConscienceReject { decision_id }).await
    }

    /// Trigger manual conscience introspection (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    pub async fn conscience_introspect(&mut self) -> Result<ResponseData> {
        self.call(Method::ConscienceIntrospect).await
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
        let socket_path = Self::discover_socket_path(None);
        let stream = UnixStream::connect(&socket_path)
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
