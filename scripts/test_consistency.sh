#!/bin/bash
# Test consistency between TUI and one-shot mode
# Beta.97: Ensures system reliability by checking answer consistency

set -e

TEST_QUESTION="${1:-What are the weak points of my system?}"
TMP_DIR=$(mktemp -d)

echo "=== Consistency Test ==="
echo "Question: $TEST_QUESTION"
echo ""

# Test 1: One-shot mode
echo "Testing one-shot mode..."
echo "$TEST_QUESTION" | annactl > "$TMP_DIR/oneshot.txt" 2>&1

# Test 2: TUI mode (simulated)
echo "Testing TUI mode..."
echo -e "$TEST_QUESTION\nexit" | annactl tui > "$TMP_DIR/tui.txt" 2>&1

# Compare outputs (ignoring UI formatting differences)
echo "Comparing outputs..."
diff_result=$(diff -u "$TMP_DIR/oneshot.txt" "$TMP_DIR/tui.txt" || true)

if [ -z "$diff_result" ]; then
    echo "✅ PASS: Outputs are identical"
    exit 0
else
    echo "❌ FAIL: Outputs differ!"
    echo ""
    echo "=== One-shot output ==="
    cat "$TMP_DIR/oneshot.txt"
    echo ""
    echo "=== TUI output ==="
    cat "$TMP_DIR/tui.txt"
    echo ""
    echo "=== Diff ==="
    echo "$diff_result"
    exit 1
fi

# Cleanup
rm -rf "$TMP_DIR"
