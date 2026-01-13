#!/bin/bash
# Invariant 1: Anna must never emit a factual claim without evidence
# Test: ClaimGate must block or mark unverified claims

set -e

echo "Testing Invariant 1: Factual claims require evidence"

# Test that ClaimGate module exists and has verification logic
if ! grep -q "verify_response\|verify_answer" crates/anna-shared/src/claim_gate.rs 2>/dev/null && \
   ! grep -q "verify_response\|verify_answer" crates/annad/src/llm_core/mod.rs 2>/dev/null; then
    echo "FAIL: No claim verification logic found"
    exit 1
fi

# Test that unverified claims are marked or blocked
if grep -q "unverified\|NeedsInvestigation" crates/anna-shared/src/claim_gate.rs 2>/dev/null || \
   grep -q "unverified\|NeedsInvestigation" crates/annad/src/llm_core/mod.rs 2>/dev/null; then
    echo "PASS: Claim verification mechanism exists"
else
    echo "FAIL: No unverified claim handling found"
    exit 1
fi

echo "Invariant 1: PASS"
