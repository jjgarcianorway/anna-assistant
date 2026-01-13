//! Unix socket server for handling client requests.

mod alerts;
mod handlers;
mod init;
mod streaming;

use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio::net::UnixListener;
use tracing::{error, info, warn};

use anna_shared::socket_path;

use crate::core_loop::{monitoring_loop, profile_refresh_loop};
use crate::state::SharedState;
use crate::update_loop::update_check_loop;

use handlers::handle_connection;
use init::initialize;

/// The daemon server
pub struct Server {
    state: SharedState,
}

impl Server {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<()> {
        // Setup socket
        let socket_path = socket_path();
        self.setup_socket(&socket_path).await?;

        // Start initialization in background
        let init_state = self.state.clone();
        tokio::spawn(async move {
            if let Err(e) = initialize(init_state).await {
                error!("Initialization failed: {}", e);
            }
        });

        // Start update check loop
        let update_state = self.state.clone();
        tokio::spawn(async move {
            update_check_loop(update_state).await;
        });

        // Start profile refresh loop (checks every 30 minutes)
        tokio::spawn(async move {
            profile_refresh_loop().await;
        });

        // Start proactive monitoring loop (checks every 5 minutes)
        tokio::spawn(async move {
            monitoring_loop().await;
        });

        // Run socket server
        self.run_socket_server(&socket_path).await
    }

    async fn setup_socket(&self, socket_path: &str) -> Result<()> {
        let path = Path::new(socket_path);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }

        // Remove old socket if it exists
        if path.exists() {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    async fn run_socket_server(&self, socket_path: &str) -> Result<()> {
        let listener = UnixListener::bind(socket_path)?;
        info!("Listening on {}", socket_path);

        // v0.3.32: Socket permissions - owner + anna group only
        // 0660 = rw-rw---- (root:anna)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o660);
            std::fs::set_permissions(socket_path, perms)?;
            info!("Socket permissions set to 0660");
        }

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state).await {
                            warn!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}
