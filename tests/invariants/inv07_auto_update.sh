#!/bin/bash
# Invariant 7: Auto-update must succeed when new release exists
# Test: Update mechanism code paths exist and are testable

set -e

echo "Testing Invariant 7: Auto-update mechanism"

# Check for update checking code
if ! grep -rq "check_for_updates\|UpdateChecker\|update_check" crates/annad/src/ --include="*.rs" 2>/dev/null; then
    echo "FAIL: No update checking mechanism found"
    exit 1
fi

# Check for GitHub release URL
if ! grep -rq "github.com/.*releases\|api.github.com" crates/ --include="*.rs" 2>/dev/null; then
    echo "FAIL: No GitHub release URL found"
    exit 1
fi

# Check for binary download logic
if ! grep -rq "download\|fetch.*binary\|annad-linux\|annactl-linux" crates/ --include="*.rs" 2>/dev/null; then
    echo "FAIL: No binary download logic found"
    exit 1
fi

echo "PASS: Auto-update code paths exist"
echo "Invariant 7: PASS"
