// Phase 1.7: Consensus State Persistence (Schema v2)
// Status: STUB - Type definitions and migration scaffolding

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info, warn};

use super::{ByzantineNode, ConsensusRound, NodeId};

// ============================================================================
// MIRROR AUDIT STATE (SCHEMA V2)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditStateV2 {
    // Phase 1.6 fields (preserved)
    pub total_audits: usize,
    pub last_audit_at: Option<DateTime<Utc>>,
    pub recent_integrity_scores: Vec<f64>,

    // Phase 1.7 additions
    #[serde(default = "default_schema_version")]
    pub schema_version: u8,

    #[serde(default)]
    pub node_id: Option<NodeId>,

    #[serde(default)]
    pub consensus_rounds: Vec<ConsensusRound>,

    #[serde(default = "default_validator_count")]
    pub validator_count: usize,

    #[serde(default)]
    pub byzantine_nodes: Vec<ByzantineNode>,
}

fn default_schema_version() -> u8 {
    1
}

fn default_validator_count() -> usize {
    1
}

impl Default for MirrorAuditStateV2 {
    fn default() -> Self {
        Self {
            total_audits: 0,
            last_audit_at: None,
            recent_integrity_scores: Vec::new(),
            schema_version: 2,
            node_id: None,
            consensus_rounds: Vec::new(),
            validator_count: 1,
            byzantine_nodes: Vec::new(),
        }
    }
}

// ============================================================================
// STATE VALIDATION
// ============================================================================

impl MirrorAuditStateV2 {
    /// Validate state integrity
    pub fn validate(&self) -> Result<()> {
        // Check schema version
        if self.schema_version < 2 {
            return Err(anyhow!(
                "Invalid schema version: {} (expected 2)",
                self.schema_version
            ));
        }

        // Validate node_id format
        if let Some(node_id) = &self.node_id {
            if !node_id.starts_with("node_") || node_id.len() != 21 {
                return Err(anyhow!("Invalid node_id format: {}", node_id));
            }
        }

        // Validate validator_count
        if self.validator_count == 0 {
            return Err(anyhow!("validator_count must be >= 1"));
        }

        Ok(())
    }

    /// Prune old consensus rounds (keep last N)
    pub fn prune_rounds(&mut self, keep: usize) {
        if self.consensus_rounds.len() > keep {
            debug!(
                "Pruning consensus rounds: {} -> {}",
                self.consensus_rounds.len(),
                keep
            );
            self.consensus_rounds.drain(0..self.consensus_rounds.len() - keep);
        }
    }
}

// ============================================================================
// MIGRATION
// ============================================================================

/// Migrate state from v1 to v2
pub async fn migrate_v1_to_v2(path: &str) -> Result<()> {
    info!("Starting migration from schema v1 to v2");

    // Read existing state
    let json = fs::read_to_string(path).await?;
    let mut value: serde_json::Value = serde_json::from_str(&json)?;

    // Detect schema version
    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    if schema_version >= 2 {
        info!("State already at schema v{}, no migration needed", schema_version);
        return Ok(());
    }

    // Create backup
    let backup_path = format!("{}.v{}.backup", path, schema_version);
    fs::copy(path, &backup_path).await?;
    info!("Created backup: {}", backup_path);

    // Add v2 fields
    value["schema_version"] = serde_json::json!(2);
    value["node_id"] = serde_json::json!(null);
    value["consensus_rounds"] = serde_json::json!([]);
    value["validator_count"] = serde_json::json!(1);
    value["byzantine_nodes"] = serde_json::json!([]);

    // Write migrated state
    let migrated_json = serde_json::to_string_pretty(&value)?;
    fs::write(path, migrated_json).await?;
    info!("Migration complete: {} -> v2", path);

    Ok(())
}

/// Load state with automatic migration
pub async fn load_or_migrate_state(path: &str) -> Result<MirrorAuditStateV2> {
    debug!("Loading state from: {}", path);

    // Check if file exists
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        info!("State file not found, creating new v2 state");
        return Ok(MirrorAuditStateV2::default());
    }

    // Read file
    let json = fs::read_to_string(path).await?;
    let value: serde_json::Value = serde_json::from_str(&json)?;

    // Detect schema version
    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    // Migrate if needed
    if schema_version < 2 {
        warn!("Detected schema v{}, migrating to v2", schema_version);
        migrate_v1_to_v2(path).await?;

        // Re-read after migration
        let json = fs::read_to_string(path).await?;
        let state: MirrorAuditStateV2 = serde_json::from_str(&json)?;
        state.validate()?;
        return Ok(state);
    }

    // Deserialize v2 state
    let state: MirrorAuditStateV2 = serde_json::from_str(&json)?;
    state.validate()?;
    Ok(state)
}

/// Save state to disk
pub async fn save_state(state: &MirrorAuditStateV2, path: &str) -> Result<()> {
    debug!("Saving state to: {}", path);

    // Validate before save
    state.validate()?;

    // Serialize
    let json = serde_json::to_string_pretty(state)?;

    // Write atomically (write to temp, then rename)
    let temp_path = format!("{}.tmp", path);
    fs::write(&temp_path, json).await?;
    fs::rename(&temp_path, path).await?;

    debug!("State saved successfully");
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_v2() {
        let state = MirrorAuditStateV2::default();
        assert_eq!(state.schema_version, 2);
        assert_eq!(state.validator_count, 1);
        assert_eq!(state.consensus_rounds.len(), 0);
        assert_eq!(state.byzantine_nodes.len(), 0);
    }

    #[test]
    fn test_state_validation() {
        let state = MirrorAuditStateV2::default();
        assert!(state.validate().is_ok());

        let mut invalid_state = state.clone();
        invalid_state.schema_version = 1;
        assert!(invalid_state.validate().is_err());

        let mut invalid_node_id = state.clone();
        invalid_node_id.node_id = Some("invalid".to_string());
        assert!(invalid_node_id.validate().is_err());

        let mut zero_validators = state.clone();
        zero_validators.validator_count = 0;
        assert!(zero_validators.validate().is_err());
    }

    #[test]
    fn test_prune_rounds() {
        let mut state = MirrorAuditStateV2::default();

        // Add 10 dummy rounds
        for i in 0..10 {
            state.consensus_rounds.push(ConsensusRound {
                round_id: format!("round_{}", i),
                window_hours: 24,
                started_at: Utc::now(),
                observations: Vec::new(),
                status: super::super::RoundStatus::Pending,
                consensus_tis: None,
                consensus_biases: Vec::new(),
            });
        }

        assert_eq!(state.consensus_rounds.len(), 10);

        // Prune to 5
        state.prune_rounds(5);
        assert_eq!(state.consensus_rounds.len(), 5);

        // Pruning again with same limit should not change
        state.prune_rounds(5);
        assert_eq!(state.consensus_rounds.len(), 5);

        // Pruning with larger limit should not change
        state.prune_rounds(10);
        assert_eq!(state.consensus_rounds.len(), 5);
    }

    #[test]
    fn test_v2_serialization_roundtrip() {
        let state = MirrorAuditStateV2 {
            total_audits: 42,
            last_audit_at: Some(Utc::now()),
            recent_integrity_scores: vec![0.85, 0.82, 0.90],
            schema_version: 2,
            node_id: Some("node_a1b2c3d4e5f60718".to_string()),
            consensus_rounds: Vec::new(),
            validator_count: 3,
            byzantine_nodes: Vec::new(),
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MirrorAuditStateV2 = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.schema_version, 2);
        assert_eq!(deserialized.total_audits, 42);
        assert_eq!(deserialized.validator_count, 3);
        assert_eq!(
            deserialized.node_id,
            Some("node_a1b2c3d4e5f60718".to_string())
        );
    }

    #[test]
    fn test_v1_compatibility() {
        // Simulate v1 state (missing v2 fields)
        let v1_json = r#"
        {
            "total_audits": 10,
            "last_audit_at": "2025-11-12T06:00:00Z",
            "recent_integrity_scores": [0.85, 0.82]
        }
        "#;

        let state: MirrorAuditStateV2 = serde_json::from_str(v1_json).unwrap();

        // Check defaults applied
        assert_eq!(state.schema_version, 1); // Default when missing
        assert_eq!(state.node_id, None);
        assert_eq!(state.consensus_rounds.len(), 0);
        assert_eq!(state.validator_count, 1);
        assert_eq!(state.byzantine_nodes.len(), 0);

        // v1 fields preserved
        assert_eq!(state.total_audits, 10);
        assert_eq!(state.recent_integrity_scores.len(), 2);
    }
}
