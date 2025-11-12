# Anna Assistant - TLS-Pinned Testnet

**Phase 2.0.0-alpha.1** - Certificate pinning testing environment

Citation: [docker:compose-networking][tls:certificate-pinning]

## Overview

This testnet provides a complete environment for testing TLS certificate pinning with 3 Anna nodes, Prometheus monitoring, and Grafana dashboards.

### Architecture

```
┌─────────────────────────────────────────────────┐
│  Docker Network: 172.20.0.0/16 (anna_testnet)  │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌────────┐  ┌────────┐  ┌────────┐           │
│  │ Node 1 │──│ Node 2 │──│ Node 3 │           │
│  │:8080   │  │:8081   │  │:8082   │           │
│  └────┬───┘  └────┬───┘  └────┬───┘           │
│       │           │           │                 │
│       └───────────┴───────────┘                 │
│                   │                             │
│            ┌──────▼──────┐                      │
│            │ Prometheus  │                      │
│            │    :9093    │                      │
│            └──────┬──────┘                      │
│                   │                             │
│            ┌──────▼──────┐                      │
│            │   Grafana   │                      │
│            │    :3000    │                      │
│            └─────────────┘                      │
└─────────────────────────────────────────────────┘
```

### Components

1. **Node 1** (Bootstrap):
   - IP: 172.20.0.10
   - Ports: 8080 (API), 9090 (Metrics)
   - Role: Primary bootstrap node

2. **Node 2** (Secondary):
   - IP: 172.20.0.11
   - Ports: 8081 (API), 9091 (Metrics)
   - Role: Secondary node with pinned certificate
   - Connects to: Node 1

3. **Node 3** (Test):
   - IP: 172.20.0.12
   - Ports: 8082 (API), 9092 (Metrics)
   - Role: Test node for MITM simulation
   - Connects to: Node 1, Node 2

4. **Prometheus**:
   - Port: 9093
   - Scrapes all nodes every 15s
   - Loads alert rules

5. **Grafana**:
   - Port: 3000
   - Credentials: admin/admin
   - Pre-loaded dashboards from observability/grafana/

## Quick Start

### Prerequisites

- Docker & Docker Compose
- OpenSSL (for certificate generation)
- curl (for testing)

### Setup

```bash
# 1. Generate certificates
cd testnet
./scripts/setup-certs.sh

# 2. Start testnet
docker-compose -f docker-compose.pinned.yml up -d

# 3. Wait for nodes to be ready (30-60 seconds for first build)
docker-compose -f docker-compose.pinned.yml logs -f

# 4. Run tests
./scripts/run-tls-test.sh
```

### Access Points

- **Node 1 Metrics**: http://localhost:9090/metrics
- **Node 2 Metrics**: http://localhost:9091/metrics
- **Node 3 Metrics**: http://localhost:9092/metrics
- **Prometheus**: http://localhost:9093
- **Grafana**: http://localhost:3000 (admin/admin)

### Shutdown

```bash
docker-compose -f testnet/docker-compose.pinned.yml down

# Remove volumes (clean slate)
docker-compose -f testnet/docker-compose.pinned.yml down -v
```

## Testing Scenarios

### 1. Normal Operation

```bash
# Start testnet
docker-compose -f docker-compose.pinned.yml up -d

# Check all nodes are connected
for port in 9090 9091 9092; do
    echo "Node ${port}:"
    curl -s http://localhost:${port}/metrics | grep anna_tls_handshakes_total
done

# Verify no pinning violations
for port in 9090 9091 9092; do
    curl -s http://localhost:${port}/metrics | grep anna_pinning_violations_total
done
```

**Expected**: All nodes connect successfully, zero pinning violations.

### 2. Certificate Rotation

```bash
# Generate new certificate for Node 3
cd testnet/certs/node3
openssl req -new -nodes \
    -out cert.csr \
    -keyout key.pem \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Testnet/OU=Nodes/CN=node3.anna.local"

openssl x509 -req -days 365 \
    -in cert.csr \
    -CA ../ca/ca-cert.pem \
    -CAkey ../ca/ca-key.pem \
    -CAcreateserial \
    -out cert.pem

# Restart Node 3
docker-compose -f ../../docker-compose.pinned.yml restart node3

# Check if pinning violation detected
curl -s http://localhost:9092/metrics | grep anna_pinning_violations_total
```

**Expected**: Pinning violations increase if new cert fingerprint not in pinning.toml.

### 3. MITM Simulation

```bash
# Generate attacker certificate (different CA)
openssl req -new -x509 -days 365 -nodes \
    -out testnet/certs/attacker/cert.pem \
    -keyout testnet/certs/attacker/key.pem \
    -subj "/C=FAKE/ST=Fake/L=Fake/O=Attacker/CN=node3.anna.local"

# Replace Node 3 certificate
cp testnet/certs/attacker/cert.pem testnet/certs/node3/cert.pem
cp testnet/certs/attacker/key.pem testnet/certs/node3/key.pem

# Restart Node 3
docker-compose -f docker-compose.pinned.yml restart node3

# Check metrics
curl -s http://localhost:9090/metrics | grep anna_pinning_violations
curl -s http://localhost:9091/metrics | grep anna_pinning_violations
```

**Expected**: Nodes 1 and 2 detect pinning violation when connecting to Node 3.

### 4. Network Partition

```bash
# Disconnect Node 3 from network
docker network disconnect anna_testnet anna-node3

# Wait 30 seconds
sleep 30

# Check peer backoff metrics
curl -s http://localhost:9091/metrics | grep anna_peer_backoff_seconds

# Reconnect
docker network connect anna_testnet anna-node3
```

**Expected**: Backoff metrics show increased duration, nodes reconnect after network restored.

## Metrics to Monitor

### TLS Handshakes
```promql
rate(anna_tls_handshakes_total[5m])
```

### Pinning Violations
```promql
increase(anna_pinning_violations_total[5m])
```

### Peer Backoff
```promql
histogram_quantile(0.95, rate(anna_peer_backoff_seconds_bucket[5m]))
```

### Consensus Rounds
```promql
rate(anna_consensus_rounds_total[5m])
```

## Prometheus Queries

Access Prometheus at http://localhost:9093 and try these queries:

```promql
# TLS handshake success rate
rate(anna_tls_handshakes_total{status="success"}[5m])

# Pinning violations by peer
sum by (peer) (anna_pinning_violations_total)

# Network coherence
anna_average_tis

# Byzantine nodes detected
anna_byzantine_nodes_total
```

## Grafana Dashboards

1. Navigate to http://localhost:3000
2. Login: admin/admin
3. Dashboards → Browse
4. Select:
   - **Anna Overview** - System health
   - **Anna TLS** - Certificate pinning monitoring
   - **Anna Consensus** - Detailed consensus metrics

## Troubleshooting

### Nodes Not Starting

```bash
# Check logs
docker-compose -f docker-compose.pinned.yml logs node1

# Check if ports are in use
netstat -tulpn | grep -E '8080|9090|3000'

# Rebuild containers
docker-compose -f docker-compose.pinned.yml build --no-cache
```

### Certificate Errors

```bash
# Verify certificate validity
openssl x509 -in testnet/certs/node1/cert.pem -text -noout

# Check certificate matches private key
openssl x509 -noout -modulus -in testnet/certs/node1/cert.pem | openssl md5
openssl rsa -noout -modulus -in testnet/certs/node1/key.pem | openssl md5
```

### Metrics Not Showing

```bash
# Check node is healthy
curl -v http://localhost:9090/metrics

# Check Prometheus targets
open http://localhost:9093/targets

# Check container networking
docker network inspect anna_testnet
```

## References

- [Docker Compose Networking](https://docs.docker.com/compose/networking/)
- [TLS Certificate Pinning](https://owasp.org/www-community/controls/Certificate_and_Public_Key_Pinning)
- [Prometheus Query Language](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)

---

**v1.0 - Phase 2.0.0-alpha.1**

For issues: https://github.com/jjgarcianorway/anna-assistant/issues
