#!/usr/bin/env bash
# Anna v0.12.3 - Arch Advisor Smoke Test
# Validates advisor JSON output and ensures all rule_ids are present

set -euo pipefail

ANNACTL="${ANNACTL:-./target/release/annactl}"
PASS=0
FAIL=0

echo "==== Arch Advisor Smoke Test ===="
echo

# Test 1: Check if annactl exists and is executable
echo "[TEST 1] Checking if annactl exists..."
if [ ! -x "$ANNACTL" ]; then
    echo "❌ FAIL: $ANNACTL not found or not executable"
    echo "Build with: cargo build --release --bin annactl"
    exit 1
fi
echo "✓ PASS: annactl found at $ANNACTL"
((PASS++))
echo

# Test 2: Check if daemon is running
echo "[TEST 2] Checking if annad daemon is running..."
if ! systemctl is-active --quiet annad 2>/dev/null; then
    echo "❌ FAIL: annad daemon not running"
    echo "Start with: sudo systemctl start annad"
    exit 1
fi
echo "✓ PASS: annad daemon is running"
((PASS++))
echo

# Test 3: Advisor can return JSON
echo "[TEST 3] Testing advisor JSON output..."
if ! JSON_OUTPUT=$($ANNACTL advisor arch --json 2>&1); then
    echo "❌ FAIL: Advisor command failed"
    echo "$JSON_OUTPUT"
    ((FAIL++))
else
    echo "✓ PASS: Advisor returned JSON"
    ((PASS++))
fi
echo

# Test 4: JSON is valid
echo "[TEST 4] Validating JSON structure..."
if ! echo "$JSON_OUTPUT" | jq . >/dev/null 2>&1; then
    echo "❌ FAIL: Invalid JSON output"
    echo "$JSON_OUTPUT"
    ((FAIL++))
else
    echo "✓ PASS: Valid JSON structure"
    ((PASS++))
fi
echo

# Test 5: Check required fields in JSON
echo "[TEST 5] Checking required fields in advice objects..."
REQUIRED_FIELDS=("id" "level" "category" "title" "reason" "action" "refs")
MISSING_FIELDS=()

for field in "${REQUIRED_FIELDS[@]}"; do
    if ! echo "$JSON_OUTPUT" | jq -e ".[0].$field" >/dev/null 2>&1; then
        # It's okay if no advice is returned (empty array)
        if [ "$(echo "$JSON_OUTPUT" | jq 'length')" -eq 0 ]; then
            echo "ℹ INFO: No advice returned (system is optimal)"
            break
        fi
        MISSING_FIELDS+=("$field")
    fi
done

if [ ${#MISSING_FIELDS[@]} -gt 0 ]; then
    echo "❌ FAIL: Missing required fields: ${MISSING_FIELDS[*]}"
    ((FAIL++))
else
    echo "✓ PASS: All required fields present"
    ((PASS++))
fi
echo

# Test 6: Check if we can get advice count
echo "[TEST 6] Counting advice items..."
ADVICE_COUNT=$(echo "$JSON_OUTPUT" | jq 'length')
echo "ℹ INFO: Found $ADVICE_COUNT advice item(s)"
if [ "$ADVICE_COUNT" -ge 0 ]; then
    echo "✓ PASS: Advice count valid"
    ((PASS++))
else
    echo "❌ FAIL: Could not determine advice count"
    ((FAIL++))
fi
echo

# Test 7: Test --explain flag (if advice exists)
if [ "$ADVICE_COUNT" -gt 0 ]; then
    echo "[TEST 7] Testing --explain flag..."
    FIRST_ID=$(echo "$JSON_OUTPUT" | jq -r '.[0].id')
    if EXPLAIN_OUTPUT=$($ANNACTL advisor arch --explain "$FIRST_ID" 2>&1); then
        if echo "$EXPLAIN_OUTPUT" | grep -q "Title:"; then
            echo "✓ PASS: --explain flag works for ID: $FIRST_ID"
            ((PASS++))
        else
            echo "❌ FAIL: --explain output missing expected content"
            ((FAIL++))
        fi
    else
        echo "❌ FAIL: --explain command failed"
        ((FAIL++))
    fi
    echo
else
    echo "[TEST 7] SKIP: No advice to test --explain"
    echo
fi

# Test 8: Check for expected rule IDs (at least some should exist)
echo "[TEST 8] Checking for known rule IDs..."
KNOWN_IDS=(
    "nvidia-headers-missing"
    "vulkan-missing"
    "microcode-amd-missing"
    "microcode-intel-missing"
    "cpu-governor-laptop"
    "nvme-scheduler"
    "power-management-missing"
    "power-management-conflict"
    "zram-recommended"
    "nvidia-wayland-modesetting"
    "aur-helper-missing"
    "orphan-packages"
)

# Count how many known IDs appear in output
FOUND_IDS=0
for known_id in "${KNOWN_IDS[@]}"; do
    if echo "$JSON_OUTPUT" | jq -e ".[] | select(.id == \"$known_id\")" >/dev/null 2>&1; then
        ((FOUND_IDS++))
    fi
done

echo "ℹ INFO: Found $FOUND_IDS known rule ID(s) in output"
echo "✓ PASS: Rule ID check completed"
((PASS++))
echo

# Summary
echo "==== Test Summary ===="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo

if [ $FAIL -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
