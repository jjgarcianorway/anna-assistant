//! Anna daemon - manages system state, Ollama, and models.
//! v0.0.73: Uses version module for consistent version reporting.

use anna_shared::version::{VERSION, GIT_SHA};
use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use annad::server::Server;

/// Anna daemon - manages system state, Ollama, and models.
#[derive(Parser)]
#[command(name = "annad")]
#[command(version = anna_shared::VERSION)]
#[command(about = "Anna daemon - manages system state, Ollama, and models")]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse args (enables --version)
    let _args = Args::parse();

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

    // Create and run server
    let server = Server::new().await?;
    server.run().await?;

    Ok(())
}
