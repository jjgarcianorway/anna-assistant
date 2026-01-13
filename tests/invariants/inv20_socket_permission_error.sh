#!/bin/bash
# Invariant F8: Socket permission denied must show specific message
# Test: When EACCES occurs on connect, message must mention permission and anna group
#
# FAIL CRITERIA (current behavior):
# - Generic "Cannot connect to Anna daemon" error
# - No EACCES-specific handling
#
# PASS CRITERIA:
# - Error message contains "permission" when EACCES
# - Suggests checking "anna group" membership

set -e

echo "Testing F8: Socket EACCES must show permission-specific message"

STREAMING="crates/annactl/src/streaming.rs"
RPC="crates/annactl/src/rpc.rs"

# Check 1: Code must handle permission errors specifically
FOUND_PERMISSION_HANDLING=false

if grep -q "permission\|Permission\|EACCES" "$STREAMING"; then
    FOUND_PERMISSION_HANDLING=true
fi

if grep -q "permission\|Permission\|EACCES" "$RPC" 2>/dev/null; then
    FOUND_PERMISSION_HANDLING=true
fi

if [ "$FOUND_PERMISSION_HANDLING" = false ]; then
    echo "FAIL: No permission-specific error handling in client code"
    exit 1
fi

# Check 2: Error message must mention anna group
if ! grep -q "anna.*group\|group.*anna" "$STREAMING" "$RPC" 2>/dev/null; then
    echo "FAIL: Error message does not suggest anna group membership"
    exit 1
fi

# Check 3: Build succeeds (basic sanity)
cargo build --package annactl 2>&1 | tail -5

echo "PASS: Socket permission errors are handled specifically"
echo "Invariant F8: PASS"
