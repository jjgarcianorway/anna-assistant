# Phase 1.9: Networked Consensus Integration

## Overview

Phase 1.9 expands the deterministic consensus PoC (1.8.0-alpha.1) into a minimal but operational networked consensus system. Multiple `annad` daemons communicate via HTTP JSON-RPC to reach quorum on signed observations.

**Status**: Minimal viable network implementation
**Version**: 1.9.0-alpha.1

## Architecture

```
┌──────────────┐       ┌──────────────┐       ┌──────────────┐
│   Node 1     │       │   Node 2     │       │   Node 3     │
│              │       │              │       │              │
│  Consensus   │◄─────►│  Consensus   │◄─────►│  Consensus   │
│  Engine      │  RPC  │  Engine      │  RPC  │  Engine      │
│              │       │              │       │              │
│  :8001       │       │  :8002       │       │  :8003       │
│  :9001/metrics│       │  :9002/metrics│       │  :9003/metrics│
└──────────────┘       └──────────────┘       └──────────────┘
        │                      │                      │
        └──────────────────────┴──────────────────────┘
                        Quorum Formation
```

## Network Protocol

### RPC Endpoints

All endpoints use HTTP JSON-RPC over TCP.

#### POST /rpc/submit
Submit a signed audit observation to the consensus engine.

**Request**:
```json
{
  "observation": {
    "node_id": "node_001",
    "audit_id": "audit_001",
    "round_id": "round_001",
    "window_hours": 24,
    "timestamp": "2025-11-12T12:00:00Z",
    "forecast_hash": "sha256:...",
    "outcome_hash": "sha256:...",
    "tis_components": {
      "prediction_accuracy": 0.85,
      "ethical_alignment": 0.80,
      "coherence_stability": 0.90
    },
    "tis_overall": 0.850,
    "bias_flags": [],
    "signature": [...]
  }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Observation accepted"
}
```

**Error Response** (Byzantine detection):
```json
{
  "success": false,
  "message": "Observation rejected (Byzantine detected)"
}
```

#### GET /rpc/status?round_id=<id>
Query consensus status for a specific round or all rounds.

**Response** (specific round):
```json
{
  "round_id": "round_001",
  "status": "Complete",
  "participating_nodes": ["node_001", "node_002", "node_003"],
  "total_observations": 3,
  "required_quorum": 2,
  "consensus_tis": 0.857,
  "consensus_biases": []
}
```

**Response** (all rounds):
```json
{
  "rounds": [
    {
      "round_id": "round_001",
      "status": "Complete",
      "observations": 3,
      "consensus_tis": 0.857
    }
  ],
  "byzantine_nodes": 0
}
```

#### POST /rpc/reconcile
Force consensus computation on all pending rounds.

**Response**:
```json
{
  "reconciled": 1,
  "message": "Reconciled 1 pending rounds"
}
```

### Peer Discovery

Peers are configured in `/etc/anna/peers.yml`:

```yaml
peers:
  - node_id: "node_001"
    address: "anna-node-1"
    port: 8001

  - node_id: "node_002"
    address: "anna-node-2"
    port: 8002

  - node_id: "node_003"
    address: "anna-node-3"
    port: 8003
```

The peer list is loaded at startup. Hot reload via SIGHUP is planned for Phase 1.10.

## Prometheus Metrics

Metrics are exposed on `/metrics` (port 9090 in testnet):

### Available Metrics

- `anna_consensus_rounds_total` - Total number of consensus rounds completed
- `anna_byzantine_nodes_total` - Current number of detected Byzantine nodes
- `anna_quorum_size` - Required quorum size for consensus
- `anna_average_tis` - Average temporal integrity score (planned)

**Example**:
```prometheus
# HELP anna_consensus_rounds_total Total number of consensus rounds completed
# TYPE anna_consensus_rounds_total counter
anna_consensus_rounds_total 3

# HELP anna_byzantine_nodes_total Current number of detected Byzantine nodes
# TYPE anna_byzantine_nodes_total gauge
anna_byzantine_nodes_total 0

# HELP anna_quorum_size Required quorum size for consensus
# TYPE anna_quorum_size gauge
anna_quorum_size 2
```

## Docker Testnet Deployment

### Prerequisites

- Docker
- Docker Compose
- 2 GB RAM minimum

### Quick Start

```bash
# Build testnet images
docker-compose build

# Start 3-node cluster
docker-compose up -d

# Check node status
curl http://localhost:8001/health
curl http://localhost:8002/health
curl http://localhost:8003/health

# View metrics
curl http://localhost:9001/metrics
curl http://localhost:9002/metrics
curl http://localhost:9003/metrics

# Submit test observation (requires signed observation JSON)
curl -X POST http://localhost:8001/rpc/submit \
  -H "Content-Type: application/json" \
  -d @test_observation.json

# Query consensus status
curl http://localhost:8001/rpc/status

# Stop cluster
docker-compose down
```

### Testnet Configuration

The testnet uses:
- **3 nodes**: anna-node-1, anna-node-2, anna-node-3
- **RPC ports**: 8001, 8002, 8003
- **Metrics ports**: 9001, 9002, 9003
- **Network**: Bridge network (anna-testnet)
- **Quorum threshold**: 2/3 (majority)

### Artifacts

Consensus state and artifacts are stored in:
- `./artifacts/testnet/node1/`
- `./artifacts/testnet/node2/`
- `./artifacts/testnet/node3/`

Each node persists:
- Consensus state: `consensus/state.json`
- Ed25519 keys: `keys/{node_id.pub, node_id.sec}`
- Round results: `rounds/round_{id}.json`

## State Schema v2 (Planned)

State v2 migration is **deferred to Phase 1.10** to focus on network functionality in 1.9.

**Planned v2 Schema**:
```json
{
  "schema_version": 2,
  "node_id": "node_001",
  "consensus_rounds": [...],
  "validator_count": 3,
  "byzantine_nodes": []
}
```

Migration will include:
- Forward-only migration with backup
- Checksum validation
- Automatic rollback on mismatch
- Preservation of audit_id monotonicity

## Security Model

- **Peer Authentication**: Ed25519 signatures on all observations
- **Byzantine Detection**: Double-submit detection within rounds
- **Advisory-Only**: All consensus outputs remain recommendations
- **Conscience Sovereignty**: User retains full control

## Limitations (Phase 1.9)

- ❌ **No TLS**: HTTP only (TLS planned for Phase 1.10)
- ❌ **No State v2 Migration**: Deferred to Phase 1.10
- ❌ **No Hot Peer Reload**: Requires restart
- ❌ **No Auto-Reconnect**: Connection errors are fatal
- ❌ **No CI Integration**: Smoke tests deferred to Phase 1.10

## Acceptance Criteria

✅ **3-node cluster reaches consensus** in ≥3 consecutive rounds
✅ **Metrics exposed** via /metrics
⏸️ **Backup/restore** (deferred to Phase 1.10)
⏸️ **CI smoke tests** (deferred to Phase 1.10)
✅ **All binaries compile** without errors

## Next Steps (Phase 1.10)

1. **State v2 Migration**: Forward-only with backup/restore
2. **TLS Support**: Encrypted peer communication
3. **Hot Peer Reload**: SIGHUP signal handling
4. **Auto-Reconnect**: Transient network error handling
5. **CI Integration**: Smoke tests for convergence and TIS drift
6. **Performance Testing**: Latency and throughput benchmarks

## References

- Phase 1.8 PoC: `docs/consensus_poc_user_guide.md`
- Consensus Engine: `crates/annad/src/consensus/mod.rs`
- Network Layer: `crates/annad/src/network/`
- Docker Testnet: `docker-compose.yml`

---

**Document Version**: 1.0
**Last Updated**: 2025-11-12
**Phase**: 1.9.0-alpha.1 (Network Integration)
