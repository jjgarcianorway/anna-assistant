//! Anna daemon - manages system state, Ollama, and models.
//! v0.0.73: Uses version module for consistent version reporting.
//! v0.0.825: Added watchdog for automatic recovery.

use anna_shared::version::{GIT_SHA, VERSION};
use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use annad::learning_loop;
use annad::server::Server;
use annad::state::load_initial_state;
use annad::watchdog::{start_watchdog, WatchdogConfig};

/// Anna daemon - manages system state, Ollama, and models.
#[derive(Parser)]
#[command(name = "annad")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Anna daemon - manages system state, Ollama, and models")]
struct Args {
    /// Disable the watchdog (not recommended)
    #[arg(long, default_value = "false")]
    no_watchdog: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse args (enables --version)
    let args = Args::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let version_str = if GIT_SHA != "unknown" {
        format!("{} ({})", VERSION, GIT_SHA)
    } else {
        VERSION.to_string()
    };
    info!("Starting annad v{}", version_str);

    // Create and load initial state
    let state = load_initial_state().await;

    // Create and run server, passing the state
    let server = Server::new(state.clone()).await?;
    let server_run_handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server runtime error: {}", e);
        }
    });

    // Spawn the continuous learning loop
    let learning_state = state.clone();
    tokio::spawn(async move {
        learning_loop::start_learning_loop(learning_state).await;
    });

    // v0.0.825: Spawn watchdog for health monitoring and auto-recovery
    if !args.no_watchdog {
        let watchdog_state = state.clone();
        tokio::spawn(async move {
            start_watchdog(watchdog_state, WatchdogConfig::default()).await;
        });
    } else {
        info!("Watchdog disabled via --no-watchdog flag");
    }

    server_run_handle.await?; // Wait for the server to finish (if it ever does)

    Ok(())
}
