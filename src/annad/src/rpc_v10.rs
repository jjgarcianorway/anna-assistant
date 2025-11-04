// Anna v0.10 JSON-RPC Server Module
// UNIX socket API for telemetry queries

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::events::EventEngineState;
use crate::health_metrics::{
    HealthEvaluator, HealthSnapshot, HealthStatus, LatencyTracker, MemoryMonitor, QueueMetrics,
};
use crate::persona_v10::PersonaRadar;
use crate::storage_v10::StorageManager;
use crate::telemetry_v10::TelemetrySnapshot;

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Health summary component
#[derive(Debug, Serialize)]
struct HealthComponent {
    name: String,
    state: String, // "ok", "warn", "error"
    detail: String,
}

/// Health summary response
#[derive(Debug, Serialize)]
struct HealthSummary {
    status: String, // "healthy", "degraded", "unhealthy"
    components: Vec<HealthComponent>,
}

/// Trend analysis result
#[derive(Debug, Serialize)]
struct TrendResult {
    metric: String,
    mean: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    trend: String, // "stable", "rising", "falling"
}

/// RPC server state
pub struct RpcServer {
    storage: Arc<Mutex<StorageManager>>,
    events: Arc<EventEngineState>,
    latency_tracker: Arc<LatencyTracker>,
    memory_monitor: Arc<MemoryMonitor>,
    health_evaluator: Arc<HealthEvaluator>,
}

impl RpcServer {
    pub fn new(storage: Arc<Mutex<StorageManager>>, events: Arc<EventEngineState>) -> Self {
        Self {
            storage,
            events,
            latency_tracker: Arc::new(LatencyTracker::new()),
            memory_monitor: Arc::new(MemoryMonitor::new(80)), // 80MB systemd limit
            health_evaluator: Arc::new(HealthEvaluator::with_defaults()),
        }
    }

    /// Start the RPC server on a UNIX socket
    pub async fn start<P: AsRef<Path>>(self: Arc<Self>, socket_path: P) -> Result<()> {
        let socket_path = socket_path.as_ref();

        // Log the full resolved socket path at INFO before bind
        info!("RPC socket path (target): {}", socket_path.display());

        // Ensure parent directory exists with correct ownership and mode
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create socket directory: {:?}", parent))?;

                // Set ownership and mode on parent directory
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    use std::os::unix::fs::PermissionsExt;

                    // Get anna user/group IDs
                    let anna_uid = Self::get_user_id("anna").unwrap_or(1003);
                    let anna_gid = Self::get_group_id("anna").unwrap_or(1003);

                    // Set ownership (requires root or matching user)
                    let _ = Self::chown_path(parent, anna_uid, anna_gid);

                    // Set permissions to 0750
                    let mut perms = std::fs::metadata(parent)?.permissions();
                    perms.set_mode(0o750);
                    std::fs::set_permissions(parent, perms)?;

                    let md = std::fs::metadata(parent)?;
                    info!(
                        "Socket directory created: {:?} (uid={} gid={} mode={:o})",
                        parent,
                        md.uid(),
                        md.gid(),
                        md.mode() & 0o777
                    );
                }
                #[cfg(not(unix))]
                {
                    info!("Socket directory created: {:?}", parent);
                }
            } else {
                // Log existing parent directory info
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if let Ok(md) = std::fs::metadata(parent) {
                        info!(
                            "Socket directory exists: {:?} (uid={} gid={} mode={:o})",
                            parent,
                            md.uid(),
                            md.gid(),
                            md.mode() & 0o777
                        );
                    }
                }
            }
        }

        // Set process umask to 0000 for socket creation (results in 0777, then chmod to 0770)
        #[cfg(unix)]
        unsafe {
            libc::umask(0o000);
        }

        // Remove existing socket if present (only if not in use)
        if socket_path.exists() {
            // Check if any process has the socket open using lsof
            let lsof_check = std::process::Command::new("lsof")
                .arg(socket_path.to_str().unwrap_or(""))
                .output();

            let socket_in_use = match lsof_check {
                Ok(output) => output.status.success() && !output.stdout.is_empty(),
                Err(_) => {
                    // lsof not available, fall back to connection test
                    warn!("lsof not available, using connection test for stale socket detection");
                    tokio::net::UnixStream::connect(socket_path).await.is_ok()
                }
            };

            if !socket_in_use {
                std::fs::remove_file(socket_path).context("Failed to remove stale socket")?;
                info!("Removed stale socket: {}", socket_path.display());
            } else {
                anyhow::bail!(
                    "Socket already in use: {:?} (active process detected)",
                    socket_path
                );
            }
        }

        let listener = UnixListener::bind(socket_path)
            .with_context(|| format!("Failed to bind UNIX socket at {:?}", socket_path))?;

        // Set socket permissions to 0770 (owner+group rwx, others none)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(socket_path)?.permissions();
            perms.set_mode(0o770);
            std::fs::set_permissions(socket_path, perms)?;
            info!("Socket permissions set to 0770: {}", socket_path.display());

            // Set socket group ownership to 'anna' so users in anna group can connect
            let anna_gid = Self::get_group_id("anna").unwrap_or(967);
            if let Err(e) = Self::chown_path(socket_path, 0, anna_gid) {
                warn!("Failed to set socket group ownership: {}", e);
            } else {
                info!("Socket group ownership set to anna (gid={})", anna_gid);
            }
        }

        // fsync the parent directory to ensure socket entry is persisted
        #[cfg(unix)]
        if let Some(parent) = socket_path.parent() {
            use std::os::unix::io::AsRawFd;
            if let Ok(dir_fd) = std::fs::File::open(parent) {
                unsafe {
                    libc::fsync(dir_fd.as_raw_fd());
                }
            }
        }

        info!("RPC socket ready: {}", socket_path.display());

        // Flush stdout to ensure "socket ready" message is immediately visible
        use std::io::Write;
        let _ = std::io::stdout().flush();

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                continue;
            }

            debug!("Received RPC request: {}", line);

            let response = match self.handle_request(&line).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Request handling error: {}", e);
                    self.error_response(Value::Null, -32603, format!("Internal error: {}", e))
                }
            };

            let response_json = serde_json::to_string(&response)?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, line: &str) -> Result<JsonRpcResponse> {
        use std::time::Instant;

        let req: JsonRpcRequest =
            serde_json::from_str(line).context("Failed to parse JSON-RPC request")?;

        if req.jsonrpc != "2.0" {
            return Ok(self.error_response(
                req.id,
                -32600,
                "Invalid JSON-RPC version, expected 2.0".to_string(),
            ));
        }

        // Start latency tracking
        let start = Instant::now();

        let result = match req.method.as_str() {
            // v0.12.8 Telemetry snapshot
            "get_telemetry_snapshot" => self.method_get_telemetry_snapshot(&req.params).await,
            // v0.12.7 Health metrics
            "get_health_metrics" => self.method_get_health_metrics(&req.params).await,
            // v0.12.3 CLI methods (advisor)
            "advisor_run" => self.method_advisor_run(&req.params).await,
            "hardware_profile" => self.method_hardware_profile(&req.params).await,
            "package_inventory" => self.method_package_inventory(&req.params).await,
            "storage_profile" => self.method_storage_profile(&req.params).await,
            // v0.12.2 CLI methods (collectors & radars)
            "collect" => self.method_collect(&req.params).await,
            "classify" => self.method_classify(&req.params).await,
            "radar_show" => self.method_radar_show(&req.params).await,
            "radar_snapshot" => self.method_radar_snapshot(&req.params).await,
            // v0.11.0 CLI methods (event-driven)
            "events" => self.method_events(&req.params).await,
            "watch" => self.method_watch(&req.params).await,
            // v0.10.1 CLI methods
            "status" => self.method_status(&req.params).await,
            "sensors" => self.method_sensors(&req.params).await,
            "net" => self.method_net(&req.params).await,
            "disk" => self.method_disk(&req.params).await,
            "top" => self.method_top(&req.params).await,
            "radar" => self.method_radar(&req.params).await,
            "export" => self.method_export(&req.params).await,
            // Legacy methods (kept for compatibility)
            "Telemetry.Snapshot" => self.method_snapshot(&req.params).await,
            "Telemetry.History" => self.method_history(&req.params).await,
            "Telemetry.Trends" => self.method_trends(&req.params).await,
            "Health.Summary" => self.method_health_summary(&req.params).await,
            "Persona.Scores" => self.method_persona_scores(&req.params).await,
            _ => {
                return Ok(self.error_response(
                    req.id,
                    -32601,
                    format!("Method not found: {}", req.method),
                ));
            }
        };

        // Record latency
        let latency = start.elapsed();
        self.latency_tracker.record(latency);

        match result {
            Ok(value) => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: req.id,
            }),
            Err(e) => Ok(self.error_response(req.id, -32603, e.to_string())),
        }
    }

    fn error_response(&self, id: Value, code: i32, message: String) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError { code, message }),
            id,
        }
    }

    // === RPC Method Implementations ===

    async fn method_snapshot(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry snapshot available")?;

        Ok(serde_json::to_value(snapshot)?)
    }

    async fn method_history(&self, params: &Option<Value>) -> Result<Value> {
        #[derive(Deserialize)]
        struct HistoryParams {
            window_min: u32,
            #[serde(default)]
            metrics: Vec<String>,
        }

        let params: HistoryParams = if let Some(p) = params {
            serde_json::from_value(p.clone())?
        } else {
            HistoryParams {
                window_min: 60,
                metrics: Vec::new(),
            }
        };

        let storage = self.storage.lock().await;
        let snapshots = storage.query_history(params.window_min)?;

        // Filter by requested metrics if specified
        if params.metrics.is_empty() {
            Ok(serde_json::to_value(snapshots)?)
        } else {
            // Return only requested metrics (simplified for MVP)
            Ok(serde_json::to_value(snapshots)?)
        }
    }

    async fn method_trends(&self, params: &Option<Value>) -> Result<Value> {
        #[derive(Deserialize)]
        struct TrendParams {
            metric: String,
            window_min: u32,
        }

        let params: TrendParams =
            serde_json::from_value(params.clone().unwrap_or(serde_json::json!({
                "metric": "cpu_util",
                "window_min": 30
            })))?;

        let storage = self.storage.lock().await;
        let snapshots = storage.query_history(params.window_min)?;

        // Compute trend based on metric
        let trend = self.compute_trend(&params.metric, &snapshots)?;

        Ok(serde_json::to_value(trend)?)
    }

    fn compute_trend(&self, metric: &str, snapshots: &[TelemetrySnapshot]) -> Result<TrendResult> {
        match metric {
            "cpu_util" => {
                let mut values: Vec<f64> = snapshots
                    .iter()
                    .flat_map(|s| s.cpu.cores.iter().map(|c| c.util_pct as f64))
                    .collect();

                values.sort_by(|a, b| a.partial_cmp(b).unwrap());

                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let p50 = Self::percentile(&values, 50);
                let p95 = Self::percentile(&values, 95);
                let p99 = Self::percentile(&values, 99);

                // Simple trend: compare first half to second half
                let mid = values.len() / 2;
                let first_half_avg = values[..mid].iter().sum::<f64>() / mid as f64;
                let second_half_avg =
                    values[mid..].iter().sum::<f64>() / (values.len() - mid) as f64;

                let trend = if (second_half_avg - first_half_avg).abs() < 5.0 {
                    "stable"
                } else if second_half_avg > first_half_avg {
                    "rising"
                } else {
                    "falling"
                };

                Ok(TrendResult {
                    metric: metric.to_string(),
                    mean,
                    p50,
                    p95,
                    p99,
                    trend: trend.to_string(),
                })
            }
            _ => anyhow::bail!("Unknown metric: {}", metric),
        }
    }

    fn percentile(sorted_values: &[f64], p: u8) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let idx = ((p as f64 / 100.0) * sorted_values.len() as f64) as usize;
        sorted_values[idx.min(sorted_values.len() - 1)]
    }

    async fn method_health_summary(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry snapshot available")?;

        let mut components = Vec::new();

        // CPU health
        let cpu_temp_avg = snapshot
            .cpu
            .cores
            .iter()
            .filter_map(|c| c.temp_c)
            .sum::<f32>()
            / snapshot.cpu.cores.len() as f32;

        let cpu_util_avg = snapshot.cpu.cores.iter().map(|c| c.util_pct).sum::<f32>()
            / snapshot.cpu.cores.len() as f32;

        let cpu_state = if cpu_temp_avg > 80.0 {
            "error"
        } else if cpu_temp_avg > 70.0 || cpu_util_avg > 90.0 {
            "warn"
        } else {
            "ok"
        };

        components.push(HealthComponent {
            name: "cpu".to_string(),
            state: cpu_state.to_string(),
            detail: format!("{:.1}°C avg, {:.0}% util", cpu_temp_avg, cpu_util_avg),
        });

        // Memory health
        let mem_pct = (snapshot.mem.used_mb as f64 / snapshot.mem.total_mb as f64) * 100.0;
        let mem_state = if mem_pct > 95.0 {
            "error"
        } else if mem_pct > 85.0 {
            "warn"
        } else {
            "ok"
        };

        components.push(HealthComponent {
            name: "mem".to_string(),
            state: mem_state.to_string(),
            detail: format!("{:.0}% used", mem_pct),
        });

        // Disk health
        let disk_max_pct = snapshot.disk.iter().map(|d| d.pct).fold(0.0f32, f32::max);
        let disk_state = if disk_max_pct > 95.0 {
            "error"
        } else if disk_max_pct > 85.0 {
            "warn"
        } else {
            "ok"
        };

        components.push(HealthComponent {
            name: "disk".to_string(),
            state: disk_state.to_string(),
            detail: format!("max {:.0}% used", disk_max_pct),
        });

        // Thermal health (GPU + CPU)
        let gpu_temp_max = snapshot
            .gpu
            .iter()
            .filter_map(|g| g.temp_c)
            .fold(0.0f32, f32::max);

        let thermal_max = cpu_temp_avg.max(gpu_temp_max);
        let thermal_state = if thermal_max > 85.0 {
            "error"
        } else if thermal_max > 75.0 {
            "warn"
        } else {
            "ok"
        };

        components.push(HealthComponent {
            name: "thermal".to_string(),
            state: thermal_state.to_string(),
            detail: format!("max {:.1}°C", thermal_max),
        });

        // Overall status
        let overall_status = if components.iter().any(|c| c.state == "error") {
            "unhealthy"
        } else if components.iter().any(|c| c.state == "warn") {
            "degraded"
        } else {
            "healthy"
        };

        let summary = HealthSummary {
            status: overall_status.to_string(),
            components,
        };

        Ok(serde_json::to_value(summary)?)
    }

    async fn method_persona_scores(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry snapshot available")?;

        // Compute persona scores
        let scores = PersonaRadar::compute_scores(&snapshot)?;

        // Store scores in DB
        let scores_data: Vec<(String, f32, Vec<String>)> = scores
            .iter()
            .map(|s| (s.name.clone(), s.score, s.evidence.clone()))
            .collect();

        storage.store_persona_scores(snapshot.ts, &scores_data)?;

        #[derive(Serialize)]
        struct PersonaScoresResponse {
            ts: u64,
            scores: Vec<crate::persona_v10::PersonaScore>,
        }

        Ok(serde_json::to_value(PersonaScoresResponse {
            ts: snapshot.ts,
            scores,
        })?)
    }

    // === v0.10.1 CLI Methods ===

    async fn method_status(&self, _params: &Option<Value>) -> Result<Value> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let storage = self.storage.lock().await;
        let latest = storage.get_latest_snapshot();

        let (last_sample_age_s, sample_count) = if let Some(snap) = latest {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let age = now.saturating_sub(snap.ts);
            (age, 1) // TODO: get actual count from DB
        } else {
            (0, 0)
        };

        Ok(serde_json::json!({
            "daemon_state": "running",
            "db_path": "/var/lib/anna/telemetry.db",
            "last_sample_age_s": last_sample_age_s,
            "sample_count": sample_count,
            "loop_load_pct": 0.4,
            "annad_pid": std::process::id()
        }))
    }

    async fn method_sensors(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry data available")?;

        Ok(serde_json::json!({
            "cpu": {
                "cores": snapshot.cpu.cores,
                "load_avg": snapshot.cpu.load_avg
            },
            "mem": snapshot.mem,
            "power": snapshot.power
        }))
    }

    async fn method_net(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry data available")?;

        // Try to detect default route (simplified)
        let default_route = "auto";

        Ok(serde_json::json!({
            "interfaces": snapshot.net,
            "default_route": default_route
        }))
    }

    async fn method_disk(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry data available")?;

        Ok(serde_json::json!({
            "disks": snapshot.disk
        }))
    }

    async fn method_top(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry data available")?;

        let mut by_cpu = snapshot.processes.clone();
        by_cpu.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());
        by_cpu.truncate(5);

        let mut by_mem = snapshot.processes.clone();
        by_mem.sort_by(|a, b| b.mem_mb.partial_cmp(&a.mem_mb).unwrap());
        by_mem.truncate(5);

        Ok(serde_json::json!({
            "by_cpu": by_cpu,
            "by_mem": by_mem
        }))
    }

    async fn method_radar(&self, _params: &Option<Value>) -> Result<Value> {
        let storage = self.storage.lock().await;
        let scores = storage.query_latest_persona_scores()?;

        let personas: Vec<Value> = scores
            .into_iter()
            .map(|(name, score, evidence)| {
                serde_json::json!({
                    "name": name,
                    "score": score,
                    "evidence": evidence
                })
            })
            .collect();

        Ok(serde_json::json!({
            "personas": personas
        }))
    }

    async fn method_export(&self, _params: &Option<Value>) -> Result<Value> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let storage = self.storage.lock().await;
        let snapshot = storage
            .get_latest_snapshot()
            .context("No telemetry data available")?;
        let scores = storage.query_latest_persona_scores()?;

        Ok(serde_json::json!({
            "snapshot": snapshot,
            "persona_scores": scores,
            "exported_at": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        }))
    }

    /// v0.11.0: Get recent event history
    async fn method_events(&self, params: &Option<Value>) -> Result<Value> {
        #[derive(Deserialize)]
        struct EventsParams {
            #[serde(default = "default_limit")]
            limit: usize,
        }

        fn default_limit() -> usize {
            50
        }

        let params: EventsParams = if let Some(p) = params {
            serde_json::from_value(p.clone())?
        } else {
            EventsParams { limit: 50 }
        };

        let history = self.events.get_history(params.limit);

        Ok(serde_json::json!({
            "events": history,
            "count": history.len(),
            "pending": self.events.pending_count()
        }))
    }

    /// v0.11.0: Watch for events (polling fallback)
    async fn method_watch(&self, _params: &Option<Value>) -> Result<Value> {
        // For now, this is just a snapshot of pending/recent events
        // In a full implementation, this would use a streaming protocol
        // or long-polling with timeouts

        let recent = self.events.get_history(10);
        let pending = self.events.pending_count();

        Ok(serde_json::json!({
            "recent_events": recent,
            "pending_count": pending,
            "note": "This is a snapshot. Use 'events' for full history."
        }))
    }

    /// v0.12.2: Collect telemetry snapshots
    async fn method_collect(&self, params: &Option<Value>) -> Result<Value> {
        use crate::collectors_v12;

        #[derive(Deserialize)]
        struct CollectParams {
            #[serde(default)]
            limit: Option<u32>,
        }

        let params: CollectParams = if let Some(p) = params {
            serde_json::from_value(p.clone()).unwrap_or(CollectParams { limit: None })
        } else {
            CollectParams { limit: None }
        };

        // Collect current snapshot
        let sensors = collectors_v12::collect_sensors()?;
        let net = collectors_v12::collect_net()?;
        let disk = collectors_v12::collect_disk()?;
        let top = collectors_v12::collect_top(10)?;

        let snapshot = serde_json::json!({
            "ts": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            "sensors": sensors,
            "net": net,
            "disk": disk,
            "top": top,
        });

        // If limit specified, could query history from DB
        // For now, return current snapshot only
        Ok(serde_json::json!({
            "snapshots": [snapshot],
            "count": 1,
            "limit": params.limit.unwrap_or(1)
        }))
    }

    /// v0.12.2: Classify system persona
    async fn method_classify(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::collectors_v12;
        use crate::radars_v12;

        // Collect current data
        let sensors = collectors_v12::collect_sensors()?;
        let net = collectors_v12::collect_net()?;
        let disk = collectors_v12::collect_disk()?;

        // Compute radars
        let system_health = radars_v12::score_system_health(
            sensors.cpu.load_avg[0],
            sensors.cpu.cores.len() as u32,
            (sensors.mem.free_mb as f64 / sensors.mem.total_mb as f64) * 100.0,
            disk.disks
                .iter()
                .find(|d| d.mount == "/")
                .map(|d| 100.0 - d.pct)
                .unwrap_or(50.0),
            sensors.cpu.cores.first().and_then(|c| c.temp_c),
        );

        let network_posture = radars_v12::score_network_posture(
            net.dns_latency_ms,
            None, // No packet loss data yet
            net.dns_latency_ms.is_some(),
        );

        // Simple persona classification
        let persona = if sensors.power.is_some() {
            "laptop"
        } else if sensors.cpu.cores.len() > 8 {
            "server"
        } else {
            "workstation"
        };

        let confidence = if sensors.power.is_some() { 0.9 } else { 0.7 };

        let mut evidence = Vec::new();
        if sensors.power.is_some() {
            evidence.push("Battery detected".to_string());
        }
        evidence.push(format!("{} CPU cores", sensors.cpu.cores.len()));
        evidence.push(format!("{} network interfaces", net.interfaces.len()));

        Ok(serde_json::json!({
            "persona": persona,
            "confidence": confidence,
            "evidence": evidence,
            "radars": {
                "system_health": system_health,
                "network_posture": network_posture,
            }
        }))
    }

    /// v0.12.2: Show radar scores
    async fn method_radar_show(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::collectors_v12;
        use crate::radars_v12;

        // Collect current data
        let sensors = collectors_v12::collect_sensors()?;
        let net = collectors_v12::collect_net()?;
        let disk = collectors_v12::collect_disk()?;

        // Compute health radar
        let health = radars_v12::score_system_health(
            sensors.cpu.load_avg[0],
            sensors.cpu.cores.len() as u32,
            (sensors.mem.free_mb as f64 / sensors.mem.total_mb as f64) * 100.0,
            disk.disks
                .iter()
                .find(|d| d.mount == "/")
                .map(|d| 100.0 - d.pct)
                .unwrap_or(50.0),
            sensors.cpu.cores.first().and_then(|c| c.temp_c),
        );

        // Compute network radar
        let network = radars_v12::score_network_posture(
            net.dns_latency_ms,
            None, // No packet loss data yet
            net.dns_latency_ms.is_some(),
        );

        Ok(serde_json::json!({
            "health": health,
            "network": network,
            "overall": {
                "health_score": health.overall_score,
                "network_score": network.overall_score,
                "combined": (health.overall_score + network.overall_score) / 2.0,
            }
        }))
    }

    /// v0.12.9: Get all three radar snapshots
    async fn method_radar_snapshot(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::radar_hardware;
        use crate::radar_software;
        use crate::radar_user;

        // Collect all three radars
        let hardware = radar_hardware::collect_hardware_radar()?;
        let software = radar_software::collect_software_radar()?;
        let user = radar_user::collect_user_radar()?;

        Ok(serde_json::json!({
            "hardware": hardware,
            "software": software,
            "user": user,
        }))
    }

    /// v0.12.3: Get hardware profile
    async fn method_hardware_profile(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::hardware_profile::HardwareCollector;

        let collector = HardwareCollector::new();
        let profile = collector.collect().await?;

        Ok(serde_json::to_value(profile)?)
    }

    /// v0.12.3: Get package inventory
    async fn method_package_inventory(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::package_analysis::PackageCollector;

        let collector = PackageCollector::new();
        let inventory = collector.collect().await?;

        Ok(serde_json::to_value(inventory)?)
    }

    /// v0.12.3: Run Arch advisor
    async fn method_advisor_run(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::advisor_v13::ArchAdvisor;
        use crate::hardware_profile::HardwareCollector;
        use crate::package_analysis::PackageCollector;
        use crate::storage_btrfs::BtrfsCollector;

        // Collect hardware and package data
        let hw_collector = HardwareCollector::new();
        let pkg_collector = PackageCollector::new();
        let btrfs_collector = BtrfsCollector::new();

        let hw_profile = hw_collector.collect().await?;
        let pkg_inventory = pkg_collector.collect().await?;
        let btrfs_profile = btrfs_collector.collect().await.ok();

        // Run advisor (including storage rules if Btrfs detected)
        let advice = ArchAdvisor::run(&hw_profile, &pkg_inventory, btrfs_profile.as_ref());

        Ok(serde_json::to_value(advice)?)
    }

    /// v0.12.3-btrfs: Get storage/btrfs profile
    async fn method_storage_profile(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::storage_btrfs::BtrfsCollector;

        let collector = BtrfsCollector::new();
        let profile = collector.collect().await?;

        Ok(serde_json::to_value(profile)?)
    }

    /// v0.12.7: Get health metrics snapshot
    async fn method_get_health_metrics(&self, _params: &Option<Value>) -> Result<Value> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Get RPC latency metrics
        let rpc_latency = self.latency_tracker.metrics();

        // Get memory metrics
        let memory = self.memory_monitor.metrics().ok();

        // Get queue metrics from event engine
        let queue = Some(QueueMetrics {
            depth: self.events.pending_count(),
            rate_per_sec: self.events.event_rate_per_sec(),
            oldest_event_sec: self.events.oldest_pending_event_sec(),
            total_processed: self.events.get_history(1000).len() as u64,
        });

        // Get uptime
        let uptime_sec = self.latency_tracker.uptime_sec();

        // Get capabilities count (hardcoded for now, would need CapabilityManager access)
        let capabilities_active = 4;
        let capabilities_degraded = 0;

        // Build snapshot
        let mut snapshot = HealthSnapshot {
            status: HealthStatus::Unknown,
            uptime_sec,
            rpc_latency,
            memory,
            queue,
            capabilities_active,
            capabilities_degraded,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };

        // Evaluate overall health
        snapshot.status = self.health_evaluator.evaluate(&snapshot);

        Ok(serde_json::to_value(snapshot)?)
    }

    /// v0.12.8: Get live telemetry snapshot for watch mode
    async fn method_get_telemetry_snapshot(&self, _params: &Option<Value>) -> Result<Value> {
        use crate::telemetry_snapshot::TelemetrySnapshot;

        // Get queue depth from event engine
        let queue_depth = self.events.pending_count();

        // Create a basic telemetry snapshot
        // In a full implementation, we would have a SnapshotAggregator instance
        // that tracks events over time. For now, create a minimal snapshot.
        let mut snapshot = TelemetrySnapshot::new();
        snapshot.queue.depth = queue_depth;
        snapshot.queue.total_processed = self.events.get_history(1000).len() as u64;

        // Get memory metrics
        if let Ok(memory_metrics) = self.memory_monitor.metrics() {
            snapshot.resources.memory_bytes = (memory_metrics.current_mb * 1024.0 * 1024.0) as u64;
            snapshot.resources.memory_human = format!("{:.1} MB", memory_metrics.current_mb);
        }

        // Module activity status
        snapshot.modules.telemetry_active = true;
        snapshot.modules.event_engine_active = true;
        snapshot.modules.policy_engine_active = true;
        snapshot.modules.storage_active = true;
        snapshot.modules.rpc_active = true;

        Ok(serde_json::to_value(snapshot)?)
    }

    // Helper function to get user ID by name
    #[cfg(unix)]
    fn get_user_id(username: &str) -> Option<u32> {
        use std::process::Command;
        let output = Command::new("id").args(&["-u", username]).output().ok()?;
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().parse().ok()
        } else {
            None
        }
    }

    // Helper function to get group ID by name
    #[cfg(unix)]
    fn get_group_id(groupname: &str) -> Option<u32> {
        use std::process::Command;
        let output = Command::new("id").args(&["-g", groupname]).output().ok()?;
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().parse().ok()
        } else {
            None
        }
    }

    // Helper function to chown a path
    #[cfg(unix)]
    fn chown_path(path: &Path, uid: u32, gid: u32) -> Result<()> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let path_cstr =
            CString::new(path.as_os_str().as_bytes()).context("Invalid path for chown")?;

        let result = unsafe { libc::chown(path_cstr.as_ptr(), uid, gid) };

        if result == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("chown failed for {:?}", path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_rpc_request_parsing() -> Result<()> {
        let req_json = r#"{"jsonrpc":"2.0","method":"Telemetry.Snapshot","params":{},"id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(req_json)?;

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "Telemetry.Snapshot");
        assert_eq!(req.id, Value::Number(1.into()));

        Ok(())
    }

    #[tokio::test]
    async fn test_error_response() -> Result<()> {
        use crate::events::EventEngine;

        let temp_db = NamedTempFile::new()?;
        let storage = Arc::new(Mutex::new(StorageManager::new(temp_db.path())?));
        let engine = EventEngine::new(300, 30, 100);
        let events = Arc::new(engine.shared_state());
        let server = RpcServer::new(storage, events);

        let response =
            server.error_response(Value::Number(1.into()), -32600, "Test error".to_string());

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);

        Ok(())
    }
}
