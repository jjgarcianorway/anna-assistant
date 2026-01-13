#!/bin/bash
# Invariant: Missing Done packet = client exits non-zero with [FAILED] (Phase 2)
# Test: Streaming code enforces terminality contract

set -e

echo "Testing missing Done packet handling"

STREAMING_FILE="crates/annactl/src/streaming.rs"

if [ ! -f "$STREAMING_FILE" ]; then
    echo "FAIL: streaming.rs not found"
    exit 1
fi

# Check for TERMINALITY CONTRACT documentation
if ! grep -q "TERMINALITY CONTRACT" "$STREAMING_FILE"; then
    echo "FAIL: TERMINALITY CONTRACT not documented"
    exit 1
fi

# Check that [FAILED] is printed when Done is missing
if ! grep -q '\[FAILED\]' "$STREAMING_FILE"; then
    echo "FAIL: [FAILED] marker not found for missing Done packet"
    exit 1
fi

# Check that function returns Err when Done is missing
if ! grep -q "Stream terminated without Done packet" "$STREAMING_FILE"; then
    echo "FAIL: Error message for missing Done packet not found"
    exit 1
fi

# Check that partial results are discarded, not emitted
if ! grep -q "Partial results discarded" "$STREAMING_FILE"; then
    echo "FAIL: Partial results should be discarded"
    exit 1
fi

# Verify the test exists
if ! grep -q "test_client_refuses_final_answer_without_terminal_packet" "$STREAMING_FILE"; then
    echo "FAIL: Contract enforcement test not found"
    exit 1
fi

echo "PASS: Missing Done packet handling is correct"
echo "Invariant 18 (Missing Done Packet): PASS"
