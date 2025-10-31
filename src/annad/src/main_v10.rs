// Anna v0.10 Daemon - Read-Only Telemetry Collection
// Unprivileged systemd service: observe first, understand second, act never (in v0.10)

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod persona_v10;
mod rpc_v10;
mod storage_v10;
mod telemetry_v10;

use persona_v10::PersonaRadar;
use rpc_v10::RpcServer;
use storage_v10::StorageManager;
use telemetry_v10::TelemetryCollector;

/// Configuration for Anna daemon
struct DaemonConfig {
    db_path: PathBuf,
    socket_path: PathBuf,
    poll_interval_secs: u64,
    poll_jitter_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("/var/lib/anna/telemetry.db"),
            socket_path: PathBuf::from("/run/anna/annad.sock"),
            poll_interval_secs: 30,
            poll_jitter_secs: 5,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();

    info!("Anna v0.10 daemon starting (telemetry-first MVP)");

    // Load configuration
    let config = DaemonConfig::default();

    // Ensure directories exist
    ensure_directories(&config)?;

    // Initialize storage
    info!("Initializing storage at {:?}", config.db_path);
    let storage = Arc::new(Mutex::new(
        StorageManager::new(&config.db_path)
            .context("Failed to initialize storage")?,
    ));

    // Start RPC server
    info!("Starting RPC server at {:?}", config.socket_path);
    let rpc_server = Arc::new(RpcServer::new(Arc::clone(&storage)));
    let rpc_socket = config.socket_path.clone();

    tokio::spawn(async move {
        if let Err(e) = rpc_server.start(rpc_socket).await {
            error!("RPC server failed: {}", e);
        }
    });

    // Wait for RPC server to bind
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Start telemetry collection loop
    info!(
        "Starting telemetry collection ({}s ±{}s interval)",
        config.poll_interval_secs, config.poll_jitter_secs
    );

    run_collection_loop(storage, config).await?;

    Ok(())
}

/// Ensure required directories exist
fn ensure_directories(config: &DaemonConfig) -> Result<()> {
    // /var/lib/anna
    if let Some(parent) = config.db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create state directory")?;
    }

    // /run/anna
    if let Some(parent) = config.socket_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create runtime directory")?;
    }

    Ok(())
}

/// Main telemetry collection loop
async fn run_collection_loop(
    storage: Arc<Mutex<StorageManager>>,
    config: DaemonConfig,
) -> Result<()> {
    let mut collector = TelemetryCollector::new();
    let mut failure_count = 0u32;
    let mut backoff_secs = config.poll_interval_secs;

    loop {
        // Collect telemetry
        match collector.collect() {
            Ok(snapshot) => {
                info!(
                    "Collected telemetry: {} CPU cores, {:.1}GB RAM, {} disks, {} net ifaces",
                    snapshot.cpu.cores.len(),
                    snapshot.mem.total_mb as f64 / 1024.0,
                    snapshot.disk.len(),
                    snapshot.net.len()
                );

                // Store snapshot
                if let Err(e) = storage.lock().await.store_snapshot(&snapshot) {
                    error!("Failed to store snapshot: {}", e);
                    failure_count += 1;
                } else {
                    // Reset backoff on success
                    failure_count = 0;
                    backoff_secs = config.poll_interval_secs;

                    // Compute persona scores every 10 collections (~5 minutes)
                    static mut COLLECTION_COUNT: u32 = 0;
                    unsafe {
                        COLLECTION_COUNT += 1;
                        if COLLECTION_COUNT % 10 == 0 {
                            if let Err(e) = update_persona_scores(&storage, &snapshot).await {
                                warn!("Failed to update persona scores: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Telemetry collection failed: {}", e);
                failure_count += 1;

                // Log alert
                if let Err(log_err) = storage.lock().await.log_alert(
                    "ERROR",
                    "telemetry_collector",
                    &format!("Collection failed: {}", e),
                ) {
                    error!("Failed to log alert: {}", log_err);
                }
            }
        }

        // Exponential backoff on failures (max 5 minutes)
        if failure_count > 0 {
            backoff_secs = (config.poll_interval_secs * 2u64.pow(failure_count))
                .min(300);
            warn!(
                "Backing off to {}s after {} failures",
                backoff_secs, failure_count
            );
        }

        // Sleep with jitter
        let jitter = rand::random::<f64>() * config.poll_jitter_secs as f64;
        let sleep_duration = Duration::from_secs_f64(
            backoff_secs as f64 + jitter - (config.poll_jitter_secs as f64 / 2.0)
        );

        time::sleep(sleep_duration).await;
    }
}

/// Update persona scores
async fn update_persona_scores(
    storage: &Arc<Mutex<StorageManager>>,
    snapshot: &telemetry_v10::TelemetrySnapshot,
) -> Result<()> {
    info!("Computing persona scores...");

    let scores = PersonaRadar::compute_scores(snapshot)?;

    let scores_data: Vec<(String, f32, Vec<String>)> = scores
        .iter()
        .map(|s| (s.name.clone(), s.score, s.evidence.clone()))
        .collect();

    storage.lock().await.store_persona_scores(snapshot.ts, &scores_data)?;

    // Log top 3 scores
    let mut sorted_scores = scores.clone();
    sorted_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    info!(
        "Top personas: {} ({:.1}), {} ({:.1}), {} ({:.1})",
        sorted_scores[0].name,
        sorted_scores[0].score,
        sorted_scores[1].name,
        sorted_scores[1].score,
        sorted_scores[2].name,
        sorted_scores[2].score,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = DaemonConfig::default();
        assert_eq!(config.poll_interval_secs, 30);
        assert_eq!(config.poll_jitter_secs, 5);
        assert_eq!(config.db_path, PathBuf::from("/var/lib/anna/telemetry.db"));
        assert_eq!(config.socket_path, PathBuf::from("/run/anna/annad.sock"));
    }
}
