#!/bin/bash
# Invariant F4: Conflicting probe results must not assert single truth
# Test: When probes disagree, output must report conflict explicitly
#
# PASS CRITERIA:
# - VerifiedResponse has conflicts_detected field
# - ClaimGate detects when probes contradict each other
# - Rust tests for conflict detection pass

set -e

echo "Testing F4: Conflicting probes must report conflict"

CLAIM_GATE_DIR="crates/anna-shared/src/claim_gate"

# Check 1: VerifiedResponse must have conflicts_detected field
if ! grep -q "conflicts_detected" "$CLAIM_GATE_DIR/verifier.rs"; then
    echo "FAIL: VerifiedResponse missing conflicts_detected field"
    exit 1
fi

# Check 2: Conflict detection logic must exist
if ! grep -q "detect_probe_conflicts" "$CLAIM_GATE_DIR/gate.rs"; then
    echo "FAIL: No conflict detection logic in ClaimGate"
    exit 1
fi

# Check 3: Rust test for conflict detection must pass
TEST_OUTPUT=$(cargo test --workspace -- test_conflicting_probes 2>&1)
if echo "$TEST_OUTPUT" | grep -q "test_conflicting_probes_block_assertion ... ok"; then
    echo "PASS: Conflict detection test passed"
else
    echo "FAIL: Conflict detection test did not pass"
    echo "$TEST_OUTPUT" | grep -E "(test.*conflict|FAILED|error)"
    exit 1
fi

echo "PASS: Conflicting probes are properly detected and reported"
echo "Invariant F4: PASS"
