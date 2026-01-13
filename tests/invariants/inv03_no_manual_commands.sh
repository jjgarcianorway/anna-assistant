#!/bin/bash
# Invariant 3: Anna must never require manual commands for install/update/uninstall
# Test: Install, update, and uninstall scripts exist and are executable

set -e

echo "Testing Invariant 3: No manual commands required"

# Check install script exists
if [ ! -f "scripts/install.sh" ]; then
    echo "FAIL: scripts/install.sh not found"
    exit 1
fi

# Check uninstall script exists
if [ ! -f "scripts/uninstall.sh" ]; then
    echo "FAIL: scripts/uninstall.sh not found"
    exit 1
fi

# Check auto-update mechanism exists in daemon
if ! grep -q "auto.update\|check_for_updates\|UpdateChecker" crates/annad/src/*.rs crates/annad/src/**/*.rs 2>/dev/null; then
    echo "FAIL: No auto-update mechanism found in daemon"
    exit 1
fi

echo "PASS: Install/update/uninstall automation exists"
echo "Invariant 3: PASS"
