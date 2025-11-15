//! Hot reload for peer and TLS configuration (Phase 1.15)
//!
//! Handles SIGHUP signal and atomic configuration reloads without daemon restart.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use super::metrics::ConsensusMetrics;
use super::peers::PeerList;

/// Reloadable configuration state
#[derive(Clone)]
pub struct ReloadableConfig {
    /// Path to peers.yml configuration file
    config_path: PathBuf,
    /// Current peer list
    peer_list: Arc<RwLock<PeerList>>,
    /// Metrics for tracking reloads
    metrics: ConsensusMetrics,
}

impl ReloadableConfig {
    /// Create new reloadable configuration
    pub fn new(config_path: PathBuf, peer_list: PeerList, metrics: ConsensusMetrics) -> Self {
        Self {
            config_path,
            peer_list: Arc::new(RwLock::new(peer_list)),
            metrics,
        }
    }

    /// Get current peer list (read-only)
    pub async fn get_peer_list(&self) -> PeerList {
        self.peer_list.read().await.clone()
    }

    /// Reload configuration atomically
    ///
    /// This method:
    /// 1. Loads new configuration from disk
    /// 2. Validates TLS certificates (if enabled)
    /// 3. Atomically swaps configuration
    /// 4. Records metrics
    ///
    /// Returns Ok(true) if reload succeeded, Ok(false) if config unchanged, Err on failure.
    pub async fn reload(&self) -> Result<bool> {
        info!("Hot reload triggered: loading configuration from {}", self.config_path.display());

        // Load new peer list from disk
        let new_peer_list = PeerList::load_from_file(&self.config_path)
            .await
            .with_context(|| format!("Failed to load peers from {}", self.config_path.display()))?;

        // Compare with current config
        let current = self.peer_list.read().await;

        // Check if configuration actually changed
        if Self::configs_equal(&*current, &new_peer_list) {
            info!("Configuration unchanged, skipping reload");
            self.metrics.record_peer_reload("unchanged");
            return Ok(false);
        }

        // Validate new configuration
        if let Some(ref tls_config) = new_peer_list.tls {
            info!("Validating new TLS configuration...");
            tls_config.validate().await
                .with_context(|| "New TLS configuration validation failed")?;

            // Pre-load TLS configs to ensure they're valid
            let _ = tls_config.load_server_config().await
                .with_context(|| "Failed to load new server TLS config")?;
            let _ = tls_config.load_client_config().await
                .with_context(|| "Failed to load new client TLS config")?;

            info!("✓ New TLS configuration validated successfully");
        }

        // Atomic swap
        drop(current);
        let mut current = self.peer_list.write().await;
        *current = new_peer_list.clone();
        drop(current);

        info!("✓ Configuration reloaded successfully");
        info!("  Peers: {} nodes", new_peer_list.peers.len());
        info!("  TLS: {}", if new_peer_list.tls_enabled() { "enabled" } else { "disabled" });

        self.metrics.record_peer_reload("success");
        Ok(true)
    }

    /// Check if two configurations are equal
    fn configs_equal(a: &PeerList, b: &PeerList) -> bool {
        // Compare peer count
        if a.peers.len() != b.peers.len() {
            return false;
        }

        // Compare TLS mode
        if a.allow_insecure_peers != b.allow_insecure_peers {
            return false;
        }

        // Compare peer list
        for (peer_a, peer_b) in a.peers.iter().zip(b.peers.iter()) {
            if peer_a.node_id != peer_b.node_id
                || peer_a.address != peer_b.address
                || peer_a.port != peer_b.port
            {
                return false;
            }
        }

        // Compare TLS config paths (if enabled)
        match (&a.tls, &b.tls) {
            (Some(tls_a), Some(tls_b)) => {
                if tls_a.ca_cert != tls_b.ca_cert
                    || tls_a.server_cert != tls_b.server_cert
                    || tls_a.server_key != tls_b.server_key
                    || tls_a.client_cert != tls_b.client_cert
                    || tls_a.client_key != tls_b.client_key
                {
                    return false;
                }
            }
            (None, None) => {}
            _ => return false,
        }

        true
    }
}

/// SIGHUP signal handler
///
/// Listens for SIGHUP and triggers configuration reload.
/// Runs in a dedicated task until cancelled.
pub async fn sighup_handler(config: ReloadableConfig) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(sig) => sig,
            Err(e) => {
                error!("Failed to register SIGHUP handler: {}", e);
                return;
            }
        };

        info!("SIGHUP handler registered, listening for reload signals");

        loop {
            sighup.recv().await;
            info!("SIGHUP received, initiating hot reload...");

            match config.reload().await {
                Ok(true) => {
                    info!("✓ Hot reload completed successfully");
                }
                Ok(false) => {
                    info!("Configuration unchanged, no reload needed");
                }
                Err(e) => {
                    error!("Hot reload failed: {}", e);
                    config.metrics.record_peer_reload("failure");
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        warn!("SIGHUP handler not available on non-Unix platforms");
        // On non-Unix platforms, this task just waits forever
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_configs_equal() {
        // Create two identical peer lists
        let peer1 = super::super::peers::PeerConfig {
            node_id: "node1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8001,
        };

        let peer_list_a = PeerList {
            allow_insecure_peers: false,
            tls: None,
            peers: vec![peer1.clone()],
        };

        let peer_list_b = PeerList {
            allow_insecure_peers: false,
            tls: None,
            peers: vec![peer1.clone()],
        };

        assert!(ReloadableConfig::configs_equal(&peer_list_a, &peer_list_b));

        // Change peer count
        let peer_list_c = PeerList {
            allow_insecure_peers: false,
            tls: None,
            peers: vec![],
        };

        assert!(!ReloadableConfig::configs_equal(&peer_list_a, &peer_list_c));
    }
}
