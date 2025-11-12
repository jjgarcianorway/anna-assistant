# Mirror Audit State Schema v2

**Version**: 1.7.0-alpha.1
**Date**: 2025-11-12
**Migration Path**: Phase 1.6 (schema v1) → Phase 1.7 (schema v2)

---

## Overview

Phase 1.7 introduces distributed consensus for temporal integrity audits. The state schema is extended with consensus-specific fields while preserving backward compatibility with Phase 1.6.

**Key Changes**:
- `schema_version` field added (value: `2`)
- `node_id` field for node identity
- `consensus_rounds` array for round history
- `validator_count` for quorum calculations
- `byzantine_nodes` for fault detection

---

## Schema Comparison

### Schema v1 (Phase 1.6)

```rust
pub struct MirrorAuditState {
    pub total_audits: usize,
    pub last_audit_at: Option<DateTime<Utc>>,
    pub recent_integrity_scores: Vec<f64>,
    pub recent_audits: Vec<AuditEntry>,
}
```

**File**: `/var/lib/anna/mirror_audit/state.json`

**Example**:
```json
{
  "total_audits": 42,
  "last_audit_at": "2025-11-12T06:00:00Z",
  "recent_integrity_scores": [0.85, 0.82, 0.90, 0.88, 0.86],
  "recent_audits": [...]
}
```

### Schema v2 (Phase 1.7)

```rust
pub struct MirrorAuditState {
    // Phase 1.6 fields (preserved)
    pub total_audits: usize,
    pub last_audit_at: Option<DateTime<Utc>>,
    pub recent_integrity_scores: Vec<f64>,
    pub recent_audits: Vec<AuditEntry>,

    // Phase 1.7 additions
    pub schema_version: u8,                    // NEW: Always 2 for Phase 1.7
    pub node_id: Option<String>,               // NEW: Ed25519 fingerprint
    pub consensus_rounds: Vec<ConsensusRound>, // NEW: Round history
    pub validator_count: usize,                // NEW: Total nodes in network
    pub byzantine_nodes: Vec<ByzantineNode>,   // NEW: Excluded nodes
}
```

**File**: `/var/lib/anna/mirror_audit/state.json` (same location)

**Example**:
```json
{
  "schema_version": 2,
  "total_audits": 42,
  "last_audit_at": "2025-11-12T06:00:00Z",
  "recent_integrity_scores": [0.85, 0.82, 0.90, 0.88, 0.86],
  "recent_audits": [...],
  "node_id": "node_a1b2c3d4e5f60718",
  "consensus_rounds": [
    {
      "round_id": "550e8400-e29b-41d4-a716-446655440000",
      "window_hours": 24,
      "started_at": "2025-11-12T06:00:00Z",
      "observations": [...],
      "status": "Complete",
      "consensus_tis": 0.84,
      "consensus_biases": ["RecencyBias"]
    }
  ],
  "validator_count": 3,
  "byzantine_nodes": []
}
```

---

## Migration Strategy

### Automatic Migration (Preferred)

**Trigger**: Daemon detects `schema_version` field missing or `schema_version < 2`

**Process**:
1. Load existing state.json
2. Detect schema version:
   - If `schema_version` field absent → assume v1
   - If `schema_version == 1` → explicit v1
3. Create backup: `state.json.v1.backup`
4. Add v2 fields with defaults:
   - `schema_version = 2`
   - `node_id = None` (will be set on first consensus operation)
   - `consensus_rounds = []`
   - `validator_count = 1` (single-node default)
   - `byzantine_nodes = []`
5. Write migrated state
6. Log: "Migrated mirror audit state from v1 to v2"

**Code Location**: `crates/annad/src/mirror_audit/state.rs::migrate_state_v1_to_v2()`

**Example**:
```rust
pub async fn load_or_migrate_state(path: &str) -> Result<MirrorAuditState> {
    let json = fs::read_to_string(path).await?;
    let mut value: serde_json::Value = serde_json::from_str(&json)?;

    // Detect schema version
    let schema_version = value.get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);  // Default to v1 if missing

    if schema_version < 2 {
        info!("Migrating mirror audit state from v{} to v2", schema_version);

        // Backup original
        let backup_path = format!("{}.v{}.backup", path, schema_version);
        fs::copy(path, &backup_path).await?;
        info!("Created backup: {}", backup_path);

        // Add v2 fields
        value["schema_version"] = json!(2);
        value["node_id"] = json!(null);
        value["consensus_rounds"] = json!([]);
        value["validator_count"] = json!(1);
        value["byzantine_nodes"] = json!([]);

        // Write migrated state
        let migrated_json = serde_json::to_string_pretty(&value)?;
        fs::write(path, migrated_json).await?;
        info!("Migration complete: {} -> v2", path);
    }

    // Now deserialize as v2
    let json = fs::read_to_string(path).await?;
    let state: MirrorAuditState = serde_json::from_str(&json)?;
    Ok(state)
}
```

### Manual Migration (Fallback)

If automatic migration fails, operator can manually migrate:

```bash
# Backup original
sudo cp /var/lib/anna/mirror_audit/state.json \
        /var/lib/anna/mirror_audit/state.json.v1.backup

# Use provided migration script
sudo bash scripts/migrate_mirror_audit_v1_to_v2.sh

# Verify
sudo cat /var/lib/anna/mirror_audit/state.json | jq '.schema_version'
# Expected output: 2
```

**Migration Script**: `scripts/migrate_mirror_audit_v1_to_v2.sh`
```bash
#!/bin/bash
set -e

STATE_FILE="/var/lib/anna/mirror_audit/state.json"
BACKUP_FILE="${STATE_FILE}.v1.backup"

if [ ! -f "$STATE_FILE" ]; then
    echo "Error: $STATE_FILE not found"
    exit 1
fi

# Backup
cp "$STATE_FILE" "$BACKUP_FILE"
echo "Backup created: $BACKUP_FILE"

# Migrate using jq
jq '. + {
    "schema_version": 2,
    "node_id": null,
    "consensus_rounds": [],
    "validator_count": 1,
    "byzantine_nodes": []
}' "$BACKUP_FILE" > "$STATE_FILE"

echo "Migration complete"
jq '.schema_version' "$STATE_FILE"
```

---

## Rollback Plan

### Rollback Trigger

- Consensus features cause instability
- Operator decides to revert to Phase 1.6
- Byzantine detection false positives

### Rollback Steps

1. **Stop daemon**:
   ```bash
   sudo systemctl stop anna-daemon
   ```

2. **Restore v1 state** (if backup exists):
   ```bash
   sudo cp /var/lib/anna/mirror_audit/state.json.v1.backup \
           /var/lib/anna/mirror_audit/state.json
   ```

3. **Downgrade binaries** to Phase 1.6:
   ```bash
   # Install Phase 1.6 package
   sudo dpkg -i anna-daemon_1.6.0-rc.1_amd64.deb

   # Or rebuild from source
   git checkout v1.6.0-rc.1
   cargo build --release --bins
   sudo install -m 0755 target/release/annad /usr/bin/annad
   sudo install -m 0755 target/release/annactl /usr/bin/annactl
   ```

4. **Verify version**:
   ```bash
   annactl --version
   # Expected: annactl 1.6.0-rc.1
   ```

5. **Start daemon**:
   ```bash
   sudo systemctl start anna-daemon
   ```

6. **Verify functionality**:
   ```bash
   annactl mirror audit-forecast
   # Should work with v1 state
   ```

### Rollback Safety

- **v1 fields preserved**: Schema v2 is a superset of v1
- **Backward compatibility**: Phase 1.6 daemon ignores v2 fields (serde `#[serde(default)]`)
- **Audit history intact**: All audit entries remain valid
- **No data loss**: Only consensus-specific data is lost (rounds, byzantine_nodes)

---

## Field Definitions

### schema_version

- **Type**: `u8`
- **Values**:
  - `1` (implicit): Phase 1.6 state
  - `2`: Phase 1.7 state
- **Default**: `1` (when field absent)
- **Immutable**: Value never decreases
- **Migration**: Set to `2` during migration

### node_id

- **Type**: `Option<String>`
- **Format**: `"node_<first 16 chars of Ed25519 pubkey>"`
- **Example**: `"node_a1b2c3d4e5f60718"`
- **Derivation**:
  ```rust
  let pubkey_hex = hex::encode(pubkey.as_bytes());
  let fingerprint = &pubkey_hex[0..16];
  format!("node_{}", fingerprint)
  ```
- **Initialization**: Set on first `annactl consensus init-keys` or daemon start
- **Persistence**: Stored in state for identity consistency

### consensus_rounds

- **Type**: `Vec<ConsensusRound>`
- **Retention**: Last 100 rounds (configurable)
- **Ordering**: Newest first
- **Cleanup**: Automatic pruning on state save

```rust
pub struct ConsensusRound {
    pub round_id: String,          // UUID v4
    pub window_hours: u64,
    pub started_at: DateTime<Utc>,
    pub observations: Vec<AuditObservation>,
    pub status: RoundStatus,       // Pending, Complete, Failed
    pub consensus_tis: Option<f64>,
    pub consensus_biases: Vec<BiasKind>,
}
```

### validator_count

- **Type**: `usize`
- **Default**: `1` (single node)
- **Source**: Count of peers in `/etc/anna/peers.yml` + 1 (self)
- **Quorum Calculation**: `⌈(validator_count + 1) / 2⌉`
- **Update Frequency**: On daemon start and peer config reload

### byzantine_nodes

- **Type**: `Vec<ByzantineNode>`
- **Retention**: Permanent (until manual removal)
- **Alert**: Logged to `/var/log/anna/consensus.log`

```rust
pub struct ByzantineNode {
    pub node_id: String,
    pub detected_at: DateTime<Utc>,
    pub reason: ByzantineReason,
    pub excluded_until: Option<DateTime<Utc>>,  // None = manual review
}

pub enum ByzantineReason {
    ConflictingObservations,
    ExcessiveDeviation,
    InvalidSignature,
}
```

---

## Backward Compatibility

### Reading v1 State in v2 Daemon

Phase 1.7 daemon uses `#[serde(default)]` for v2 fields:

```rust
#[derive(Serialize, Deserialize)]
pub struct MirrorAuditState {
    // v1 fields (no defaults needed, always present)
    pub total_audits: usize,
    pub last_audit_at: Option<DateTime<Utc>>,
    pub recent_integrity_scores: Vec<f64>,
    pub recent_audits: Vec<AuditEntry>,

    // v2 fields (with defaults for backward compat)
    #[serde(default = "default_schema_version")]
    pub schema_version: u8,

    #[serde(default)]
    pub node_id: Option<String>,

    #[serde(default)]
    pub consensus_rounds: Vec<ConsensusRound>,

    #[serde(default = "default_validator_count")]
    pub validator_count: usize,

    #[serde(default)]
    pub byzantine_nodes: Vec<ByzantineNode>,
}

fn default_schema_version() -> u8 { 1 }
fn default_validator_count() -> usize { 1 }
```

### Reading v2 State in v1 Daemon

Phase 1.6 daemon ignores unknown fields (serde default behavior):

```rust
// Phase 1.6 struct (no v2 fields)
pub struct MirrorAuditState {
    pub total_audits: usize,
    pub last_audit_at: Option<DateTime<Utc>>,
    pub recent_integrity_scores: Vec<f64>,
    pub recent_audits: Vec<AuditEntry>,
}

// When deserializing v2 state, extra fields are silently ignored
```

**Warning**: Phase 1.6 daemon will NOT write v2 fields on state save. Consensus data will be lost.

---

## Validation

### State Integrity Checks

**On Load**:
```rust
pub fn validate_state_v2(state: &MirrorAuditState) -> Result<()> {
    // Check schema version
    if state.schema_version < 2 {
        return Err(anyhow!("Invalid schema version: {}", state.schema_version));
    }

    // Validate node_id format
    if let Some(node_id) = &state.node_id {
        if !node_id.starts_with("node_") || node_id.len() != 21 {
            return Err(anyhow!("Invalid node_id format: {}", node_id));
        }
    }

    // Validate validator_count
    if state.validator_count == 0 {
        return Err(anyhow!("validator_count must be >= 1"));
    }

    // Validate consensus rounds
    for round in &state.consensus_rounds {
        validate_consensus_round(round)?;
    }

    Ok(())
}
```

### Migration Validation

**Post-Migration**:
```bash
# Check schema version
jq '.schema_version' /var/lib/anna/mirror_audit/state.json
# Expected: 2

# Check v1 fields preserved
jq '.total_audits, .last_audit_at, .recent_integrity_scores | length' \
   /var/lib/anna/mirror_audit/state.json

# Check v2 fields initialized
jq '.node_id, .consensus_rounds | length, .validator_count, .byzantine_nodes | length' \
   /var/lib/anna/mirror_audit/state.json
```

---

## Testing

### Unit Tests

**Location**: `crates/annad/src/mirror_audit/state.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1_state = r#"
        {
          "total_audits": 10,
          "last_audit_at": "2025-11-12T06:00:00Z",
          "recent_integrity_scores": [0.85, 0.82],
          "recent_audits": []
        }
        "#;

        let state: MirrorAuditState = serde_json::from_str(v1_state).unwrap();
        assert_eq!(state.schema_version, 1);  // Default
        assert_eq!(state.node_id, None);
        assert_eq!(state.consensus_rounds.len(), 0);
        assert_eq!(state.validator_count, 1);
    }

    #[test]
    fn test_v2_state_roundtrip() {
        let state = MirrorAuditState {
            schema_version: 2,
            total_audits: 42,
            node_id: Some("node_a1b2c3d4e5f60718".to_string()),
            validator_count: 3,
            // ... other fields
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MirrorAuditState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.schema_version, 2);
        assert_eq!(deserialized.node_id, state.node_id);
    }
}
```

### Integration Tests

**Scenario 1**: Fresh install (no existing state)
- Expected: State created with schema_version=2

**Scenario 2**: Upgrade from Phase 1.6
- Expected: Automatic migration, backup created

**Scenario 3**: Downgrade to Phase 1.6
- Expected: v1 daemon ignores v2 fields, functionality preserved

**Scenario 4**: Re-upgrade to Phase 1.7
- Expected: Recognizes v2 state, no migration needed

---

## Monitoring

### State File Size

```bash
# Expected size growth with consensus
du -h /var/lib/anna/mirror_audit/state.json

# Phase 1.6: ~5-10 KB (50 audits)
# Phase 1.7: ~20-50 KB (50 audits + 100 rounds)
```

### Migration Logs

```bash
# Check for migration events
sudo journalctl -u anna-daemon | grep -i "migrat"

# Expected output:
# INFO Migrating mirror audit state from v1 to v2
# INFO Created backup: /var/lib/anna/mirror_audit/state.json.v1.backup
# INFO Migration complete: /var/lib/anna/mirror_audit/state.json -> v2
```

---

## Appendix: Full Schema Examples

### Schema v1 (Phase 1.6)

<details>
<summary>Click to expand</summary>

```json
{
  "total_audits": 15,
  "last_audit_at": "2025-11-12T06:00:00Z",
  "recent_integrity_scores": [0.82, 0.85, 0.80, 0.88, 0.86],
  "recent_audits": [
    {
      "forecast_id": "forecast_123",
      "predicted": {
        "health_score": 0.85,
        "empathy_index": 0.75,
        "strain_index": 0.30,
        "network_coherence": 0.90,
        "avg_trust_score": 0.80
      },
      "actual": {
        "health_score": 0.82,
        "empathy_index": 0.73,
        "strain_index": 0.32,
        "network_coherence": 0.88,
        "avg_trust_score": 0.78
      },
      "errors": {
        "mean_absolute_error": 0.03,
        "rmse": 0.035
      },
      "temporal_integrity_score": {
        "overall": 0.82,
        "prediction_accuracy": 0.97,
        "ethical_alignment": 0.75,
        "coherence_stability": 0.98
      },
      "bias_findings": [],
      "adjustment_plan": null
    }
  ]
}
```
</details>

### Schema v2 (Phase 1.7)

<details>
<summary>Click to expand</summary>

```json
{
  "schema_version": 2,
  "total_audits": 15,
  "last_audit_at": "2025-11-12T06:00:00Z",
  "recent_integrity_scores": [0.82, 0.85, 0.80, 0.88, 0.86],
  "recent_audits": [...],
  "node_id": "node_a1b2c3d4e5f60718",
  "consensus_rounds": [
    {
      "round_id": "550e8400-e29b-41d4-a716-446655440000",
      "window_hours": 24,
      "started_at": "2025-11-12T06:00:00Z",
      "observations": [
        {
          "node_id": "node_a1b2c3d4e5f60718",
          "audit_id": "audit_abc123",
          "round_id": "550e8400-e29b-41d4-a716-446655440000",
          "window_hours": 24,
          "timestamp": "2025-11-12T06:00:15Z",
          "forecast_hash": "sha256:3a8b9c1d...",
          "outcome_hash": "sha256:9b8a7f6e...",
          "tis_components": {
            "prediction_accuracy": 0.97,
            "ethical_alignment": 0.75,
            "coherence_stability": 0.98
          },
          "tis_overall": 0.82,
          "bias_flags": [],
          "signature": [...]
        }
      ],
      "status": "Complete",
      "consensus_tis": 0.84,
      "consensus_biases": []
    }
  ],
  "validator_count": 3,
  "byzantine_nodes": []
}
```
</details>

---

**Citation**: [archwiki:System_maintenance]
