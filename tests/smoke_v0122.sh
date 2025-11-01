#!/bin/bash
# Anna v0.12.2 Smoke Test
# Quick end-to-end test: status, collect, classify, radar show

set -euo pipefail

echo "=== Anna v0.12.2 Smoke Test ==="
echo

# Check if daemon is running
if ! systemctl is-active --quiet annad; then
    echo "ERROR: annad is not running"
    echo "Try: sudo systemctl start annad"
    exit 1
fi

# Test 1: annactl status
echo "Test 1: annactl status"
if annactl status >/dev/null 2>&1; then
    echo "  ✓ status works"
else
    echo "  ✗ status failed"
    exit 1
fi

# Test 2: annactl collect --limit 1 --json
echo "Test 2: annactl collect --limit 1 --json"
if annactl collect --limit 1 --json | jq '.snapshots[0]' >/dev/null 2>&1; then
    echo "  ✓ collect works and returns valid JSON"
else
    echo "  ✗ collect failed or invalid JSON"
    exit 1
fi

# Test 3: annactl classify --json
echo "Test 3: annactl classify --json"
if annactl classify --json | jq '.persona' >/dev/null 2>&1; then
    echo "  ✓ classify works and returns valid JSON"
else
    echo "  ✗ classify failed or invalid JSON"
    exit 1
fi

# Test 4: annactl radar show --json
echo "Test 4: annactl radar show --json"
if annactl radar show --json | jq '.overall' >/dev/null 2>&1; then
    echo "  ✓ radar show works and returns valid JSON"
else
    echo "  ✗ radar show failed or invalid JSON"
    exit 1
fi

# Test 5: Human output (no --json)
echo "Test 5: Human output tests"
if annactl collect --limit 1 | grep -q "Telemetry Snapshots"; then
    echo "  ✓ collect human output works"
else
    echo "  ✗ collect human output failed"
    exit 1
fi

if annactl classify | grep -q "System Classification"; then
    echo "  ✓ classify human output works"
else
    echo "  ✗ classify human output failed"
    exit 1
fi

if annactl radar show | grep -q "Radar Scores"; then
    echo "  ✓ radar show human output works"
else
    echo "  ✗ radar show human output failed"
    exit 1
fi

echo
echo "=== All smoke tests passed! ==="
echo

exit 0
