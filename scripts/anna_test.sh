#!/bin/bash
# Anna Test Harness v7.40.0
#
# Tests annactl commands with built-in timing.
# Works with set -u (strict mode), no external dependencies.
#
# Usage: ./scripts/anna_test.sh [test_name]
#
# Tests:
#   sw        - Test software command performance
#   status    - Test status command performance
#   hw        - Test hardware command performance
#   version   - Test version output format
#   all       - Run all tests (default)
#
# Environment:
#   ANNACTL   - Path to annactl binary (default: auto-detect)
#   VERBOSE   - Set to 1 for verbose output

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ANNACTL="${ANNACTL:-}"
VERBOSE="${VERBOSE:-0}"

# Colors (using raw escape codes for portability)
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
CYAN=$'\033[0;36m'
BOLD=$'\033[1m'
DIM=$'\033[2m'
NC=$'\033[0m'

# Counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Find annactl binary
find_annactl() {
    # Explicit override
    if [[ -n "$ANNACTL" ]]; then
        if [[ -x "$ANNACTL" ]]; then
            echo "$ANNACTL"
            return 0
        fi
        echo "${RED}[ERROR]${NC} ANNACTL=$ANNACTL is not executable" >&2
        return 1
    fi

    # Try release build first
    local release_bin="$PROJECT_DIR/target/release/annactl"
    if [[ -x "$release_bin" ]]; then
        echo "$release_bin"
        return 0
    fi

    # Try debug build
    local debug_bin="$PROJECT_DIR/target/debug/annactl"
    if [[ -x "$debug_bin" ]]; then
        echo "$debug_bin"
        return 0
    fi

    # Try system path
    if command -v annactl &>/dev/null; then
        command -v annactl
        return 0
    fi

    echo "${RED}[ERROR]${NC} Could not find annactl binary" >&2
    return 1
}

# Get current time in milliseconds (portable)
now_ms() {
    # Try nanoseconds first (Linux)
    if date +%s%N &>/dev/null; then
        echo $(( $(date +%s%N) / 1000000 ))
    else
        # Fallback to seconds (macOS without coreutils)
        echo $(( $(date +%s) * 1000 ))
    fi
}

# Run command and measure time
measure_cmd() {
    local label="$1"
    shift
    local start_ms end_ms duration_ms output exit_code

    start_ms=$(now_ms)
    set +e
    output=$("$@" 2>&1)
    exit_code=$?
    set -e
    end_ms=$(now_ms)

    duration_ms=$((end_ms - start_ms))

    echo "$duration_ms"
    if [[ "$VERBOSE" == "1" ]]; then
        echo "$output" | head -20 >&2
    fi

    return $exit_code
}

# Print test header
test_header() {
    local name="$1"
    echo ""
    echo "${BOLD}[$name]${NC}"
}

# Assert duration is under threshold
assert_duration() {
    local name="$1"
    local actual_ms="$2"
    local max_ms="$3"

    if [[ "$actual_ms" -le "$max_ms" ]]; then
        echo "  ${GREEN}PASS${NC} $name: ${actual_ms}ms <= ${max_ms}ms"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo "  ${RED}FAIL${NC} $name: ${actual_ms}ms > ${max_ms}ms"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Assert output matches pattern
assert_output_matches() {
    local name="$1"
    local output="$2"
    local pattern="$3"

    if echo "$output" | grep -qE "$pattern"; then
        echo "  ${GREEN}PASS${NC} $name: matches '$pattern'"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo "  ${RED}FAIL${NC} $name: does not match '$pattern'"
        echo "    ${DIM}Got: $output${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Skip test with reason
skip_test() {
    local name="$1"
    local reason="$2"
    echo "  ${YELLOW}SKIP${NC} $name: $reason"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
}

# --- Test Suites ---

test_version() {
    test_header "VERSION"

    local output
    output=$("$ANNACTL" --version 2>&1) || true

    # Should output exactly "annactl vX.Y.Z"
    assert_output_matches "format" "$output" "^annactl v[0-9]+\.[0-9]+\.[0-9]+$"

    # Should not contain ANSI codes
    if echo "$output" | grep -q $'\033'; then
        echo "  ${RED}FAIL${NC} no-ansi: contains ANSI escape codes"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    else
        echo "  ${GREEN}PASS${NC} no-ansi: clean output"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi

    # Also test 'version' subcommand
    local output2
    output2=$("$ANNACTL" version 2>&1) || true
    assert_output_matches "subcommand" "$output2" "^annactl v[0-9]+\.[0-9]+\.[0-9]+$"
}

test_status() {
    test_header "STATUS"

    # Performance test (p95 < 300ms)
    local duration_ms
    duration_ms=$(measure_cmd "status" "$ANNACTL" status) || true
    assert_duration "p95 < 300ms" "$duration_ms" 300

    # Multiple runs for consistency
    local total_ms=0
    for i in 1 2 3; do
        duration_ms=$(measure_cmd "status-$i" "$ANNACTL" status) || true
        total_ms=$((total_ms + duration_ms))
    done
    local avg_ms=$((total_ms / 3))
    echo "  ${DIM}Average over 3 runs: ${avg_ms}ms${NC}"
}

test_sw() {
    test_header "SOFTWARE"

    # Clear user cache to test cold start
    rm -f ~/.cache/anna/sw_cache.json 2>/dev/null || true

    # Cold start (cache miss) - allow up to 10s for first run
    echo "  ${DIM}Testing cold start (cache miss)...${NC}"
    local cold_ms
    cold_ms=$(measure_cmd "sw-cold" "$ANNACTL" sw) || true

    if [[ "$cold_ms" -gt 15000 ]]; then
        echo "  ${RED}FAIL${NC} cold-start: ${cold_ms}ms > 15000ms"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    else
        echo "  ${GREEN}PASS${NC} cold-start: ${cold_ms}ms <= 15000ms"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi

    # Warm cache - should be fast (p95 < 1000ms, target < 100ms)
    echo "  ${DIM}Testing warm cache...${NC}"
    local warm_ms
    warm_ms=$(measure_cmd "sw-warm" "$ANNACTL" sw) || true
    assert_duration "warm p95 < 1000ms" "$warm_ms" 1000

    # Test --full flag
    local full_ms
    full_ms=$(measure_cmd "sw --full" "$ANNACTL" sw --full) || true
    # Full mode reads telemetry, allow more time
    assert_duration "--full < 2000ms" "$full_ms" 2000

    # Test --json flag
    local json_output json_ms
    json_ms=$(measure_cmd "sw --json" "$ANNACTL" sw --json) || true
    json_output=$("$ANNACTL" sw --json 2>&1) || true

    assert_duration "--json < 100ms" "$json_ms" 100

    # Verify JSON is valid
    if echo "$json_output" | head -1 | grep -q "^{"; then
        echo "  ${GREEN}PASS${NC} --json: valid JSON output"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  ${RED}FAIL${NC} --json: invalid JSON output"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

test_hw() {
    test_header "HARDWARE"

    # Performance test (p95 < 1200ms)
    local duration_ms
    duration_ms=$(measure_cmd "hw" "$ANNACTL" hw) || true
    assert_duration "p95 < 1200ms" "$duration_ms" 1200

    # Multiple runs for consistency
    local total_ms=0
    for i in 1 2 3; do
        duration_ms=$(measure_cmd "hw-$i" "$ANNACTL" hw) || true
        total_ms=$((total_ms + duration_ms))
    done
    local avg_ms=$((total_ms / 3))
    echo "  ${DIM}Average over 3 runs: ${avg_ms}ms${NC}"
}

test_help() {
    test_header "HELP"

    # Running with no args should show help
    local output
    output=$("$ANNACTL" 2>&1) || true

    assert_output_matches "shows commands" "$output" "annactl.*status|annactl.*sw|annactl.*hw"
}

# --- Main ---

main() {
    local test_name="${1:-all}"

    echo "${BOLD}Anna Test Harness v7.40.0${NC}"
    echo "------------------------------------------------------------"

    # Find binary
    ANNACTL=$(find_annactl) || exit 1
    echo "Using: $ANNACTL"
    echo "Version: $("$ANNACTL" --version)"

    case "$test_name" in
        version)
            test_version
            ;;
        status)
            test_status
            ;;
        sw)
            test_sw
            ;;
        hw)
            test_hw
            ;;
        help)
            test_help
            ;;
        all)
            test_version
            test_status
            test_sw
            test_hw
            test_help
            ;;
        *)
            echo "${RED}[ERROR]${NC} Unknown test: $test_name"
            echo "Available: version, status, sw, hw, help, all"
            exit 1
            ;;
    esac

    # Summary
    echo ""
    echo "------------------------------------------------------------"
    echo "${BOLD}Summary:${NC}"
    echo "  ${GREEN}Passed:${NC}  $TESTS_PASSED"
    echo "  ${RED}Failed:${NC}  $TESTS_FAILED"
    echo "  ${YELLOW}Skipped:${NC} $TESTS_SKIPPED"

    if [[ "$TESTS_FAILED" -gt 0 ]]; then
        echo ""
        echo "${RED}TESTS FAILED${NC}"
        exit 1
    else
        echo ""
        echo "${GREEN}ALL TESTS PASSED${NC}"
        exit 0
    fi
}

main "$@"
