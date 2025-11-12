#!/bin/bash
# Anna Assistant - TLS Pinning Test Runner
# Phase 2.0.0-alpha.1 - Test certificate pinning in testnet

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTNET_DIR="${SCRIPT_DIR}/.."

echo "Anna Assistant - TLS Pinning Test"
echo "=================================="
echo

# Check if certificates exist
if [ ! -d "${TESTNET_DIR}/certs/ca" ]; then
    echo "ERROR: Certificates not found!"
    echo "Run: ${SCRIPT_DIR}/setup-certs.sh"
    exit 1
fi

# Start testnet
echo "[1/5] Starting testnet..."
cd "${TESTNET_DIR}"
docker-compose -f docker-compose.pinned.yml up -d

echo "  ✓ Testnet started"
echo

# Wait for nodes to be ready
echo "[2/5] Waiting for nodes to be healthy..."
sleep 10

for i in {1..30}; do
    if curl -sf http://localhost:9090/metrics >/dev/null 2>&1 && \
       curl -sf http://localhost:9091/metrics >/dev/null 2>&1 && \
       curl -sf http://localhost:9092/metrics >/dev/null 2>&1; then
        echo "  ✓ All nodes healthy"
        break
    fi

    if [ $i -eq 30 ]; then
        echo "  ✗ Timeout waiting for nodes"
        docker-compose -f docker-compose.pinned.yml logs
        exit 1
    fi

    sleep 2
done
echo

# Check metrics
echo "[3/5] Checking TLS handshake metrics..."
echo "Node 1:"
curl -s http://localhost:9090/metrics | grep anna_tls_handshakes_total || echo "  No handshakes yet"
echo
echo "Node 2:"
curl -s http://localhost:9091/metrics | grep anna_tls_handshakes_total || echo "  No handshakes yet"
echo
echo "Node 3:"
curl -s http://localhost:9092/metrics | grep anna_tls_handshakes_total || echo "  No handshakes yet"
echo

# Check for pinning violations
echo "[4/5] Checking for pinning violations..."
violations=0
for port in 9090 9091 9092; do
    count=$(curl -s http://localhost:${port}/metrics | grep -c anna_pinning_violations_total || echo "0")
    violations=$((violations + count))
done

if [ $violations -gt 0 ]; then
    echo "  ⚠ Found ${violations} pinning violations"
else
    echo "  ✓ No pinning violations detected"
fi
echo

# Display logs
echo "[5/5] Recent logs:"
echo
docker-compose -f docker-compose.pinned.yml logs --tail=20

echo
echo "=================================="
echo "Test complete!"
echo
echo "Access points:"
echo "  - Node 1 metrics: http://localhost:9090/metrics"
echo "  - Node 2 metrics: http://localhost:9091/metrics"
echo "  - Node 3 metrics: http://localhost:9092/metrics"
echo "  - Prometheus: http://localhost:9093"
echo "  - Grafana: http://localhost:3000 (admin/admin)"
echo
echo "To stop testnet:"
echo "  docker-compose -f testnet/docker-compose.pinned.yml down"
echo
