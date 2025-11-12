# Phase 1.7 Consensus Testnet

**3-Node Local Consensus Cluster for Testing**

## Overview

This testnet provides a local 3-node Anna cluster for developing and testing distributed consensus features. All nodes run in Docker containers with static peer configuration.

## Prerequisites

- Docker and Docker Compose installed
- At least 2GB RAM available
- Ports 8081-8083 available on host

## Quick Start

```bash
# Build and start cluster
docker-compose up -d

# Check node health
docker-compose ps

# View node 1 logs
docker-compose logs -f anna-node-1

# Connect to node 1
docker exec -it anna-node-1 bash
annactl status

# Stop cluster
docker-compose down

# Stop and remove volumes (clean state)
docker-compose down -v
```

## Network Topology

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  anna-node-1     │────▶│  anna-node-2     │────▶│  anna-node-3     │
│  172.20.0.10     │     │  172.20.0.11     │     │  172.20.0.12     │
│  :8081 (host)    │◀────│  :8082 (host)    │◀────│  :8083 (host)    │
└──────────────────┘     └──────────────────┘     └──────────────────┘
```

## Test Scenarios

### 1. Healthy Quorum (3/3 nodes)

All nodes running and communicating.

```bash
# Expected: Quorum threshold = 2
# Expected: Consensus reached with all observations
docker-compose up -d
docker exec -it anna-node-1 annactl consensus status --json | jq .
```

### 2. Slow Node (1 node delayed)

One node is slow to respond but eventually participates.

```bash
# Pause node 3 for 60 seconds
docker pause anna-node-3
sleep 60
docker unpause anna-node-3

# Check if consensus still reached
docker exec -it anna-node-1 annactl consensus status
```

### 3. Byzantine Node (1 node misbehaving)

One node sends conflicting observations.

```bash
# TODO Phase 1.8: Implement Byzantine injection script
# For now: Manual observation submission with conflicting data
```

### 4. Network Partition Healing

Network split between node 1+2 and node 3, then healed.

```bash
# Create partition (node 3 isolated)
docker network disconnect anna-consensus_anna-consensus anna-node-3

# Wait 120 seconds
sleep 120

# Heal partition
docker network connect anna-consensus_anna-consensus anna-node-3

# Check consensus state after healing
docker exec -it anna-node-1 annactl consensus status
```

## Test Fixtures

Located in `testnet/fixtures/`:

- `observation_healthy.json` - Valid observation
- `observation_byzantine.json` - Conflicting observation (for Byzantine testing)
- `consensus_result_expected.json` - Expected consensus output

## Configuration

### Peers (testnet/peers.yml)

Static peer list with 3 nodes:
- node1: `node_aabbccddeeff0011` @ 172.20.0.10
- node2: `node_1122334455667788` @ 172.20.0.11
- node3: `node_99aabbccddeeff00` @ 172.20.0.12

Quorum threshold: Majority (2/3)

### Keys (testnet/keys/)

**WARNING**: Test keys only, NOT secure!

Each node has a pre-generated Ed25519 keypair:
- `node_id.pub` - Public key (hex-encoded)
- `node_id.sec` - Secret key (hex-encoded)

## Monitoring

### View Consensus State

```bash
# From host
docker exec -it anna-node-1 annactl consensus status --json | jq .

# Inside container
docker exec -it anna-node-1 bash
cat /var/lib/anna/mirror_audit/state.json | jq '.consensus_rounds'
```

### Check Logs

```bash
# All nodes
docker-compose logs -f

# Specific node
docker-compose logs -f anna-node-2

# Consensus logs only
docker exec -it anna-node-1 tail -f /var/log/anna/consensus.log
```

## Troubleshooting

### Nodes won't start

```bash
# Check logs
docker-compose logs

# Rebuild images
docker-compose build --no-cache
docker-compose up -d
```

### Consensus not reaching quorum

```bash
# Check peer connectivity
docker exec -it anna-node-1 annactl consensus status

# Verify network
docker network inspect anna-consensus_anna-consensus

# Check state files
docker exec -it anna-node-1 cat /var/lib/anna/mirror_audit/state.json | jq .
```

### Clean restart

```bash
# Remove all data
docker-compose down -v

# Rebuild and restart
docker-compose up --build -d
```

## Development

### Modify consensus logic

1. Edit consensus module: `crates/annad/src/consensus/`
2. Rebuild: `docker-compose build`
3. Restart: `docker-compose up -d`

### Add new test scenario

1. Create fixture in `testnet/fixtures/`
2. Document scenario in this README
3. Add validation script to `testnet/scripts/`

## CI Integration

```bash
# Automated test run
make test-consensus-cluster

# Expected: All scenarios pass, exit code 0
```

## Citations

- [archwiki:Docker](https://wiki.archlinux.org/title/Docker)
- [archwiki:System_maintenance](https://wiki.archlinux.org/title/System_maintenance)

---

**Status**: Phase 1.7.0-alpha.1 (STUB - consensus logic not implemented)
