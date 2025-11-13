# Phase 1.7: Distributed Consensus Test Plan

**Version**: 1.7.0-alpha.1
**Status**: Design Document - Test Harness Skeletons Only
**Date**: 2025-11-12

---

## Overview

This document defines the test plan for Phase 1.7 distributed consensus. Since Phase 1.7.0-alpha.1 contains only stubs and type definitions, this test plan focuses on:

1. **Fixture design** for future testing
2. **Test scenario specifications**
3. **Validation criteria** for each scenario
4. **Harness scaffolding** (no live networking)

---

## Test Scenarios

### Scenario 1: Healthy Quorum (3/3 Nodes)

**Description**: All three nodes are online, communicating, and submit valid observations for consensus.

**Setup**:
- 3 Anna nodes running in Docker containers
- Static peer configuration (testnet/peers.yml)
- All nodes have valid Ed25519 keypairs
- Network fully connected (no partitions)

**Input Fixture** (`testnet/fixtures/scenario_1_observations.json`):
```json
{
  "scenario": "healthy_quorum",
  "nodes": 3,
  "observations": [
    {
      "node_id": "node_aabbccddeeff0011",
      "audit_id": "audit_001",
      "round_id": "round_123",
      "window_hours": 24,
      "timestamp": "2025-11-12T12:00:00Z",
      "tis_overall": 0.85,
      "bias_flags": []
    },
    {
      "node_id": "node_1122334455667788",
      "audit_id": "audit_002",
      "round_id": "round_123",
      "window_hours": 24,
      "timestamp": "2025-11-12T12:00:15Z",
      "tis_overall": 0.82,
      "bias_flags": []
    },
    {
      "node_id": "node_99aabbccddeeff00",
      "audit_id": "audit_003",
      "round_id": "round_123",
      "window_hours": 24,
      "timestamp": "2025-11-12T12:00:30Z",
      "tis_overall": 0.88,
      "bias_flags": []
    }
  ]
}
```

**Expected Outcome**:
- Quorum threshold: 2 nodes
- Quorum reached: YES (3/3 observations received)
- Consensus TIS: 0.85 (median of [0.82, 0.85, 0.88])
- Round status: Complete
- Byzantine nodes detected: 0

**Validation Harness** (`testnet/scripts/test_scenario_1.sh`):
```bash
#!/bin/bash
# TODO Phase 1.8: Implement actual test
echo "STUB: Scenario 1 - Healthy Quorum"
echo "Expected: Consensus TIS = 0.85, Round Complete, No Byzantine"
exit 0  # Placeholder success
```

---

### Scenario 2: Slow Node (1/3 Node Delayed)

**Description**: Node 3 is slow to submit observation but eventually participates. Consensus waits for quorum.

**Setup**:
- 3 Anna nodes running
- Node 1 and 2 submit immediately
- Node 3 delayed by 60 seconds

**Input Fixture** (`testnet/fixtures/scenario_2_observations.json`):
```json
{
  "scenario": "slow_node",
  "nodes": 3,
  "observations": [
    {
      "node_id": "node_aabbccddeeff0011",
      "audit_id": "audit_010",
      "round_id": "round_456",
      "timestamp": "2025-11-12T12:00:00Z",
      "tis_overall": 0.80
    },
    {
      "node_id": "node_1122334455667788",
      "audit_id": "audit_011",
      "round_id": "round_456",
      "timestamp": "2025-11-12T12:00:05Z",
      "tis_overall": 0.82
    },
    {
      "node_id": "node_99aabbccddeeff00",
      "audit_id": "audit_012",
      "round_id": "round_456",
      "timestamp": "2025-11-12T12:01:05Z",
      "tis_overall": 0.84,
      "delay_seconds": 60
    }
  ]
}
```

**Expected Outcome**:
- At T+10s: Round status = Pending (2/3 observations, below quorum)
- At T+70s: Round status = Complete (3/3 observations, quorum reached)
- Consensus TIS: 0.82 (median)
- No timeout (round completes before 300s limit)

**Validation Harness** (`testnet/scripts/test_scenario_2.sh`):
```bash
#!/bin/bash
# TODO Phase 1.8: Implement timing validation
echo "STUB: Scenario 2 - Slow Node"
echo "Expected: Pending -> Complete after 60s, No timeout"
exit 0
```

---

### Scenario 3: Byzantine Node (1/3 Node Misbehaving)

**Description**: Node 2 sends conflicting observations for the same round, triggering Byzantine detection.

**Setup**:
- 3 Anna nodes running
- Node 2 submits two observations with different TIS for same round

**Input Fixture** (`testnet/fixtures/scenario_3_observations.json`):
```json
{
  "scenario": "byzantine_node",
  "nodes": 3,
  "observations": [
    {
      "node_id": "node_aabbccddeeff0011",
      "audit_id": "audit_020",
      "round_id": "round_789",
      "tis_overall": 0.85
    },
    {
      "node_id": "node_1122334455667788",
      "audit_id": "audit_021",
      "round_id": "round_789",
      "tis_overall": 0.90,
      "comment": "First submission"
    },
    {
      "node_id": "node_1122334455667788",
      "audit_id": "audit_021_conflicting",
      "round_id": "round_789",
      "tis_overall": 0.50,
      "comment": "Conflicting submission - Byzantine"
    },
    {
      "node_id": "node_99aabbccddeeff00",
      "audit_id": "audit_022",
      "round_id": "round_789",
      "tis_overall": 0.88
    }
  ]
}
```

**Expected Outcome**:
- Byzantine detection triggered: node_1122334455667788
- Reason: ConflictingObservations
- Node excluded from quorum calculations
- Effective quorum: 2/3 → 2/2 (after exclusion)
- Consensus TIS: 0.865 (median of [0.85, 0.88], excluding Byzantine)
- Round status: Complete
- Byzantine node logged to `/var/log/anna/consensus.log`

**Validation Harness** (`testnet/scripts/test_scenario_3.sh`):
```bash
#!/bin/bash
# TODO Phase 1.8: Implement Byzantine detection validation
echo "STUB: Scenario 3 - Byzantine Node"
echo "Expected: Node 2 excluded, Consensus from nodes 1 and 3 only"
exit 0
```

---

### Scenario 4: Network Partition Healing

**Description**: Network splits into [Node 1, Node 2] and [Node 3] partitions, then heals. Consensus resumes after healing.

**Setup**:
- 3 Anna nodes running
- Network partition created at T+0s (node 3 isolated)
- Partition healed at T+120s

**Input Fixture** (`testnet/fixtures/scenario_4_observations.json`):
```json
{
  "scenario": "network_partition_healing",
  "nodes": 3,
  "partition_duration_seconds": 120,
  "observations": [
    {
      "node_id": "node_aabbccddeeff0011",
      "audit_id": "audit_030",
      "round_id": "round_abc",
      "timestamp": "2025-11-12T12:00:00Z",
      "tis_overall": 0.80,
      "partition": "A"
    },
    {
      "node_id": "node_1122334455667788",
      "audit_id": "audit_031",
      "round_id": "round_abc",
      "timestamp": "2025-11-12T12:00:05Z",
      "tis_overall": 0.82,
      "partition": "A"
    },
    {
      "node_id": "node_99aabbccddeeff00",
      "audit_id": "audit_032",
      "round_id": "round_abc",
      "timestamp": "2025-11-12T12:00:10Z",
      "tis_overall": 0.84,
      "partition": "B",
      "isolated": true
    }
  ]
}
```

**Expected Outcome**:
- At T+60s (during partition):
  - Partition A (nodes 1+2): Quorum reached (2/2), consensus TIS = 0.81
  - Partition B (node 3): Pending (1/1, no quorum)
- At T+130s (after healing):
  - Node 3 observation propagates
  - Network reconciles state
  - All nodes agree on consensus TIS = 0.82 (median of all 3)
- Round status: Complete (after healing)

**Validation Harness** (`testnet/scripts/test_scenario_4.sh`):
```bash
#!/bin/bash
# TODO Phase 1.8: Implement partition testing
echo "STUB: Scenario 4 - Network Partition Healing"
echo "Expected: Partition A reaches consensus, heals, reconciles with B"
exit 0
```

---

## Fixture Files

All fixtures located in `testnet/fixtures/`:

### observation_healthy.json
Valid observation for baseline testing.

### observation_byzantine.json
Conflicting observation for Byzantine detection testing.

### consensus_result_expected.json
Expected consensus output structure.

### peers_3node.yml
Static peer configuration for 3-node cluster (copied from testnet/peers.yml).

---

## Test Harness Structure

### Phase 1.7.0-alpha.1 (Current)

**Status**: Scaffolding only, no live tests

**Deliverables**:
- Fixture file templates (JSON schemas defined)
- Test scenario specifications (this document)
- Harness scripts (stubs returning exit 0)

**Non-Deliverables**:
- Actual network testing
- Byzantine injection
- Timing validation
- Automated CI integration

### Phase 1.8 (Future)

**Status**: Full implementation

**Deliverables**:
- Live consensus networking
- Signature verification
- Byzantine detection logic
- Automated test harness execution
- CI integration (GitHub Actions)

---

## Validation Criteria

### Acceptance Checklist

- [ ] All 4 scenario fixtures created
- [ ] All 4 harness scripts created (stubs)
- [ ] Testnet docker-compose starts successfully
- [ ] Manual observation submission via annactl works
- [ ] Consensus status query returns stub data
- [ ] No runtime errors in stub code

### Success Metrics (Phase 1.8)

- Scenario 1: 100% consensus reached
- Scenario 2: < 5% timeout rate
- Scenario 3: 100% Byzantine detection rate
- Scenario 4: < 10s reconciliation time after healing

---

## Running Tests

### Phase 1.7.0-alpha.1 (Stubs)

```bash
# Start testnet
docker-compose up -d

# Run stub harnesses
bash testnet/scripts/test_scenario_1.sh
bash testnet/scripts/test_scenario_2.sh
bash testnet/scripts/test_scenario_3.sh
bash testnet/scripts/test_scenario_4.sh

# All exit 0 (stubs)
```

### Phase 1.8 (Live Tests)

```bash
# Automated test suite
make test-consensus

# Expected: 4/4 scenarios pass
```

---

## CI Integration Plan

**GitHub Actions Workflow** (`consensus-test.yml`):

```yaml
name: Consensus Tests
on: [push, pull_request]
jobs:
  consensus-testnet:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build testnet
        run: docker-compose build
      - name: Run scenarios
        run: make test-consensus
      - name: Upload logs
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: testnet-logs
          path: testnet/logs/
```

---

## Citations

- [archwiki:System_maintenance](https://wiki.archlinux.org/title/System_maintenance)
- [archwiki:Docker](https://wiki.archlinux.org/title/Docker)
- Phase 1.7 Design Document: `docs/phase_1_7_distributed_consensus.md`

---

**Status**: Phase 1.7.0-alpha.1 - Design complete, harnesses are stubs only
