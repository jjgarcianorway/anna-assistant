// Phase 2: Certificate Pinning Configuration
//
// Loads and validates /etc/anna/pinned_certs.json
//
// Format:
// {
//   "enforce": true,
//   "peers": {
//     "node1.example.com": "sha256:HEXDIGEST",
//     "node2.example.com": "sha256:HEXDIGEST"
//   }
// }

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use super::pinning_verifier::PinningConfig;

/// JSON structure for pinned_certs.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinningConfigFile {
    /// Whether to enforce pinning (fail-closed on mismatch)
    #[serde(default = "default_enforce")]
    pub enforce: bool,

    /// Map of peer hostnames to SHA256 fingerprints
    /// Format: "hostname" -> "sha256:HEXDIGEST"
    pub peers: HashMap<String, String>,
}

fn default_enforce() -> bool {
    true // Fail-closed by default for security
}

impl PinningConfigFile {
    /// Load pinning configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            warn!(
                path = %path.display(),
                "Certificate pinning config not found, pinning disabled"
            );
            return Ok(Self {
                enforce: false,
                peers: HashMap::new(),
            });
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read pinning config: {}", path.display()))?;

        let config: Self = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse pinning config: {}", path.display()))?;

        // Validate fingerprint format
        for (hostname, fingerprint) in &config.peers {
            if !fingerprint.starts_with("sha256:") {
                anyhow::bail!(
                    "Invalid fingerprint format for {}: must start with 'sha256:'",
                    hostname
                );
            }

            // Check hex digest length (64 hex chars = 32 bytes)
            let hex_part = &fingerprint[7..]; // Skip "sha256:"
            if hex_part.len() != 64 {
                anyhow::bail!(
                    "Invalid fingerprint length for {}: expected 64 hex chars, got {}",
                    hostname,
                    hex_part.len()
                );
            }

            // Validate hex characters
            if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                anyhow::bail!(
                    "Invalid fingerprint for {}: contains non-hex characters",
                    hostname
                );
            }
        }

        info!(
            peers = config.peers.len(),
            enforce = config.enforce,
            "Loaded certificate pinning configuration"
        );

        Ok(config)
    }

    /// Convert to PinningConfig used by verifier
    pub fn into_pinning_config(self) -> PinningConfig {
        let mut config = PinningConfig::new(self.enforce);
        for (hostname, fingerprint) in self.peers {
            config.add_pin(hostname, fingerprint);
        }
        config
    }

    /// Generate example configuration file content
    pub fn example() -> String {
        let example = Self {
            enforce: true,
            peers: HashMap::from([
                (
                    "node1.example.com".to_string(),
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .to_string(),
                ),
                (
                    "node2.example.com".to_string(),
                    "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
                        .to_string(),
                ),
            ]),
        };

        serde_json::to_string_pretty(&example).unwrap()
    }
}

/// Default path for pinning configuration
pub const DEFAULT_PINNING_CONFIG_PATH: &str = "/etc/anna/pinned_certs.json";

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let config_json = r#"{
            "enforce": true,
            "peers": {
                "node1.example.com": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "node2.example.com": "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = PinningConfigFile::load_from_file(temp_file.path()).unwrap();

        assert!(config.enforce);
        assert_eq!(config.peers.len(), 2);
        assert!(config.peers.contains_key("node1.example.com"));
        assert!(config.peers.contains_key("node2.example.com"));
    }

    #[test]
    fn test_load_missing_file() {
        let config = PinningConfigFile::load_from_file("/nonexistent/path").unwrap();

        // Should return disabled config
        assert!(!config.enforce);
        assert_eq!(config.peers.len(), 0);
    }

    #[test]
    fn test_invalid_fingerprint_format() {
        let config_json = r#"{
            "enforce": true,
            "peers": {
                "node1.example.com": "invalid_format"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = PinningConfigFile::load_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with 'sha256:'"));
    }

    #[test]
    fn test_invalid_fingerprint_length() {
        let config_json = r#"{
            "enforce": true,
            "peers": {
                "node1.example.com": "sha256:tooshort"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = PinningConfigFile::load_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expected 64 hex chars"));
    }

    #[test]
    fn test_invalid_hex_characters() {
        let config_json = r#"{
            "enforce": true,
            "peers": {
                "node1.example.com": "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = PinningConfigFile::load_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-hex characters"));
    }

    #[test]
    fn test_example_generation() {
        let example = PinningConfigFile::example();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&example).unwrap();
        assert!(parsed["enforce"].as_bool().unwrap());
        assert!(parsed["peers"].is_object());
    }

    #[test]
    fn test_into_pinning_config() {
        let config_file = PinningConfigFile {
            enforce: true,
            peers: HashMap::from([(
                "node1.example.com".to_string(),
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
            )]),
        };

        let pinning_config = config_file.into_pinning_config();

        assert!(pinning_config.enforce);
        assert_eq!(
            pinning_config.get_pin("node1.example.com"),
            Some(
                &"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string()
            )
        );
    }
}
