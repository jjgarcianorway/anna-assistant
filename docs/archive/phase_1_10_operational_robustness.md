# Phase 1.10: Operational Robustness and Validation

## Overview

Phase 1.10 hardens the Phase 1.9 network foundation into an operationally reliable distributed consensus system with state migration, extended observability, and validation infrastructure.

**Status**: Operational hardening - State migration and metrics complete
**Version**: 1.10.0-alpha.1

## What's New in 1.10

### 1. State Schema v2 Migration ✅

Forward-only migration from v1 to v2 with automatic backup and rollback on failure.

**Features**:
- Automatic backup creation (`state.backup.v1`)
- SHA256 checksum verification
- Atomic migration with rollback on failure
- Audit log entries for all migration events
- Preservation of audit_id monotonicity

**Usage**:
```bash
# Migration happens automatically on first start
sudo systemctl start annad

# Check migration status
journalctl -u annad | grep migration

# Manual migration (if needed)
annactl migrate-state-v2

# Verify state version
cat /var/lib/anna/state.json | jq '.schema_version'
# Should output: 2
```

**Migration Process**:
1. Create backup: `state.backup.v1`
2. Compute SHA256 checksum: `state.backup.v1.sha256`
3. Load v1 state
4. Convert to v2 schema
5. Save as `state.v2.json`
6. Verify backup checksum
7. If valid: Replace `state.json` with v2
8. If invalid: Rollback to v1, log CRITICAL, exit code 78

**State v2 Schema**:
```json
{
  "schema_version": 2,
  "node_id": "node_001",
  "created_at": "2025-11-12T12:00:00Z",
  "updated_at": "2025-11-12T12:05:00Z",

  "system_state": "configured",
  "last_boot_time": "2025-11-12T00:00:00Z",
  "configuration_hash": "abc123",

  "consensus": {
    "validator_count": 3,
    "rounds_completed": 10,
    "last_round_id": "round_010",
    "last_round_at": "2025-11-12T12:05:00Z",
    "byzantine_nodes": []
  },

  "network": {
    "peer_count": 2,
    "last_peer_reload": "2025-11-12T12:00:00Z",
    "tls_enabled": false
  }
}
```

### 2. Extended Prometheus Metrics ✅

Additional metrics for operational observability.

**New Metrics** (Phase 1.10):
- `anna_average_tis` (gauge) - Average temporal integrity score
- `anna_peer_request_total{peer,status}` (counter) - Peer request counters
- `anna_peer_reload_total{result}` (counter) - Peer reload events
- `anna_migration_events_total{result}` (counter) - Migration events

**Existing Metrics** (Phase 1.9):
- `anna_consensus_rounds_total` (counter)
- `anna_byzantine_nodes_total` (gauge)
- `anna_quorum_size` (gauge)

**Query Examples**:
```bash
# Get all consensus metrics
curl http://localhost:9090/metrics | grep ^anna_

# Check average TIS
curl -s http://localhost:9090/metrics | grep anna_average_tis

# Count peer requests by status
curl -s http://localhost:9090/metrics | grep anna_peer_request_total
```

**Example Output**:
```prometheus
# HELP anna_consensus_rounds_total Total number of consensus rounds completed
# TYPE anna_consensus_rounds_total counter
anna_consensus_rounds_total 3

# HELP anna_average_tis Average temporal integrity score across recent rounds
# TYPE anna_average_tis gauge
anna_average_tis 0.867

# HELP anna_peer_request_total Total number of peer requests by peer and status
# TYPE anna_peer_request_total counter
anna_peer_request_total{peer="node_002",status="success"} 5
anna_peer_request_total{peer="node_003",status="success"} 5

# HELP anna_migration_events_total Total number of state migration events
# TYPE anna_migration_events_total counter
anna_migration_events_total{result="success"} 1
```

### 3. TLS/mTLS Support (Foundation Ready)

Infrastructure prepared for TLS, implementation deferred due to context limits.

**Configuration** (planned):
```yaml
# /etc/anna/peers.yml
allow_insecure_peers: false  # Default: require TLS

tls:
  enabled: true
  ca_cert: /etc/anna/tls/ca.pem
  server_cert: /etc/anna/tls/server.pem
  server_key: /etc/anna/tls/server.key
  client_cert: /etc/anna/tls/client.pem
  client_key: /etc/anna/tls/client.key

peers:
  - node_id: node_001
    address: anna-node-1
    port: 8001
```

**Security Model**:
- mTLS required for `/rpc/*` endpoints
- Client certificate verification enforced
- Insecure mode loudly logged: `WARN: Running with allow_insecure_peers=true - TLS DISABLED`
- Self-signed CA support for testing

### 4. Hot Reload via SIGHUP (Foundation Ready)

Infrastructure prepared, implementation deferred.

**Planned Usage**:
```bash
# Edit peer configuration
sudo vim /etc/anna/peers.yml

# Reload without restart
sudo systemctl reload annad
# OR
sudo kill -HUP $(cat /run/anna/annad.pid)

# Check reload metrics
curl -s http://localhost:9090/metrics | grep anna_peer_reload_total
```

**Validation**:
- YAML schema validation
- Certificate existence checks (if TLS enabled)
- Reject invalid updates, keep last good configuration
- Atomic reload: all-or-nothing

### 5. Testnet Validation Script ✅

3-round consensus test with artifact collection.

**Usage**:
```bash
# Start testnet
docker-compose up -d

# Wait for nodes to be ready (5-10 seconds)
sleep 10

# Run 3-round test
./testnet/scripts/run_rounds.sh

# Check artifacts
ls -la ./artifacts/testnet/
ls -la ./artifacts/testnet/round_1/
cat ./artifacts/testnet/round_1/node_1.json | jq

# View metrics
cat ./artifacts/testnet/node_1_metrics.txt | grep anna_
```

**Test Scenarios**:
1. **Round 1**: Healthy quorum (all 3 nodes agree)
2. **Round 2**: Slow node (1 node delayed)
3. **Round 3**: Byzantine detection (conflicting observations)

**Artifacts**:
```
./artifacts/testnet/
├── round_1/
│   ├── node_1.json
│   ├── node_2.json
│   └── node_3.json
├── round_2/
│   └── ...
├── round_3/
│   └── ...
├── node_1_metrics.txt
├── node_2_metrics.txt
└── node_3_metrics.txt
```

## Migration Guide

### Migrating from Phase 1.9 to 1.10

**1. Backup Existing State** (recommended):
```bash
sudo cp /var/lib/anna/state.json /var/lib/anna/state.manual.backup
```

**2. Update Anna**:
```bash
# Stop daemon
sudo systemctl stop annad

# Update binaries
sudo make install

# Start daemon (migration happens automatically)
sudo systemctl start annad
```

**3. Verify Migration**:
```bash
# Check logs
sudo journalctl -u annad -n 50 | grep migration

# Should see:
# INFO Starting state v1 → v2 migration
# INFO Creating backup of v1 state
# INFO Computing checksum of backup
# INFO Loading v1 state
# INFO Converting v1 state to v2
# INFO Saving v2 state
# INFO Verifying backup checksum
# INFO ✓ State migration v1 → v2 completed successfully

# Verify state version
sudo cat /var/lib/anna/state.json | jq '.schema_version'
# Output: 2

# Check backup exists
ls -lh /var/lib/anna/state.backup.v1*
```

**4. Check Metrics**:
```bash
curl -s http://localhost:9090/metrics | grep anna_migration_events_total
# anna_migration_events_total{result="success"} 1
```

### Rollback Procedure

If migration fails, rollback happens automatically. Manual rollback:

```bash
# Stop daemon
sudo systemctl stop annad

# Verify backup checksum
cd /var/lib/anna
sha256sum -c state.backup.v1.sha256

# If valid, restore manually
sudo cp state.backup.v1 state.json

# Restart daemon
sudo systemctl start annad
```

**Exit Codes**:
- `78`: Migration verification failed, automatic rollback performed

## Monitoring and Observability

### Metrics Dashboard (Grafana)

**Consensus Health Panel**:
```promql
# Rounds completed
rate(anna_consensus_rounds_total[5m])

# Average TIS trend
anna_average_tis

# Byzantine node count
anna_byzantine_nodes_total

# Peer success rate
rate(anna_peer_request_total{status="success"}[5m])
/
rate(anna_peer_request_total[5m])
```

### Log Inspection

**Migration Events**:
```bash
sudo journalctl -u annad | grep -E "migration|rollback"
```

**Consensus Events**:
```bash
sudo journalctl -u annad | grep -E "quorum|consensus|byzantine"
```

**Audit Log**:
```bash
# View state migration audit trail
sudo cat /var/log/anna/audit.jsonl | jq 'select(.event == "state_migration")'
```

## Common Failure Modes

### 1. Migration Checksum Mismatch

**Symptoms**:
- Daemon exits with code 78
- Log: `CRITICAL: Rolling back to v1 state`
- Log: `Backup checksum verification failed`

**Cause**: Backup file corrupted or modified during migration

**Resolution**:
- Automatic rollback to v1
- Check disk health: `sudo smartctl -a /dev/sda`
- Retry migration after fixing storage issues

### 2. Peer Communication Failures

**Symptoms**:
- `anna_peer_request_total{status="error"}` increasing
- Consensus rounds not completing

**Cause**: Network issues, peer down, configuration mismatch

**Resolution**:
```bash
# Check peer health
for port in 8001 8002 8003; do
  curl -s http://localhost:$port/health || echo "Port $port down"
done

# Check docker network
docker network inspect anna-testnet

# Restart failed nodes
docker-compose restart anna-node-2
```

### 3. TIS Drift > 0.01

**Symptoms**:
- Nodes reporting different consensus_tis values
- Divergent Byzantine node lists

**Cause**: Network partition, state desync, Byzantine behavior

**Resolution**:
```bash
# Force reconciliation on all nodes
for port in 8001 8002 8003; do
  curl -X POST http://localhost:$port/rpc/reconcile
done

# Compare state
for port in 8001 8002 8003; do
  echo "=== Node $port ==="
  curl -s http://localhost:$port/rpc/status | jq '.rounds[-1]'
done
```

## Testnet Quick Start

```bash
# 1. Build
make consensus-poc

# 2. Start 3-node cluster
docker-compose up -d

# 3. Wait for startup
sleep 10

# 4. Run 3 rounds
./testnet/scripts/run_rounds.sh

# 5. Verify metrics
curl -s http://localhost:9001/metrics | grep anna_

# 6. Check convergence
for port in 8001 8002 8003; do
  curl -s http://localhost:$port/rpc/status | jq '.rounds'
done

# 7. Stop cluster
docker-compose down
```

## Performance Benchmarks

**Phase 1.10 Baseline** (3-node testnet, localhost):
- Round completion: ~100-200ms
- Peer request latency: ~5-10ms
- Migration time: ~50-100ms
- State file size: ~5-10 KB (v2)

## Security Considerations

### Current Security Model
- ✅ Ed25519 signatures on observations
- ✅ Byzantine detection (double-submit)
- ✅ State backup with checksum verification
- ✅ Audit logging for migrations
- ⏸️ TLS/mTLS (foundation ready, Phase 1.11)
- ⏸️ Rate limiting (Phase 1.11)
- ⏸️ Peer authentication beyond signatures (Phase 1.11)

### Advisory-Only Enforcement
All consensus outputs remain **advisory-only**. Conscience sovereignty preserved.

## Limitations (Phase 1.10)

- ❌ **No TLS**: HTTP only (foundation ready)
- ❌ **No Hot Reload**: Requires daemon restart
- ❌ **No Auto-Reconnect**: Connection errors are fatal
- ❌ **No CI Integration**: Manual testing only
- ❌ **No Load Testing**: Performance baselines informal

## Next Steps (Phase 1.11)

1. **TLS/mTLS Implementation**: Enable encrypted peer communication
2. **SIGHUP Hot Reload**: Implement signal handling
3. **Auto-Reconnect**: Exponential backoff with jitter
4. **Timeouts and Limits**: Request size limits, idempotency keys
5. **CI Smoke Tests**: GitHub Actions workflow
6. **Load Testing**: Benchmark multi-node performance

## References

- Phase 1.9 Network Foundation: `docs/phase_1_9_networked_consensus.md`
- Phase 1.8 PoC: `docs/consensus_poc_user_guide.md`
- State v2 Schema: `crates/annad/src/state/v2.rs`
- Migration Logic: `crates/annad/src/state/migrate.rs`
- Extended Metrics: `crates/annad/src/network/metrics.rs`

---

**Document Version**: 1.0
**Last Updated**: 2025-11-12
**Phase**: 1.10.0-alpha.1 (Operational Robustness)
