use anyhow::{Result, Context};
use std::path::Path;
use tokio::net::UnixListener;
use tracing::{info, error, warn};
use std::os::unix::fs::PermissionsExt;

mod config;
mod rpc;
mod diagnostics;
mod telemetry;
mod telemetry_collector;
mod polkit;
mod autonomy;
mod persistence;
mod policy;
mod events;
mod learning;
mod state;

const SOCKET_PATH: &str = "/run/anna/annad.sock";
const SOCKET_DIR: &str = "/run/anna";
const CONFIG_DIR: &str = "/etc/anna";
const STATE_DIR: &str = "/var/lib/anna";
const ANNA_GROUP: &str = "anna";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("[BOOT] Anna Assistant Daemon v{} starting...", env!("CARGO_PKG_VERSION"));

    // Verify we're running as the anna user
    let current_user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    if current_user != "anna" && current_user != "root" {
        warn!("[BOOT] Running as user '{}' (expected 'anna')", current_user);
    }

    // Verify required directories exist (created by systemd or installer)
    if let Err(e) = verify_directories() {
        error!("[FATAL] Required directories missing: {}", e);
        error!("[FATAL] Run the installer or ensure systemd RuntimeDirectory/StateDirectory are configured");
        std::process::exit(1);
    }

    // Initialize telemetry
    telemetry::init()
        .context("Failed to initialize telemetry")?;
    telemetry::log_event(telemetry::Event::DaemonStarted)
        .context("Failed to log daemon start event")?;

    // Initialize persistence
    persistence::init()
        .context("Failed to initialize persistence")?;
    info!("[BOOT] Persistence ready");

    // Load configuration
    let config = config::load_config()
        .context("Failed to load configuration")?;
    info!("[BOOT] Config loaded");

    // Clean up old socket if it exists
    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)
            .context("Failed to remove old socket")?;
    }

    // Bind Unix socket
    let listener = UnixListener::bind(SOCKET_PATH)
        .context(format!("Failed to bind socket at {}", SOCKET_PATH))?;

    // Set socket permissions (0660 so anna group can access)
    std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))
        .context("Failed to set socket permissions")?;

    info!("[BOOT] RPC online ({}, permissions: 0660)", SOCKET_PATH);

    // Initialize global daemon state
    let state = match state::DaemonState::new() {
        Ok(s) => std::sync::Arc::new(s),
        Err(e) => {
            error!("[FATAL] Failed to initialize daemon state: {}", e);
            std::process::exit(78);
        }
    };

    // Emit bootstrap events
    if let Err(e) = state.emit_bootstrap_events() {
        warn!("Failed to emit bootstrap events: {}", e);
    }

    info!("[BOOT] Policy/Event/Learning subsystems active");

    // Initialize and start telemetry collector (runs in background)
    let telemetry_db_path = format!("{}/telemetry.db", STATE_DIR);
    match telemetry_collector::TelemetryCollector::new(&telemetry_db_path) {
        Ok(collector) => {
            let collector_arc = std::sync::Arc::new(collector);
            collector_arc.clone().start_collection_loop();
            info!("[BOOT] Telemetry collector started (60s interval)");
        }
        Err(e) => {
            warn!("Failed to initialize telemetry collector: {}", e);
        }
    }

    // Start CPU watchdog (monitors idle CPU every 5 minutes)
    start_cpu_watchdog();

    info!("[READY] anna-assistant operational");

    // Run RPC server with state
    rpc::serve(listener, config, state).await
        .context("RPC server error")?;

    Ok(())
}

/// CPU watchdog: monitors daemon's own CPU usage when idle
/// Logs warning if idle CPU > 5% for 3 consecutive samples
fn start_cpu_watchdog() {
    tokio::spawn(async {
        use sysinfo::{System, Pid};
        use tokio::time::{interval, Duration};

        let mut interval = interval(Duration::from_secs(300)); // 5 minutes
        let pid = Pid::from_u32(std::process::id());
        let mut high_cpu_count = 0;

        loop {
            interval.tick().await;

            let mut sys = System::new();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                let cpu_usage = process.cpu_usage();

                if cpu_usage > 5.0 {
                    high_cpu_count += 1;
                    warn!("[WATCHDOG] Idle CPU usage: {:.1}% (sample {}/3)", cpu_usage, high_cpu_count);

                    if high_cpu_count >= 3 {
                        error!(
                            "[WATCHDOG] High idle CPU detected! Daemon using {:.1}% CPU. \
                            Suspected culprits: telemetry collector, policy engine, or event loop",
                            cpu_usage
                        );
                        // Reset counter after logging alert
                        high_cpu_count = 0;
                    }
                } else {
                    // Reset counter if CPU drops back to normal
                    if high_cpu_count > 0 {
                        info!("[WATCHDOG] CPU usage normalized: {:.1}%", cpu_usage);
                        high_cpu_count = 0;
                    }
                }
            }
        }
    });
}

/// Verify all required directories exist (created by systemd or installer)
fn verify_directories() -> Result<()> {
    let required = vec![
        (SOCKET_DIR, "socket directory"),
        (STATE_DIR, "state directory"),
        (CONFIG_DIR, "config directory"),
    ];

    for (path, name) in required {
        if !Path::new(path).exists() {
            anyhow::bail!("{} missing: {}", name, path);
        }
    }

    info!("[BOOT] All required directories present");
    Ok(())
}

