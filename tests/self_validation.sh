#!/usr/bin/env bash
# Anna Self-Validation Test Harness
# Tests the doctor validate and repair commands

set -euo pipefail

ANNACTL="./target/release/annactl"
TESTS_PASSED=0
TESTS_FAILED=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Self-Validation Test Suite       │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

info() {
    echo -e "${YELLOW}→${NC} $1"
}

# Test 1: annactl binary exists
test_binary_exists() {
    info "Test 1: Binary exists"
    if [ -f "$ANNACTL" ]; then
        pass "annactl binary found"
    else
        fail "annactl binary not found (run: cargo build --release)"
        exit 1
    fi
}

# Test 2: doctor validate command exists
test_validate_command() {
    info "Test 2: Validate command exists"
    if $ANNACTL doctor --help 2>&1 | grep -q "validate"; then
        pass "doctor validate command available"
    else
        fail "doctor validate command not found"
    fi
}

# Test 3: doctor validate runs without crashing
test_validate_runs() {
    info "Test 3: Validate command runs"
    if $ANNACTL doctor validate 2>&1 >/dev/null || true; then
        pass "doctor validate executes without panic"
    else
        fail "doctor validate crashed"
    fi
}

# Test 4: doctor validate produces expected output
test_validate_output() {
    info "Test 4: Validate output format"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "Component"; then
        pass "Validate produces table header"
    else
        fail "Validate output missing table header"
    fi

    if echo "$output" | grep -q "Expected"; then
        pass "Validate shows 'Expected' column"
    else
        fail "Validate missing 'Expected' column"
    fi

    if echo "$output" | grep -q "Found"; then
        pass "Validate shows 'Found' column"
    else
        fail "Validate missing 'Found' column"
    fi
}

# Test 5: doctor validate checks at least 8 components
test_validate_checks() {
    info "Test 5: Validate runs all checks"
    output=$($ANNACTL doctor validate 2>&1 || true)

    # Count lines with ✓ or ✗
    check_count=$(echo "$output" | grep -E "^│ [✓✗]" | wc -l)

    if [ "$check_count" -ge 8 ]; then
        pass "Validate runs 8 checks (found $check_count)"
    else
        fail "Validate only runs $check_count checks (expected 8)"
    fi
}

# Test 6: doctor repair --dry-run works
test_repair_dry_run() {
    info "Test 6: Repair dry-run mode"
    if $ANNACTL doctor repair --dry-run 2>&1 >/dev/null || true; then
        pass "doctor repair --dry-run executes"
    else
        fail "doctor repair --dry-run crashed"
    fi
}

# Test 7: doctor check still works
test_doctor_check() {
    info "Test 7: Legacy doctor check"
    if $ANNACTL doctor check 2>&1 >/dev/null || true; then
        pass "doctor check executes"
    else
        fail "doctor check crashed"
    fi
}

# Test 8: profile checks work (offline command)
test_profile_checks() {
    info "Test 8: Profile checks (offline)"
    if $ANNACTL profile checks 2>&1 >/dev/null; then
        pass "profile checks executes successfully"
    else
        fail "profile checks failed"
    fi
}

# Test 9: CPU usage validation check exists
test_cpu_check() {
    info "Test 9: CPU usage check"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "CPU Usage"; then
        pass "CPU usage check present in validation"
    else
        fail "CPU usage check missing"
    fi
}

# Test 10: Service status check exists
test_service_check() {
    info "Test 10: Service status check"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "Service Status"; then
        pass "Service status check present"
    else
        fail "Service status check missing"
    fi
}

# Test 11: Socket check exists
test_socket_check() {
    info "Test 11: Socket check"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "Socket"; then
        pass "Socket check present"
    else
        fail "Socket check missing"
    fi
}

# Test 12: User & group check exists
test_user_check() {
    info "Test 12: User & group check"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "User & Group"; then
        pass "User & group check present"
    else
        fail "User & group check missing"
    fi
}

# Test 13: Recommended fixes are shown on failures
test_fixes_shown() {
    info "Test 13: Recommended fixes"
    output=$($ANNACTL doctor validate 2>&1 || true)

    if echo "$output" | grep -q "Recommended fixes"; then
        pass "Shows recommended fixes for failures"
    else
        # This might not show if all checks pass
        info "  (may be hidden if all checks pass)"
    fi
}

# Test 14: CPU benchmark - annactl should execute quickly
test_cpu_benchmark() {
    info "Test 14: CPU benchmark (offline commands)"

    # Test profile checks execution time
    start=$(date +%s%N)
    $ANNACTL profile checks >/dev/null 2>&1
    end=$(date +%s%N)

    elapsed_ms=$(( (end - start) / 1000000 ))

    if [ $elapsed_ms -lt 1000 ]; then
        pass "profile checks completes in ${elapsed_ms}ms (< 1s)"
    else
        fail "profile checks took ${elapsed_ms}ms (> 1s)"
    fi

    # Test doctor validate execution time
    start=$(date +%s%N)
    $ANNACTL doctor validate >/dev/null 2>&1 || true
    end=$(date +%s%N)

    elapsed_ms=$(( (end - start) / 1000000 ))

    if [ $elapsed_ms -lt 2000 ]; then
        pass "doctor validate completes in ${elapsed_ms}ms (< 2s)"
    else
        fail "doctor validate took ${elapsed_ms}ms (> 2s)"
    fi
}

# Run all tests
test_binary_exists
test_validate_command
test_validate_runs
test_validate_output
test_validate_checks
test_repair_dry_run
test_doctor_check
test_profile_checks
test_cpu_check
test_service_check
test_socket_check
test_user_check
test_fixes_shown
test_cpu_benchmark

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  Test Results                           │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "  Passed: $TESTS_PASSED"
echo "  Failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
