// Anna v0.10.1 Daemon - Pure Telemetry Observer
// Unprivileged systemd service: collect, classify, never act

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod advisor_v13;
mod capabilities;
mod collectors_v12;
mod doctor;
mod doctor_handler;
mod events;
mod hardware_profile;
mod integrity;
mod listeners;
mod package_analysis;
mod persona_v10;
mod policy;
mod radars_v12;
mod rpc_v10;
mod storage_btrfs;
mod storage_v10;
mod telemetry_v10;

use capabilities::CapabilityManager;
use doctor::DoctorApply;
use doctor_handler::DoctorHandlerImpl;
use events::EventEngine;
use integrity::IntegrityWatchdog;
use persona_v10::PersonaRadar;
use policy::PolicyEngine;
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
    // Check for --doctor-apply mode
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--doctor-apply".to_string()) {
        let verbose = args.contains(&"--verbose".to_string());
        let doctor = DoctorApply::new(verbose);
        return doctor.apply();
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();

    info!(
        "Anna v{} daemon starting (event-driven intelligence)",
        env!("CARGO_PKG_VERSION")
    );

    // Log effective uid/gid
    #[cfg(unix)]
    {
        let euid = unsafe { libc::geteuid() };
        let egid = unsafe { libc::getegid() };
        info!("Running as uid={}, gid={}", euid, egid);
    }

    // Verify running as anna user
    if let Ok(user) = std::env::var("USER") {
        if user != "anna" && user != "root" {
            warn!("Running as user '{}' (expected 'anna')", user);
        }
    }

    // Check capabilities on startup
    match CapabilityManager::new() {
        Ok(cap_mgr) => {
            let checks = cap_mgr.check_all();
            let active = checks
                .iter()
                .filter(|c| c.status == capabilities::ModuleStatus::Active)
                .count();
            let degraded = checks
                .iter()
                .filter(|c| c.status == capabilities::ModuleStatus::Degraded)
                .count();
            info!("Capabilities: {} active, {} degraded", active, degraded);

            for check in checks
                .iter()
                .filter(|c| c.status == capabilities::ModuleStatus::Degraded)
            {
                if check.required {
                    warn!(
                        "Required module '{}' degraded: {}",
                        check.module_name,
                        check.reason.as_ref().unwrap_or(&"Unknown".to_string())
                    );
                }
            }
        }
        Err(e) => {
            warn!("Failed to load capability registry: {}", e);
        }
    }

    // Load configuration
    let config = DaemonConfig::default();

    // Ensure directories exist
    ensure_directories(&config)?;

    // Initialize storage
    info!("Initializing storage at {:?}", config.db_path);
    let storage = Arc::new(Mutex::new(
        StorageManager::new(&config.db_path).context("Failed to initialize storage")?,
    ));

    // Initialize event engine (before RPC server so it can access events)
    info!("Initializing event-driven intelligence...");
    let event_engine = EventEngine::new(300, 30, 500); // 300ms debounce, 30s cooldown, 500 event history
    let event_tx = event_engine.sender();
    let event_engine_shared = Arc::new(event_engine.shared_state());

    // Start RPC server
    info!("Starting RPC server at {:?}", config.socket_path);
    let rpc_server = Arc::new(RpcServer::new(
        Arc::clone(&storage),
        Arc::clone(&event_engine_shared),
    ));
    let rpc_socket = config.socket_path.clone();

    tokio::spawn(async move {
        if let Err(e) = rpc_server.start(rpc_socket).await {
            error!("RPC server failed: {}", e);
        }
    });

    // Spawn watchdog heartbeat thread (10s interval)
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            info!("Watchdog heartbeat: daemon alive");
        }
    });

    // Wait for RPC server to bind
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Initialize policy engine
    let policy = match PolicyEngine::new() {
        Ok(engine) => Arc::new(Mutex::new(engine)),
        Err(e) => {
            warn!("Failed to load policy.toml: {}", e);
            warn!("Starting with default policy (alert-only mode)");
            // Create minimal default - will try to load again in future
            match PolicyEngine::new() {
                Ok(engine) => Arc::new(Mutex::new(engine)),
                Err(_) => {
                    error!("Cannot initialize policy engine, using fallback");
                    // For now, just try again - in production this would need a hardcoded fallback
                    return Err(e);
                }
            }
        }
    };

    // Initialize integrity watchdog
    let integrity = Arc::new(Mutex::new(IntegrityWatchdog::new()));

    // Create doctor handler
    let doctor_handler = Arc::new(DoctorHandlerImpl::new(
        Arc::clone(&integrity),
        Arc::clone(&policy),
    ));

    // Spawn event listeners
    info!("Spawning event listeners (packages, config, storage, devices, network)");
    let listener_handles = listeners::spawn_all(event_tx);
    info!("Spawned {} event listeners", listener_handles.len());

    // Start event engine (consumes the engine, history/queue remain shared via event_engine_shared)
    tokio::spawn(async move {
        info!("Event engine starting...");
        if let Err(e) = event_engine.run(doctor_handler).await {
            error!("Event engine failed: {}", e);
        }
    });

    // Start integrity watchdog (runs every 10 minutes)
    let watchdog_integrity = Arc::clone(&integrity);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(600)); // 10 minutes

        loop {
            interval.tick().await;
            info!("Running integrity watchdog sweep...");

            match watchdog_integrity.lock().await.sweep() {
                Ok(alerts) => {
                    if !alerts.is_empty() {
                        warn!("Integrity watchdog found {} alerts", alerts.len());
                    }
                }
                Err(e) => {
                    error!("Integrity watchdog failed: {}", e);
                }
            }
        }
    });

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
        std::fs::create_dir_all(parent).context("Failed to create state directory")?;
    }

    // /run/anna
    if let Some(parent) = config.socket_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create runtime directory")?;
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
    let mut collection_count = 0u32;

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
                    collection_count += 1;

                    // Compute persona scores every 10 collections (~5 minutes)
                    if collection_count % 10 == 0 {
                        if let Err(e) = update_persona_scores(&storage, &snapshot).await {
                            warn!("Failed to update persona scores: {}", e);
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
            backoff_secs = (config.poll_interval_secs * 2u64.pow(failure_count)).min(300);
            warn!(
                "Backing off to {}s after {} failures",
                backoff_secs, failure_count
            );
        }

        // Sleep with jitter
        let jitter = rand::random::<f64>() * config.poll_jitter_secs as f64;
        let sleep_duration = Duration::from_secs_f64(
            backoff_secs as f64 + jitter - (config.poll_jitter_secs as f64 / 2.0),
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

    storage
        .lock()
        .await
        .store_persona_scores(snapshot.ts, &scores_data)?;

    // Log top 3 scores
    let mut sorted_scores = scores.clone();
    sorted_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    if sorted_scores.len() >= 3 {
        info!(
            "Top personas: {} ({:.1}), {} ({:.1}), {} ({:.1})",
            sorted_scores[0].name,
            sorted_scores[0].score,
            sorted_scores[1].name,
            sorted_scores[1].score,
            sorted_scores[2].name,
            sorted_scores[2].score,
        );
    }

    Ok(())
}
