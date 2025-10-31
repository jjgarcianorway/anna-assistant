#!/usr/bin/env bash
# Anna v0.11.0 - CI Smoke Test
# Runs install, smoke tests, and annactl matrix, then prints summary

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Anna v0.11.0 CI Smoke Test             │"
echo "╰─────────────────────────────────────────╯"
echo ""

PASS=0
FAIL=0

# Test 1: Installation
echo "→ Running installation..."
if ./scripts/install.sh; then
    echo "✓ Installation passed"
    ((PASS++))
else
    echo "✗ Installation failed"
    ((FAIL++))
    exit 1
fi
echo ""

# Wait for daemon to stabilize
sleep 5

# Test 2: Smoke test
echo "→ Running smoke test..."
if ./tests/smoke_v011.sh; then
    echo "✓ Smoke test passed"
    ((PASS++))
else
    echo "✗ Smoke test failed"
    ((FAIL++))
fi
echo ""

# Test 3: annactl matrix
echo "→ Running annactl matrix..."
if ./tests/annactl_matrix.sh; then
    echo "✓ annactl matrix passed"
    ((PASS++))
else
    echo "✗ annactl matrix failed"
    ((FAIL++))
fi
echo ""

# Summary
echo "═════════════════════════════════════════"
echo "CI SMOKE TEST SUMMARY"
echo "═════════════════════════════════════════"
echo "  Passed:  $PASS/3"
echo "  Failed:  $FAIL/3"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✓ PASS - All CI tests passed"
    exit 0
else
    echo "✗ FAIL - Some CI tests failed"
    exit 1
fi
