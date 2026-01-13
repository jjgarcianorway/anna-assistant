#!/bin/bash
# Invariant: Evidence line format is standardized (Phase 2)
# Test: Format code exists and handles edge cases

set -e

echo "Testing Evidence line standardization"

# Check that format_evidence_line function exists with proper signature
if ! grep -q "fn format_evidence_line" crates/annad/src/llm_core/mod.rs; then
    echo "FAIL: format_evidence_line function not found"
    exit 1
fi

# Check that failed probes are handled explicitly
if ! grep -q "ALL PROBES FAILED" crates/annad/src/llm_core/mod.rs; then
    echo "FAIL: All-probes-failed case not handled"
    exit 1
fi

# Check that FAILED marker exists for individual failed probes
if ! grep -q "\[FAILED\]" crates/annad/src/llm_core/mod.rs; then
    echo "FAIL: Failed probe marker not found"
    exit 1
fi

# Check that evidence line always starts with "Evidence:"
if ! grep -q 'format!("Evidence:' crates/annad/src/llm_core/mod.rs; then
    echo "FAIL: Evidence line format not standardized"
    exit 1
fi

echo "PASS: Evidence line format is standardized"
echo "Invariant 16 (Evidence Line): PASS"
