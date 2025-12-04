//! Anna daemon - manages system state, Ollama, and models.

mod hardware;
mod health;
mod ollama;
mod permissions;
mod probes;
mod rpc_handler;
mod scoring;
mod server;
mod service_desk;
mod state;
mod translator;
mod update;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting annad v{}", anna_shared::VERSION);

    // Create and run server
    let server = Server::new().await?;
    server.run().await?;

    Ok(())
}
