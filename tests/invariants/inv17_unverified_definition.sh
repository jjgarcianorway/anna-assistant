#!/bin/bash
# Invariant: Unverified claim definition exists in SPEC.md (Phase 2)
# Test: SPEC.md defines unverified claims testably

set -e

echo "Testing unverified claim definition"

# Check SPEC.md exists
if [ ! -f "SPEC.md" ]; then
    echo "FAIL: SPEC.md not found"
    exit 1
fi

# Check for unverified claim section
if ! grep -q "## Unverified Claim Definition" SPEC.md; then
    echo "FAIL: Unverified Claim Definition section not found in SPEC.md"
    exit 1
fi

# Check for "BLOCKED" requirement
if ! grep -q "BLOCKED" SPEC.md; then
    echo "FAIL: SPEC.md does not specify that unverified claims must be BLOCKED"
    exit 1
fi

# Check for testability statement
if ! grep -q "testable" SPEC.md; then
    echo "FAIL: SPEC.md does not state that the definition is testable"
    exit 1
fi

# Check for the four conditions
if ! grep -q "no probe was run" SPEC.md; then
    echo "FAIL: Missing condition: no probe was run"
    exit 1
fi

if ! grep -q "probe failed" SPEC.md; then
    echo "FAIL: Missing condition: probe failed"
    exit 1
fi

if ! grep -q "probe output contradicts" SPEC.md; then
    echo "FAIL: Missing condition: probe output contradicts"
    exit 1
fi

echo "PASS: Unverified claim definition is complete and testable"
echo "Invariant 17 (Unverified Definition): PASS"
