//! RPC Client - Unix socket client for communicating with daemon
//!
//! Beta.237: Enhanced error categorization and resilience
//! Beta.239: Connection reuse and performance optimization

use anna_common::ipc::{Method, Request, Response, ResponseData};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Connection reuse statistics for performance monitoring
/// Beta.239: Track connection overhead and reuse patterns
/// Beta.240: Extended with RPC call success/failure tracking
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Number of new connections established
    pub connections_created: u64,
    /// Number of successful connection reuses
    pub connections_reused: u64,
    /// Number of reconnections due to broken connections
    pub reconnections: u64,
    /// Time spent establishing connections (microseconds)
    pub connect_time_us: u64,

    // Beta.240: RPC call tracking
    /// Number of successful RPC calls
    pub successful_calls: u64,
    /// Number of failed calls due to daemon unavailable
    pub failed_daemon_unavailable: u64,
    /// Number of failed calls due to permission denied
    pub failed_permission_denied: u64,
    /// Number of failed calls due to timeout
    pub failed_timeout: u64,
    /// Number of failed calls due to connection issues
    pub failed_connection: u64,
    /// Number of failed calls due to internal errors
    pub failed_internal: u64,
    /// Number of successful reconnections
    pub successful_reconnects: u64,
}

/// RPC Error categories for better error handling
/// Beta.237: Typed errors for clearer failure modes
#[derive(Debug, Clone)]
pub enum RpcErrorKind {
    /// Daemon is not running or socket doesn't exist
    DaemonUnavailable(String),
    /// Permission denied accessing socket
    PermissionDenied(String),
    /// RPC call timed out
    Timeout(String),
    /// Connection refused or broken
    ConnectionFailed(String),
    /// Unexpected internal error
    Internal(String),
}

impl RpcErrorKind {
    pub fn to_user_message(&self) -> String {
        match self {
            Self::DaemonUnavailable(msg) => format!("Daemon unavailable: {}", msg),
            Self::PermissionDenied(msg) => format!("Permission denied: {}", msg),
            Self::Timeout(msg) => format!("RPC timeout: {}", msg),
            Self::ConnectionFailed(msg) => format!("Connection failed: {}", msg),
            Self::Internal(msg) => format!("Internal error: {}", msg),
        }
    }
}

/// RPC Client for communicating with the daemon
/// Beta.239: Connection reuse - keeps connection alive across multiple calls
pub struct RpcClient {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
    socket_path: String,  // Beta.239: Store path for reconnection
    stats: ConnectionStats,  // Beta.239: Track connection metrics
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
                let mut stats = ConnectionStats::default();
                stats.connections_created = 1;
                Ok(Self { reader, writer, socket_path: path.clone(), stats })
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Daemon unavailable: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Connection timeout")),
        }
    }

    /// Connect to the daemon with retry logic and errno-specific error messages
    ///
    /// v1.16.2-alpha.2: Socket discovery order and detailed error hints
    /// Beta.239: Track connection time and stats
    pub async fn connect_with_path(socket_path: Option<&str>) -> Result<Self> {
        use std::time::Duration;
        use tokio::time::sleep;

        let path = Self::discover_socket_path(socket_path);
        let max_retries = 10;
        let mut retry_delay = Duration::from_millis(50);
        let mut last_error: Option<std::io::Error> = None;

        // Beta.239: Track connection establishment time
        let start = std::time::Instant::now();

        for attempt in 0..max_retries {
            match tokio::time::timeout(Duration::from_millis(500), UnixStream::connect(&path)).await
            {
                Ok(Ok(stream)) => {
                    let (reader, writer) = stream.into_split();
                    let reader = BufReader::new(reader);

                    // Beta.239: Record connection stats
                    let mut stats = ConnectionStats::default();
                    stats.connections_created = 1;
                    stats.connect_time_us = start.elapsed().as_micros() as u64;

                    return Ok(Self { reader, writer, socket_path: path.clone(), stats });
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

    /// Categorize an RPC error for better handling
    /// Beta.237: Type-safe error categorization
    fn categorize_error(error: &anyhow::Error) -> RpcErrorKind {
        let error_msg = error.to_string().to_lowercase();

        if error_msg.contains("socket not found") || error_msg.contains("no such file") {
            RpcErrorKind::DaemonUnavailable(error.to_string())
        } else if error_msg.contains("permission denied") {
            RpcErrorKind::PermissionDenied(error.to_string())
        } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
            RpcErrorKind::Timeout(error.to_string())
        } else if error_msg.contains("connection refused")
            || error_msg.contains("broken pipe")
            || error_msg.contains("connection reset") {
            RpcErrorKind::ConnectionFailed(error.to_string())
        } else {
            RpcErrorKind::Internal(error.to_string())
        }
    }

    /// Check if an error indicates a broken connection that should trigger reconnection
    /// Beta.239: Connection health detection
    fn is_broken_connection(error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("broken pipe")
            || error_msg.contains("connection reset")
            || error_msg.contains("failed to send request")
            || error_msg.contains("failed to read response")
    }

    /// Attempt to reconnect once (called after detecting broken connection)
    /// Beta.239: Auto-reconnect on connection failure
    async fn reconnect(&mut self) -> Result<()> {
        use std::time::Duration;

        let start = std::time::Instant::now();

        // Single reconnection attempt (no retry loop - already failed once)
        match tokio::time::timeout(Duration::from_millis(500), UnixStream::connect(&self.socket_path)).await {
            Ok(Ok(stream)) => {
                let (reader, writer) = stream.into_split();
                self.reader = BufReader::new(reader);
                self.writer = writer;

                // Beta.239: Track reconnection stats
                self.stats.reconnections += 1;
                self.stats.connect_time_us += start.elapsed().as_micros() as u64;

                Ok(())
            }
            Ok(Err(e)) => Err(Self::socket_error_with_hint(&self.socket_path, e)),
            Err(_) => anyhow::bail!("Reconnection timeout"),
        }
    }

    /// Get connection statistics (for debugging/monitoring)
    /// Beta.239: Performance instrumentation
    #[allow(dead_code)]
    pub fn get_stats(&self) -> &ConnectionStats {
        &self.stats
    }

    /// Send a request and get a response with timeout and retry
    ///
    /// Beta.235: Added timeout and exponential backoff for transient errors
    /// Beta.237: Enhanced error categorization and documentation
    /// Beta.239: Connection reuse with automatic reconnection on broken connections
    /// Beta.240: Statistics tracking for successful/failed calls
    ///
    /// # Behavior
    /// - Connection reuse: Keeps connection alive across multiple calls
    /// - Auto-reconnect: Detects broken connections and reconnects once
    /// - Request timeout: Method-specific (5s standard, 10s for expensive calls)
    /// - Retries: Up to 3 attempts with exponential backoff (50ms → 800ms)
    /// - Transient errors: Retried automatically
    /// - Permanent errors: Returned immediately
    ///
    /// # Error Categories
    /// - DaemonUnavailable: Socket not found, daemon not running
    /// - PermissionDenied: User not in anna group
    /// - Timeout: Call exceeded timeout duration
    /// - ConnectionFailed: Broken pipe, connection refused
    /// - Internal: Unexpected errors
    pub async fn call(&mut self, method: Method) -> Result<ResponseData> {
        use std::time::Duration;
        use tokio::time::sleep;

        // Beta.235: Method-specific timeout
        let timeout = match &method {
            Method::BrainAnalysis => Duration::from_secs(10), // Brain needs more time
            Method::GetHistorianSummary => Duration::from_secs(10), // Historian too
            _ => Duration::from_secs(5), // Standard timeout for other calls
        };

        // Beta.235: Exponential backoff for transient errors
        let max_retries = 3;
        let mut retry_delay = Duration::from_millis(50);
        let mut last_error: Option<anyhow::Error> = None;
        let mut reconnected = false; // Beta.239: Track if we've already tried reconnecting

        for attempt in 0..=max_retries {
            match tokio::time::timeout(timeout, self.call_inner(method.clone())).await {
                Ok(Ok(response)) => {
                    // Beta.239: Track successful connection reuse
                    if attempt == 0 && !reconnected {
                        self.stats.connections_reused += 1;
                    }
                    // Beta.240: Track successful call
                    self.stats.successful_calls += 1;
                    return Ok(response);
                }
                Ok(Err(e)) => {
                    // Beta.239: Check if this is a broken connection
                    if Self::is_broken_connection(&e) && !reconnected {
                        // Try to reconnect once
                        if let Ok(()) = self.reconnect().await {
                            self.stats.successful_reconnects += 1;  // Beta.240
                            reconnected = true;
                            // Retry this attempt after successful reconnection
                            continue;
                        }
                    }

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
                        // Beta.240: Track failure by category before returning
                        self.track_error_stats(&e);
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
                        // Beta.240: Track timeout failure
                        self.stats.failed_timeout += 1;
                        return Err(timeout_error.context(format!("After {} retries", max_retries)));
                    }
                }
            }
        }

        // Should never reach here, but handle gracefully
        let final_error = last_error.unwrap_or_else(|| anyhow::anyhow!("RPC call failed after {} retries", max_retries));
        self.track_error_stats(&final_error);  // Beta.240
        Err(final_error)
    }

    /// Beta.240: Track error statistics by category
    fn track_error_stats(&mut self, error: &anyhow::Error) {
        let kind = Self::categorize_error(error);
        match kind {
            RpcErrorKind::DaemonUnavailable(_) => self.stats.failed_daemon_unavailable += 1,
            RpcErrorKind::PermissionDenied(_) => self.stats.failed_permission_denied += 1,
            RpcErrorKind::Timeout(_) => self.stats.failed_timeout += 1,
            RpcErrorKind::ConnectionFailed(_) => self.stats.failed_connection += 1,
            RpcErrorKind::Internal(_) => self.stats.failed_internal += 1,
        }
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
