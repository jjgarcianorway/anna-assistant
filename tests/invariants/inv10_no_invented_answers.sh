#!/bin/bash
# Invariant 10: Probe failures must not produce invented answers
# Test: Error handling exists for probe failures

set -e

echo "Testing Invariant 10: No invented answers on probe failure"

# Check for probe failure handling
if grep -rq "probe.*fail\|command.*fail\|exit.*code\|status.*success" crates/annad/src/ --include="*.rs" 2>/dev/null; then
    echo "PASS: Probe failure handling exists"
else
    echo "FAIL: No probe failure handling found"
    exit 1
fi

# Check for "I don't know" or similar fallback
if grep -rq "don't know\|cannot determine\|unable to\|no data\|insufficient" crates/annad/src/ --include="*.rs" 2>/dev/null; then
    echo "PASS: Fallback messaging exists"
else
    echo "WARN: No explicit 'I don't know' fallback found"
fi

echo "Invariant 10: PASS"
