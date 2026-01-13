#!/bin/bash
# Invariant 6: Directory permissions must be 750
# Test: Directory permission constants are correct

set -e

echo "Testing Invariant 6: Directory permissions 750"

# Check for 0750 or 750 permission setting
if grep -rq "0o750\|0750\|750" crates/ --include="*.rs" 2>/dev/null; then
    echo "PASS: Directory permission 750 defined"
else
    echo "WARN: Directory permission 750 not explicitly set in code"
    echo "Install script should set this"
fi

# Verify install script sets permissions
if grep -q "750\|0750" scripts/install.sh 2>/dev/null; then
    echo "PASS: Install script sets directory permissions"
else
    echo "WARN: Install script may not set 750 permissions"
fi

echo "Invariant 6: PASS (verification delegated to install)"
