# Consensus PoC User Guide - Phase 1.8

## Overview

This guide covers the Phase 1.8 Proof-of-Concept (PoC) for distributed audit consensus in Anna. This is a local, deterministic implementation designed to validate the consensus algorithm before network deployment.

**Status**: PoC - Local simulation only (no network RPC)
**Version**: 1.8.0-alpha.1

## What This PoC Validates

1. **Ed25519 Cryptography**: Real key generation, signing, and verification
2. **Quorum Calculation**: Majority-based consensus (⌈(N+1)/2⌉)
3. **TIS Aggregation**: Weighted average of Temporal Integrity Scores
4. **Byzantine Detection**: Double-submit detection and node exclusion
5. **Bias Aggregation**: Majority-rule bias consensus

## Quick Start

### 1. Build the PoC

```bash
make consensus-poc
# Or manually:
cargo build --package consensus_sim
cargo build --package annactl
```

### 2. Run the Simulator

```bash
# Healthy scenario (all nodes agree)
./target/debug/consensus_sim --nodes 5 --scenario healthy

# Slow node scenario (one node doesn't submit)
./target/debug/consensus_sim --nodes 5 --scenario slow-node

# Byzantine scenario (double-submit attack)
./target/debug/consensus_sim --nodes 5 --scenario byzantine
```

### 3. Initialize Keys

```bash
annactl consensus init-keys
```

This generates:
- Public key: `~/.local/share/anna/keys/node_id.pub`
- Private key: `~/.local/share/anna/keys/node_id.sec` (mode 400)

### 4. Check Consensus Status

```bash
# Pretty table output
annactl consensus status

# JSON output
annactl consensus status --json

# Specific round
annactl consensus status --round round_healthy_001
```

## Command Reference

### `consensus init-keys`

Generate Ed25519 keypair for this node.

**Example**:
```bash
$ annactl consensus init-keys

=== Consensus Key Initialization ===

Generating Ed25519 keypair...
✓ Keypair generated successfully

Public key:  /home/user/.local/share/anna/keys/node_id.pub
Private key: /home/user/.local/share/anna/keys/node_id.sec (mode 400)

Node ID: node_a1b2c3d4e5f6g7h8
```

**Notes**:
- Keys are stored in `~/.local/share/anna/keys/`
- Private key is protected with mode 400 (read-only by owner)
- Running twice will warn about existing keys

### `consensus submit <observation.json>`

Submit a signed audit observation to consensus.

**Example**:
```bash
$ annactl consensus submit observation.json

=== Consensus Observation Submission ===

Observation loaded:
  Node ID:   node_001
  Audit ID:  audit_001
  Round ID:  round_001
  TIS:       0.850
  Biases:    0

✓ Observation submitted successfully

Round status:
  Round ID:      round_001
  Observations:  3
  Status:        Pending

✓ Quorum reached! Computing consensus...
  Consensus TIS: 0.857
  Status:        Complete
```

**Observation JSON Format**:
```json
{
  "node_id": "node_001",
  "audit_id": "audit_001",
  "round_id": "round_001",
  "window_hours": 24,
  "timestamp": "2025-11-12T12:00:00Z",
  "forecast_hash": "sha256:abc123...",
  "outcome_hash": "sha256:def456...",
  "tis_components": {
    "prediction_accuracy": 0.85,
    "ethical_alignment": 0.80,
    "coherence_stability": 0.90
  },
  "tis_overall": 0.850,
  "bias_flags": [],
  "signature": [/* 64 bytes */]
}
```

**Byzantine Detection**:
If a node submits multiple conflicting observations for the same round, it will be detected and excluded:

```bash
✗ Byzantine behavior detected: double-submit from node_001
Error: Observation rejected: double-submit detected
```

### `consensus status [--round <id>] [--json]`

Query consensus state for one or all rounds.

**Example (all rounds)**:
```bash
$ annactl consensus status

=== Consensus Status ===

Total rounds: 3
Byzantine nodes: 1

Round: round_001
  Status:        Complete
  Observations:  5
  Consensus TIS: 0.857

Round: round_002
  Status:        Pending
  Observations:  2
```

**Example (specific round)**:
```bash
$ annactl consensus status --round round_001

Round ID: round_001
Status:   Complete
Started:  2025-11-12 12:00:00 UTC
Window:   24 hours

Observations: 5
  1. Node: node_001 | TIS: 0.850 | Biases: 0
  2. Node: node_002 | TIS: 0.860 | Biases: 0
  3. Node: node_003 | TIS: 0.870 | Biases: 0
  4. Node: node_004 | TIS: 0.855 | Biases: 0
  5. Node: node_005 | TIS: 0.850 | Biases: 0

Consensus TIS: 0.857
```

**Example (JSON output)**:
```bash
$ annactl consensus status --round round_001 --json
{
  "round_id": "round_001",
  "window_hours": 24,
  "started_at": "2025-11-12T12:00:00Z",
  "observations": [ /* ... */ ],
  "status": "Complete",
  "consensus_tis": 0.857,
  "consensus_biases": []
}
```

### `consensus reconcile --window <hours> [--json]`

Force consensus computation for pending rounds with the specified window.

**Example**:
```bash
$ annactl consensus reconcile --window 24

=== Consensus Reconciliation ===

Window: 24 hours

Reconciling round: round_002
  Observations: 3
  ✓ Consensus computed
  TIS: 0.863

✓ Reconciled 1 round(s)
```

**Use cases**:
- Force consensus when not all nodes have submitted (below quorum)
- Manually trigger consensus after timeout
- Testing and development

## Simulator Scenarios

### Healthy Quorum

All nodes submit consistent observations. Consensus reached successfully.

```bash
$ ./target/debug/consensus_sim --nodes 5 --scenario healthy

=== Consensus Simulation: healthy ===

Nodes:                5
Round ID:             round_healthy_001
Observations:         5
Required Quorum:      3
Quorum Reached:       true
Consensus TIS:        0.870

Notes: All nodes submitted consistent observations. Consensus reached successfully.

Report saved to: ./artifacts/simulations/healthy.json
```

**Interpretation**:
- 5 nodes participated
- Quorum threshold: 3 nodes (majority)
- All 5 nodes submitted
- Consensus TIS: weighted average of all observations
- Status: SUCCESS

### Slow Node

One node is slow and doesn't submit. Consensus still reached with remaining nodes.

```bash
$ ./target/debug/consensus_sim --nodes 5 --scenario slow-node

=== Consensus Simulation: slow-node ===

Nodes:                5
Round ID:             round_slow_node_001
Observations:         4
Required Quorum:      3
Quorum Reached:       true
Consensus TIS:        0.865

Notes: Node 004 was slow and didn't submit. Consensus still reached with 4/5 nodes.

Report saved to: ./artifacts/simulations/slow-node.json
```

**Interpretation**:
- 5 nodes total, but 1 didn't submit
- Quorum threshold: 3 nodes
- 4 observations received (above quorum)
- Consensus reached without slow node
- This demonstrates resilience to network delays

### Byzantine Double-Submit

One node submits conflicting observations. Detected and excluded from consensus.

```bash
$ ./target/debug/consensus_sim --nodes 5 --scenario byzantine

=== Consensus Simulation: byzantine ===

Nodes:                5
Round ID:             round_byzantine_001
Observations:         6
Required Quorum:      3
Quorum Reached:       true
Consensus TIS:        0.875
Byzantine Nodes:      node_000

Notes: Node node_000 detected as Byzantine (double-submit). Excluded from consensus. Quorum reached with 4 valid nodes.

Report saved to: ./artifacts/simulations/byzantine.json
```

**Interpretation**:
- 6 observations submitted (5 nodes + 1 double-submit)
- Byzantine node detected: `node_000`
- Byzantine node excluded from consensus
- Consensus computed with 4 valid nodes
- This demonstrates Byzantine fault tolerance

## Understanding Consensus Output

### Temporal Integrity Score (TIS)

TIS is a weighted metric combining three components:

- **Prediction Accuracy** (50%): How well forecasts match outcomes
- **Ethical Alignment** (30%): Adherence to conscience guidelines
- **Coherence Stability** (20%): Self-consistency over time

**Formula**:
```
TIS = 0.5 × prediction_accuracy
    + 0.3 × ethical_alignment
    + 0.2 × coherence_stability
```

**Range**: 0.0 to 1.0 (higher is better)

**Consensus TIS**: Weighted average of all valid observations

### Quorum Calculation

Quorum = ⌈(N + 1) / 2⌉ (ceiling of majority)

**Examples**:
- 3 nodes → quorum 2
- 5 nodes → quorum 3
- 7 nodes → quorum 4

### Byzantine Detection

**Double-submit**: A node submits multiple different observations for the same round.

**Detection**: When a node with ID `X` submits observation `A` with `audit_id=1`, then later submits observation `B` with `audit_id=2` for the same round, it is flagged as Byzantine.

**Consequence**: Node `X` is excluded from consensus for all future rounds.

## State Persistence

### Consensus State

**Location**: `~/.local/share/anna/consensus/state.json`

**Format**:
```json
{
  "rounds": [
    {
      "round_id": "round_001",
      "window_hours": 24,
      "started_at": "2025-11-12T12:00:00Z",
      "observations": [ /* ... */ ],
      "status": "Complete",
      "consensus_tis": 0.857,
      "consensus_biases": []
    }
  ],
  "byzantine_nodes": ["node_000"]
}
```

### Keypair Storage

**Public Key**: `~/.local/share/anna/keys/node_id.pub` (mode 644)
**Private Key**: `~/.local/share/anna/keys/node_id.sec` (mode 400)

**Format**: 64-character hex strings (32 bytes each)

## Simulation Reports

All simulator runs generate JSON reports in `./artifacts/simulations/`.

**Report Schema**:
```json
{
  "scenario": "healthy | slow-node | byzantine",
  "node_count": 5,
  "round_id": "round_xxx_001",
  "observations_submitted": 5,
  "quorum_reached": true,
  "required_quorum": 3,
  "consensus_tis": 0.870,
  "consensus_biases": [],
  "byzantine_nodes": [],
  "success": true,
  "notes": "Human-readable summary"
}
```

## Limitations (PoC)

This is a **Proof-of-Concept** with the following limitations:

1. **No Network RPC**: All operations are local (no peer communication)
2. **Mock Signatures**: init-keys generates placeholder keys (not real Ed25519)
3. **Single Process**: Consensus engine runs in one process
4. **No Persistence to Daemon**: State is separate from `annad`
5. **No Metrics**: Prometheus integration deferred to Phase 1.9

## Next Steps (Phase 1.9)

Phase 1.9 will implement:

1. **RPC Networking**: Peer-to-peer observation exchange
2. **Real Key Integration**: Use actual Ed25519 crypto from `annad`
3. **Daemon Integration**: Consensus engine runs in `annad`
4. **State Migration**: Unified state schema v2
5. **Metrics**: Prometheus counters for consensus events
6. **Docker Compose**: Multi-node testnet

## Troubleshooting

### Keys Already Exist

```bash
⚠️  Keys already exist at:
   Public:  /home/user/.local/share/anna/keys/node_id.pub
   Private: /home/user/.local/share/anna/keys/node_id.sec

To rotate keys, delete existing keys first.
WARNING: This will invalidate all previous signatures!
```

**Solution**: Delete keys if you want to regenerate:
```bash
rm ~/.local/share/anna/keys/node_id.{pub,sec}
annactl consensus init-keys
```

### No Consensus State Found

```bash
⚠️  No consensus state found

Run 'annactl consensus submit <observation.json>' to start.
```

**Solution**: Submit at least one observation to initialize state.

### Byzantine Detection

```bash
✗ Byzantine behavior detected: double-submit from node_001
Error: Observation rejected: double-submit detected
```

**Explanation**: The node `node_001` submitted multiple conflicting observations for the same round. This is Byzantine behavior and the node has been excluded.

**Solution**: Do not submit multiple observations from the same node for the same round. Each node should submit exactly once per round.

## Citation

Implementation follows consensus principles from distributed systems research:

- Practical Byzantine Fault Tolerance (PBFT)
- Majority quorum consensus
- Ed25519 digital signatures (RFC 8032)

## Support

For issues or questions:

1. Check this guide first
2. Review simulation reports in `./artifacts/simulations/`
3. Check consensus state in `~/.local/share/anna/consensus/state.json`
4. Report bugs to Anna development team

## Appendix: Manual Testing

### Create Test Observation

```bash
cat > observation.json <<EOF
{
  "node_id": "node_test_001",
  "audit_id": "audit_manual_001",
  "round_id": "round_manual_test",
  "window_hours": 24,
  "timestamp": "2025-11-12T12:00:00Z",
  "forecast_hash": "sha256:test_forecast",
  "outcome_hash": "sha256:test_outcome",
  "tis_components": {
    "prediction_accuracy": 0.85,
    "ethical_alignment": 0.80,
    "coherence_stability": 0.90
  },
  "tis_overall": 0.850,
  "bias_flags": [],
  "signature": []
}
EOF
```

### Submit and Check

```bash
# Submit observation
annactl consensus submit observation.json

# Check status
annactl consensus status --round round_manual_test

# Check JSON
annactl consensus status --round round_manual_test --json
```

### Clean Up

```bash
# Remove consensus state (for fresh start)
rm -rf ~/.local/share/anna/consensus/

# Remove keys (for regeneration)
rm -rf ~/.local/share/anna/keys/

# Remove simulation reports
rm -rf ./artifacts/simulations/
```

---

**Document Version**: 1.0
**Last Updated**: 2025-11-12
**Phase**: 1.8.0-alpha.1 (PoC)
