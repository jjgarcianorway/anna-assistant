use anyhow::{Result, Context};
use std::path::Path;
use tokio::net::UnixListener;
use tracing::{info, error, warn};
use std::os::unix::fs::PermissionsExt;

mod config;
mod rpc;
mod diagnostics;
mod telemetry;
mod polkit;
mod autonomy;
mod persistence;
mod policy;
mod events;
mod learning;

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
    info!("[BOOT] Policy/Event/Learning subsystems active");
    info!("[READY] anna-assistant operational");

    // Run RPC server
    rpc::serve(listener, config).await
        .context("RPC server error")?;

    Ok(())
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
    use nix::unistd::{Gid, Uid, chown};

    // Set permissions
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .context(format!("Failed to set permissions on {}", path))?;

    // Set ownership to root:anna (if anna group exists)
    if let Some(gid) = gid {
        chown(path, Some(Uid::from_raw(0)), Some(Gid::from_raw(gid)))
            .context(format!("Failed to set ownership on {}", path))?;
    }

    Ok(())
}

/// Configure socket permissions: 0660 root:anna
fn configure_socket_permissions() -> Result<()> {
    use nix::unistd::{Gid, Uid, chown};

    let anna_gid = get_anna_group_id();

    // Set permissions to 0660 (owner and group can read/write)
    std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))
        .context(format!("Failed to set socket permissions on {}", SOCKET_PATH))?;

    // Set ownership to root:anna
    if let Some(gid) = anna_gid {
        chown(SOCKET_PATH, Some(Uid::from_raw(0)), Some(Gid::from_raw(gid)))
            .context(format!("Failed to set socket ownership on {}", SOCKET_PATH))?;
        info!("[BOOT] Socket permissions: 0660 root:{}", ANNA_GROUP);
    } else {
        warn!("Socket ownership: root:root (anna group not found)");
        // Fallback to 0666 if anna group doesn't exist
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))
            .context("Failed to set fallback socket permissions")?;
    }

    Ok(())
}
