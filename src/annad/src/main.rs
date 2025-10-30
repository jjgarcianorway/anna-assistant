use anyhow::Result;
use std::path::Path;
use tokio::net::UnixListener;
use tracing::{info, error};

mod config;
mod rpc;
mod diagnostics;
mod telemetry;
mod polkit;

const SOCKET_PATH: &str = "/run/anna/annad.sock";
const SOCKET_DIR: &str = "/run/anna";
const CONFIG_DIR: &str = "/etc/anna";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Anna Assistant Daemon v{} starting...", env!("CARGO_PKG_VERSION"));

    // Check if running as root
    if !nix::unistd::Uid::effective().is_root() {
        error!("annad must run as root");
        std::process::exit(1);
    }

    // Ensure directories exist
    if !Path::new(CONFIG_DIR).exists() {
        std::fs::create_dir_all(CONFIG_DIR)?;
        info!("Created config directory: {}", CONFIG_DIR);
    }

    if !Path::new(SOCKET_DIR).exists() {
        std::fs::create_dir_all(SOCKET_DIR)?;
        info!("Created socket directory: {}", SOCKET_DIR);
    }

    // Initialize telemetry
    telemetry::init()?;
    telemetry::log_event(telemetry::Event::DaemonStarted)?;

    // Load configuration
    let config = config::load_config()?;
    info!("Configuration loaded successfully");

    // Clean up old socket if it exists
    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }

    // Bind Unix socket
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("Listening on {}", SOCKET_PATH);

    // Set socket permissions (readable/writable by all users)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))?;
    }

    // Run RPC server
    rpc::serve(listener, config).await?;

    Ok(())
}
