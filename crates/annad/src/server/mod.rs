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
use crate::telegram;
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
        info!("Server starting, socket path: {}", socket_path);
        self.setup_socket(&socket_path).await?;
        info!("Socket setup complete");

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

        // Start Telegram bot if configured
        let telegram_state = self.state.clone();
        tokio::spawn(async move {
            if let Err(e) = telegram::start_telegram_bot(telegram_state).await {
                error!("Telegram bot failed: {}", e);
            }
        });

        // Run socket server
        self.run_socket_server(&socket_path).await
    }

    async fn setup_socket(&self, socket_path: &str) -> Result<()> {
        let path = Path::new(socket_path);

        // Create parent directory if needed (don't silently fail!)
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                info!("Creating socket directory: {:?}", parent);
                fs::create_dir_all(parent).await.map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to create socket directory {:?}: {}. \
                         Check that the daemon has permission to write to /run/",
                        parent, e
                    )
                })?;

                // Set permissions: 750 root:anna
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(0o750);
                    std::fs::set_permissions(parent, perms).ok();
                }
            }
        }

        // Remove old socket if it exists
        if path.exists() {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    async fn run_socket_server(&self, socket_path: &str) -> Result<()> {
        info!("Binding socket at {}", socket_path);
        let listener = match UnixListener::bind(socket_path) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind socket at {}: {}", socket_path, e);
                return Err(anyhow::anyhow!("Socket bind failed: {}", e));
            }
        };
        info!("Listening on {}", socket_path);

        // v0.3.32: Socket permissions - owner + anna group only
        // 0660 = rw-rw---- (root:anna)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o660);
            std::fs::set_permissions(socket_path, perms)?;
            info!("Socket permissions set to 0660");

            // v0.3.42: Set socket group ownership to 'anna'
            // Without this, socket is root:root and users in anna group can't connect
            match std::process::Command::new("chown")
                .args([":anna", socket_path])
                .output()
            {
                Ok(output) if output.status.success() => {
                    info!("Socket group set to 'anna'");
                }
                Ok(output) => {
                    warn!(
                        "Failed to set socket group: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => {
                    warn!("Failed to run chown: {}", e);
                }
            }
        }

        // v0.3.38: Notify systemd AFTER socket is ready (not before!)
        // This ensures clients can connect immediately after ready notification
        let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);
        info!("Daemon ready, socket accepting connections");

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
