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

    // Check if running as root
    if !nix::unistd::Uid::effective().is_root() {
        error!("[FATAL] annad must run as root");
        std::process::exit(1);
    }

    // Initialize directories with proper permissions
    if let Err(e) = ensure_directories() {
        error!("[FATAL] Failed to initialize directories: {}", e);
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

    // Set socket permissions and ownership
    if let Err(e) = configure_socket_permissions() {
        error!("[FATAL] Failed to configure socket permissions: {}", e);
        std::process::exit(1);
    }

    info!("[BOOT] RPC online ({})", SOCKET_PATH);

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

/// Ensure all required directories exist with correct permissions
fn ensure_directories() -> Result<()> {
    // Get anna group ID
    let anna_gid = get_anna_group_id();

    // Config directory: 0750 root:anna
    if !Path::new(CONFIG_DIR).exists() {
        std::fs::create_dir_all(CONFIG_DIR)
            .context(format!("Failed to create {}", CONFIG_DIR))?;
    }
    set_directory_permissions(CONFIG_DIR, 0o750, anna_gid)?;

    // Socket directory: 0770 root:anna
    if !Path::new(SOCKET_DIR).exists() {
        std::fs::create_dir_all(SOCKET_DIR)
            .context(format!("Failed to create {}", SOCKET_DIR))?;
    }
    set_directory_permissions(SOCKET_DIR, 0o770, anna_gid)?;

    // State directory: 0750 root:anna
    if !Path::new(STATE_DIR).exists() {
        std::fs::create_dir_all(STATE_DIR)
            .context(format!("Failed to create {}", STATE_DIR))?;
    }
    set_directory_permissions(STATE_DIR, 0o750, anna_gid)?;

    info!("[BOOT] Directories initialized");
    Ok(())
}

/// Get the anna group ID, or None if group doesn't exist
fn get_anna_group_id() -> Option<u32> {
    use nix::unistd::Group;
    match Group::from_name(ANNA_GROUP) {
        Ok(Some(group)) => Some(group.gid.as_raw()),
        Ok(None) => {
            warn!("Group '{}' not found, using root group", ANNA_GROUP);
            None
        }
        Err(e) => {
            warn!("Failed to lookup group '{}': {}", ANNA_GROUP, e);
            None
        }
    }
}

/// Set directory permissions and group ownership
fn set_directory_permissions(path: &str, mode: u32, gid: Option<u32>) -> Result<()> {
    // Set permissions
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .context(format!("Failed to set permissions on {}", path))?;

    // Set ownership to root:anna (if anna group exists)
    if gid.is_some() {
        let result = std::process::Command::new("chown")
            .arg("root:anna")
            .arg(path)
            .status();

        if let Err(e) = result {
            warn!("Failed to set ownership on {}: {}", path, e);
        }
    }

    Ok(())
}

/// Configure socket permissions: 0660 root:anna
fn configure_socket_permissions() -> Result<()> {
    let anna_gid = get_anna_group_id();

    // Set permissions to 0660 (owner and group can read/write)
    std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))
        .context(format!("Failed to set socket permissions on {}", SOCKET_PATH))?;

    // Set ownership to root:anna
    if anna_gid.is_some() {
        let result = std::process::Command::new("chown")
            .arg("root:anna")
            .arg(SOCKET_PATH)
            .status();

        if result.is_ok() {
            info!("[BOOT] Socket permissions: 0660 root:{}", ANNA_GROUP);
        } else {
            warn!("Failed to set socket ownership");
        }
    } else {
        warn!("Socket ownership: root:root (anna group not found)");
        // Fallback to 0666 if anna group doesn't exist
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))
            .context("Failed to set fallback socket permissions")?;
    }

    Ok(())
}
