#!/bin/bash
# Invariant 12: Deployment must be via auto-update or installer script only
# Test: No manual deployment instructions in docs

set -e

echo "Testing Invariant 12: Automated deployment only"

# Check install script exists
if [ ! -f "scripts/install.sh" ]; then
    echo "FAIL: No install script"
    exit 1
fi

# Check README does not have manual cp commands for deployment
if grep -q "sudo cp.*annad\|sudo cp.*annactl" README.md 2>/dev/null; then
    echo "FAIL: README contains manual deployment instructions"
    exit 1
fi

# Check CLAUDE.md prohibits manual deployment
if grep -q "NEVER deploy via.*sudo cp\|NEVER.*manual.*install" CLAUDE.md 2>/dev/null; then
    echo "PASS: CLAUDE.md prohibits manual deployment"
else
    echo "WARN: CLAUDE.md should explicitly prohibit manual deployment"
fi

echo "Invariant 12: PASS"
