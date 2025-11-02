#!/usr/bin/env bash
# Anna v0.12.3 - Hardware Profile Smoke Test
# Validates hardware profile collection and JSON schema

set -euo pipefail

echo "=== Hardware Profile Smoke Test ==="
echo

# Check that annactl is available
if ! command -v annactl &>/dev/null; then
    echo "✗ FAIL: annactl not found in PATH"
    exit 1
fi

# Check that daemon is running
if ! systemctl is-active annad &>/dev/null; then
    echo "✗ FAIL: annad daemon is not running"
    exit 1
fi

START_TIME=$(date +%s)

# Test 1: annactl hw show (human output)
echo "Test 1: annactl hw show (human output)"
if annactl hw show; then
    echo "✓ PASS: Human output succeeds"
else
    echo "✗ FAIL: Human output failed"
    exit 1
fi
echo

# Test 2: annactl hw show --json
echo "Test 2: annactl hw show --json (JSON output)"
if ! HW_JSON=$(annactl hw show --json 2>&1); then
    echo "✗ FAIL: JSON output failed"
    echo "$HW_JSON"
    exit 1
fi

# Validate JSON syntax
if ! echo "$HW_JSON" | jq . >/dev/null 2>&1; then
    echo "✗ FAIL: Invalid JSON output"
    echo "$HW_JSON"
    exit 1
fi
echo "✓ PASS: JSON output is valid JSON"
echo

# Test 3: Validate required fields
echo "Test 3: Validate required JSON fields"
REQUIRED_KEYS=(
    "version"
    "generated_at"
    "kernel"
    "cpu"
    "gpus"
    "storage"
    "storage.block_devices"
    "battery"
    "memory"
)

for key in "${REQUIRED_KEYS[@]}"; do
    if ! echo "$HW_JSON" | jq -e "has(\"${key%%.*}\")" >/dev/null 2>&1; then
        echo "✗ FAIL: Missing required key: $key"
        exit 1
    fi
done
echo "✓ PASS: All required keys present"
echo

# Test 4: Validate schema version
echo "Test 4: Validate schema version"
VERSION=$(echo "$HW_JSON" | jq -r '.version')
if [ "$VERSION" != "1" ]; then
    echo "✗ FAIL: Expected version '1', got '$VERSION'"
    exit 1
fi
echo "✓ PASS: Schema version is '1'"
echo

# Test 5: Validate timestamp format (RFC3339)
echo "Test 5: Validate timestamp format"
GENERATED_AT=$(echo "$HW_JSON" | jq -r '.generated_at')
if ! echo "$GENERATED_AT" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}'; then
    echo "✗ FAIL: generated_at is not RFC3339 format: $GENERATED_AT"
    exit 1
fi
echo "✓ PASS: Timestamp is RFC3339 format"
echo

# Test 6: Validate kernel field is non-empty
echo "Test 6: Validate kernel field"
KERNEL=$(echo "$HW_JSON" | jq -r '.kernel')
if [ -z "$KERNEL" ] || [ "$KERNEL" = "null" ]; then
    echo "✗ FAIL: kernel field is empty or null"
    exit 1
fi
echo "✓ PASS: Kernel field is non-empty: $KERNEL"
echo

# Test 7: Validate data types
echo "Test 7: Validate data types"
if ! echo "$HW_JSON" | jq -e '.cpu | type == "object"' >/dev/null; then
    echo "✗ FAIL: cpu is not an object"
    exit 1
fi
if ! echo "$HW_JSON" | jq -e '.gpus | type == "array"' >/dev/null; then
    echo "✗ FAIL: gpus is not an array"
    exit 1
fi
if ! echo "$HW_JSON" | jq -e '.storage.block_devices | type == "array"' >/dev/null; then
    echo "✗ FAIL: storage.block_devices is not an array"
    exit 1
fi
if ! echo "$HW_JSON" | jq -e '.battery.present | type == "boolean"' >/dev/null; then
    echo "✗ FAIL: battery.present is not a boolean"
    exit 1
fi
echo "✓ PASS: All data types are correct"
echo

# Test 8: Test with --wide flag
echo "Test 8: annactl hw show --wide"
if annactl hw show --wide >/dev/null; then
    echo "✓ PASS: Wide output succeeds"
else
    echo "✗ FAIL: Wide output failed"
    exit 1
fi
echo

# Test 9: Performance check (should complete within 5 seconds)
echo "Test 9: Performance check"
PERF_START=$(date +%s%3N)
annactl hw show --json >/dev/null
PERF_END=$(date +%s%3N)
PERF_DURATION=$((PERF_END - PERF_START))
if [ "$PERF_DURATION" -gt 5000 ]; then
    echo "⚠ WARN: Collection took ${PERF_DURATION}ms (>5000ms)"
else
    echo "✓ PASS: Collection took ${PERF_DURATION}ms"
fi
echo

# Calculate total time
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo "==================================="
echo "✓ ALL TESTS PASSED (${TOTAL_TIME}s)"
echo "==================================="
echo
echo "Sample output (first 40 lines):"
echo "$HW_JSON" | jq . | head -40
