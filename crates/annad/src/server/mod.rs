//! Unix socket server for annad.
//! v0.0.159: Update check loop extracted to update_loop.rs.
//! v0.0.269: Intelligent model auto-selection with benchmarking.
//! v0.0.281: Telemetry collector integration.
//! v0.0.310: Non-blocking model pulls - daemon ready immediately.
//! v0.0.825: Model setup extracted to model_setup.rs.

mod background;
mod initialization;
mod model_setup;
mod socket;
mod types;

use anyhow::Result;
use tracing::error;

use crate::state::SharedState;

pub use types::Server;

impl Server {
    pub async fn new(state: SharedState) -> Result<Self> {
        Ok(Self { state })
    }

    pub async fn run(&self) -> Result<()> {
        // v0.0.298: Create socket EARLY so annactl can connect while initializing
        // This fixes the long wait for anna.sock after install
        self.setup_directories().await?;

        // Start socket server in background BEFORE initialization
        // The server will accept connections but return "initializing" status
        let state_for_socket = self.state.clone();
        let socket_handle = tokio::spawn(async move {
            if let Err(e) = Self::run_socket_server_impl(state_for_socket).await {
                error!("Socket server error: {}", e);
            }
        });

        // Now initialize daemon (this can be slow - model selection, pulling)
        // But clients can already connect and see "initializing" status
        self.initialize().await?;

        // Spawn all background tasks
        background::spawn_background_tasks(self.state.clone());

        // Wait for socket server (will run forever or until error)
        let _ = socket_handle.await;
        Ok(())
    }
}

// v0.0.159: update_check_loop moved to update_loop.rs
