//! State Schema v2 - Consensus-aware state (Phase 1.10)
//!
//! Extends v1 state with consensus tracking and network identity.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// State schema version 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateV2 {
    pub schema_version: u32,
    pub node_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Legacy v1 fields
    pub system_state: String,
    pub last_boot_time: Option<DateTime<Utc>>,
    pub configuration_hash: Option<String>,

    // New v2 fields
    pub consensus: ConsensusState,
    pub network: NetworkState,
}

/// Consensus state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub validator_count: usize,
    pub rounds_completed: u64,
    pub last_round_id: Option<String>,
    pub last_round_at: Option<DateTime<Utc>>,
    pub byzantine_nodes: Vec<ByzantineNodeRecord>,
}

/// Byzantine node record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByzantineNodeRecord {
    pub node_id: String,
    pub detected_at: DateTime<Utc>,
    pub reason: String,
}

/// Network state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub peer_count: usize,
    pub last_peer_reload: Option<DateTime<Utc>>,
    pub tls_enabled: bool,
}

impl StateV2 {
    /// Create new v2 state
    pub fn new(node_id: String) -> Self {
        let now = Utc::now();
        Self {
            schema_version: 2,
            node_id,
            created_at: now,
            updated_at: now,
            system_state: "configured".to_string(),
            last_boot_time: None,
            configuration_hash: None,
            consensus: ConsensusState {
                validator_count: 0,
                rounds_completed: 0,
                last_round_id: None,
                last_round_at: None,
                byzantine_nodes: Vec::new(),
            },
            network: NetworkState {
                peer_count: 0,
                last_peer_reload: None,
                tls_enabled: false,
            },
        }
    }

    /// Load from file
    pub async fn load(path: &Path) -> Result<Self> {
        info!("Loading state v2 from: {}", path.display());

        let content = fs::read_to_string(path).await
            .map_err(|e| anyhow!("Failed to read state file: {}", e))?;

        let state: StateV2 = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse state v2: {}", e))?;

        if state.schema_version != 2 {
            return Err(anyhow!(
                "Invalid schema version: expected 2, got {}",
                state.schema_version
            ));
        }

        info!("Loaded state v2 for node: {}", state.node_id);
        Ok(state)
    }

    /// Save to file (atomic write)
    pub async fn save(&self, path: &Path) -> Result<()> {
        info!("Saving state v2 to: {}", path.display());

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Serialize
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize state v2: {}", e))?;

        // Atomic write: temp file + rename
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &content).await
            .map_err(|e| anyhow!("Failed to write temp state: {}", e))?;

        fs::rename(&temp_path, path).await
            .map_err(|e| anyhow!("Failed to rename state file: {}", e))?;

        info!("State v2 saved successfully");
        Ok(())
    }

    /// Update consensus state
    pub fn update_consensus(
        &mut self,
        round_id: String,
        validator_count: usize,
        byzantine_nodes: Vec<ByzantineNodeRecord>,
    ) {
        self.consensus.rounds_completed += 1;
        self.consensus.last_round_id = Some(round_id);
        self.consensus.last_round_at = Some(Utc::now());
        self.consensus.validator_count = validator_count;
        self.consensus.byzantine_nodes = byzantine_nodes;
        self.updated_at = Utc::now();
    }

    /// Update network state
    pub fn update_network(&mut self, peer_count: usize, tls_enabled: bool) {
        self.network.peer_count = peer_count;
        self.network.tls_enabled = tls_enabled;
        self.network.last_peer_reload = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// Legacy v1 state (minimal representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateV1 {
    pub schema_version: Option<u32>,
    pub system_state: String,
    pub last_boot_time: Option<DateTime<Utc>>,
    pub configuration_hash: Option<String>,
}

impl StateV1 {
    /// Load from file
    pub async fn load(path: &Path) -> Result<Self> {
        info!("Loading state v1 from: {}", path.display());

        let content = fs::read_to_string(path).await
            .map_err(|e| anyhow!("Failed to read state file: {}", e))?;

        let state: StateV1 = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse state v1: {}", e))?;

        // v1 may not have schema_version field
        if let Some(version) = state.schema_version {
            if version != 1 {
                warn!("Unexpected schema version in v1 state: {}", version);
            }
        }

        info!("Loaded state v1");
        Ok(state)
    }

    /// Convert to v2
    pub fn to_v2(self, node_id: String) -> StateV2 {
        info!("Converting state v1 to v2");

        let now = Utc::now();
        StateV2 {
            schema_version: 2,
            node_id,
            created_at: now,
            updated_at: now,
            system_state: self.system_state,
            last_boot_time: self.last_boot_time,
            configuration_hash: self.configuration_hash,
            consensus: ConsensusState {
                validator_count: 0,
                rounds_completed: 0,
                last_round_id: None,
                last_round_at: None,
                byzantine_nodes: Vec::new(),
            },
            network: NetworkState {
                peer_count: 0,
                last_peer_reload: None,
                tls_enabled: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_v2_creation() {
        let state = StateV2::new("node_test".to_string());
        assert_eq!(state.schema_version, 2);
        assert_eq!(state.node_id, "node_test");
        assert_eq!(state.consensus.rounds_completed, 0);
    }

    #[test]
    fn test_v1_to_v2_conversion() {
        let v1 = StateV1 {
            schema_version: Some(1),
            system_state: "configured".to_string(),
            last_boot_time: None,
            configuration_hash: Some("abc123".to_string()),
        };

        let v2 = v1.to_v2("node_001".to_string());
        assert_eq!(v2.schema_version, 2);
        assert_eq!(v2.node_id, "node_001");
        assert_eq!(v2.system_state, "configured");
        assert_eq!(v2.configuration_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_update_consensus() {
        let mut state = StateV2::new("node_test".to_string());

        state.update_consensus(
            "round_001".to_string(),
            3,
            vec![],
        );

        assert_eq!(state.consensus.rounds_completed, 1);
        assert_eq!(state.consensus.last_round_id, Some("round_001".to_string()));
        assert_eq!(state.consensus.validator_count, 3);
    }
}
