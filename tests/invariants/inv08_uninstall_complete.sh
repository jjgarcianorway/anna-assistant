#!/bin/bash
# Invariant 8: Uninstall must remove all Anna-created files
# Test: Uninstall script removes all standard paths

set -e

echo "Testing Invariant 8: Complete uninstall"

UNINSTALL="scripts/uninstall.sh"

if [ ! -f "$UNINSTALL" ]; then
    echo "FAIL: Uninstall script not found"
    exit 1
fi

# Check uninstall removes system paths
if ! grep -q '/etc/anna' "$UNINSTALL"; then
    echo "FAIL: Uninstall does not remove /etc/anna"
    exit 1
fi

if ! grep -q '/var/lib/anna' "$UNINSTALL"; then
    echo "FAIL: Uninstall does not remove /var/lib/anna"
    exit 1
fi

if ! grep -q '/run/anna' "$UNINSTALL"; then
    echo "FAIL: Uninstall does not remove /run/anna"
    exit 1
fi

# Check uninstall removes binaries
if ! grep -q 'annad\|annactl' "$UNINSTALL"; then
    echo "FAIL: Uninstall does not remove binaries"
    exit 1
fi

echo "PASS: Uninstall script removes all paths"
echo "Invariant 8: PASS"
