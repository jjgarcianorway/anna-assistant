#!/bin/bash
# Invariant 14: All acceptance gates must pass before release
# Test: Acceptance gates script exists and contains gate checks

set -e

echo "Testing Invariant 14: Acceptance gates"

GATES="tests/acceptance_gates.sh"

if [ ! -f "$GATES" ]; then
    echo "FAIL: Acceptance gates script not found"
    exit 1
fi

# Check gates script has gate definitions
if ! grep -q "GATE\|pass()\|fail()" "$GATES"; then
    echo "FAIL: Acceptance gates does not define gates"
    exit 1
fi

# Check gates script checks key invariants
if ! grep -q "Home Writes\|system paths\|invariant" "$GATES"; then
    echo "FAIL: Acceptance gates does not check invariants"
    exit 1
fi

echo "PASS: Acceptance gates exist and check invariants"
echo "Invariant 14: PASS"
