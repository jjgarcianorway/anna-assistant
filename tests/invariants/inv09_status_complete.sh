#!/bin/bash
# Invariant 9: Status command must report daemon health, version, update state
# Test: Status module contains required fields

set -e

echo "Testing Invariant 9: Status command completeness"

STATUS_DIR="crates/anna-shared/src/status"

if [ ! -d "$STATUS_DIR" ]; then
    echo "FAIL: status module directory not found"
    exit 1
fi

# Check for health/state field in daemon_status.rs or types.rs
if ! grep -q "state\|healthy\|DaemonState" "$STATUS_DIR"/*.rs; then
    echo "FAIL: No health/state indicator in status module"
    exit 1
fi

# Check for version field
if ! grep -q "version" "$STATUS_DIR"/*.rs; then
    echo "FAIL: No version in status module"
    exit 1
fi

# Check for update state field
if ! grep -q "update_state\|UpdateCheckState" "$STATUS_DIR"/*.rs; then
    echo "FAIL: No update state in status module"
    exit 1
fi

# Verify DaemonStatus struct exists
if ! grep -q "pub struct DaemonStatus" "$STATUS_DIR"/*.rs; then
    echo "FAIL: DaemonStatus struct not found"
    exit 1
fi

echo "PASS: Status module contains required fields"
echo "Invariant 9: PASS"
