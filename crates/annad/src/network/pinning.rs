//! Certificate pinning for TLS connections (Phase 1.16)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// Certificate pinning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinningConfig {
    /// Enable certificate pinning
    pub enable_pinning: bool,
    /// Pin client certificates (in addition to server certs)
    pub pin_client_certs: bool,
    /// Map of node_id -> SHA256 fingerprint (hex)
    pub pins: HashMap<String, String>,
}

impl PinningConfig {
    /// Load pinning configuration from file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            info!("Pinning config not found at {}, using defaults", path.display());
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read pinning config: {}", path.display()))?;

        let config: PinningConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse pinning config: {}", path.display()))?;

        info!(
            "Loaded certificate pinning config: {} pins, enabled={}",
            config.pins.len(),
            config.enable_pinning
        );

        Ok(config)
    }

    /// Validate certificate fingerprint against pinned value
    pub fn validate_fingerprint(&self, node_id: &str, cert_der: &[u8]) -> bool {
        if !self.enable_pinning {
            return true; // Pinning disabled
        }

        let fingerprint = Self::compute_fingerprint(cert_der);

        match self.pins.get(node_id) {
            Some(expected) => {
                if fingerprint == *expected {
                    true
                } else {
                    warn!(
                        "Certificate fingerprint mismatch for {}: expected {}, got {}",
                        node_id,
                        expected,
                        fingerprint
                    );
                    false
                }
            }
            None => {
                warn!("No pinned certificate for node {}", node_id);
                false
            }
        }
    }

    /// Compute SHA256 fingerprint of certificate (DER format)
    pub fn compute_fingerprint(cert_der: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_der);
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    /// Add or update pin for a node
    pub fn add_pin(&mut self, node_id: String, fingerprint: String) {
        self.pins.insert(node_id, fingerprint);
    }

    /// Remove pin for a node
    pub fn remove_pin(&mut self, node_id: &str) {
        self.pins.remove(node_id);
    }

    /// Save configuration to file
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize pinning config")?;

        fs::write(path, content).await
            .with_context(|| format!("Failed to write pinning config: {}", path.display()))?;

        Ok(())
    }
}

impl Default for PinningConfig {
    fn default() -> Self {
        Self {
            enable_pinning: false,
            pin_client_certs: false,
            pins: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fingerprint() {
        let cert_data = b"test certificate";
        let fingerprint = PinningConfig::compute_fingerprint(cert_data);

        // SHA256 should produce 64-char hex string
        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_validate_fingerprint() {
        let mut config = PinningConfig {
            enable_pinning: true,
            pin_client_certs: false,
            pins: HashMap::new(),
        };

        let cert_data = b"test certificate";
        let fingerprint = PinningConfig::compute_fingerprint(cert_data);

        // Add pin
        config.add_pin("node1".to_string(), fingerprint.clone());

        // Validate with correct fingerprint
        assert!(config.validate_fingerprint("node1", cert_data));

        // Validate with wrong certificate
        assert!(!config.validate_fingerprint("node1", b"wrong cert"));

        // Validate unknown node
        assert!(!config.validate_fingerprint("node2", cert_data));
    }

    #[test]
    fn test_pinning_disabled() {
        let config = PinningConfig {
            enable_pinning: false,
            pin_client_certs: false,
            pins: HashMap::new(),
        };

        // Should always validate when pinning disabled
        assert!(config.validate_fingerprint("node1", b"any cert"));
    }
}
