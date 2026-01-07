//! Anna daemon - simplified version.

use anna_shared::VERSION;
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use annad::server::Server;
use annad::state::SharedState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting annad v{}", VERSION);

    // Create shared state
    let state = SharedState::new();

    // Create and run server
    let server = Server::new(state);
    server.run().await?;

    Ok(())
}
