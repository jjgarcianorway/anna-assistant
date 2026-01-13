#!/bin/bash
# Invariant 4: All state must reside in system paths only
# Test: Paths module uses /etc/anna, /var/lib/anna, /run/anna

set -e

echo "Testing Invariant 4: System paths only"

PATHS_FILE="crates/anna-shared/src/paths.rs"

if [ ! -f "$PATHS_FILE" ]; then
    echo "FAIL: paths.rs not found"
    exit 1
fi

# Check for system paths
if ! grep -q '/etc/anna' "$PATHS_FILE"; then
    echo "FAIL: /etc/anna not found in paths.rs"
    exit 1
fi

if ! grep -q '/var/lib/anna' "$PATHS_FILE"; then
    echo "FAIL: /var/lib/anna not found in paths.rs"
    exit 1
fi

if ! grep -q '/run/anna' "$PATHS_FILE"; then
    echo "FAIL: /run/anna not found in paths.rs"
    exit 1
fi

echo "PASS: System paths are canonical"
echo "Invariant 4: PASS"
