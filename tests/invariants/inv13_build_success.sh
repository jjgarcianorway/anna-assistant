#!/bin/bash
# Invariant 13: Build must succeed with zero errors
# Test: cargo build --workspace succeeds

set -e

echo "Testing Invariant 13: Build succeeds"

# Run cargo build
if cargo build --workspace 2>&1 | grep -q "^error\["; then
    echo "FAIL: Build has errors"
    exit 1
fi

echo "PASS: Build succeeds"
echo "Invariant 13: PASS"
