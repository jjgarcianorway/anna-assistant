#!/bin/bash
# Invariant: ClaimGate BLOCKS unverified factual claims (Phase 2)
# Test: Rust unit tests prove blocking works

set -e

echo "Testing ClaimGate blocking enforcement"

# Run ClaimGate-specific tests
cargo test --workspace -- claim_gate 2>&1 | tee /tmp/claimgate_test.log

# Verify key adversarial tests passed
if ! grep -q "test_probe_failure_blocks_claims ... ok" /tmp/claimgate_test.log; then
    echo "FAIL: test_probe_failure_blocks_claims did not pass"
    exit 1
fi

if ! grep -q "test_no_evidence_explicit_uncertainty ... ok" /tmp/claimgate_test.log; then
    echo "FAIL: test_no_evidence_explicit_uncertainty did not pass"
    exit 1
fi

if ! grep -q "test_conflicting_probes_block_assertion ... ok" /tmp/claimgate_test.log; then
    echo "FAIL: test_conflicting_probes_block_assertion did not pass"
    exit 1
fi

if ! grep -q "test_fact_without_evidence_blocked ... ok" /tmp/claimgate_test.log; then
    echo "FAIL: test_fact_without_evidence_blocked did not pass"
    exit 1
fi

echo "PASS: All ClaimGate adversarial tests passed"
echo "Invariant 15 (ClaimGate Blocking): PASS"
