#!/bin/bash
# Invariant F15: Empty probe output on SUCCESS must be explicitly handled
# Test: Probe SUCCESS (exit 0) with empty stdout must be detected and marked
#
# PASS CRITERIA:
# - EvidenceType.ProbeResult has output_empty field
# - evidence_from_probe sets output_empty when output is empty
# - Rust tests for empty output detection pass

set -e

echo "Testing F15: Empty probe output (success) must be explicitly handled"

CLAIM_GATE_DIR="crates/anna-shared/src/claim_gate"

# Check 1: EvidenceType::ProbeResult must have output_empty field
if ! grep -q "output_empty" "$CLAIM_GATE_DIR/types.rs"; then
    echo "FAIL: EvidenceType::ProbeResult missing output_empty field"
    exit 1
fi

# Check 2: evidence_from_probe must detect empty output
if ! grep -q "output_empty" "$CLAIM_GATE_DIR/gate.rs"; then
    echo "FAIL: evidence_from_probe does not detect empty output"
    exit 1
fi

# Check 3: Rust test for empty SUCCESS output exists and passes
TEST_OUTPUT=$(cargo test --workspace -- test_empty_success_output 2>&1)
if echo "$TEST_OUTPUT" | grep -q "test_empty_success_output ... ok"; then
    echo "PASS: Empty output detection test passed"
else
    echo "FAIL: Empty output detection test did not pass"
    echo "$TEST_OUTPUT" | grep -E "(test.*empty|FAILED|error)"
    exit 1
fi

echo "PASS: Empty probe output (success) is handled"
echo "Invariant F15: PASS"
