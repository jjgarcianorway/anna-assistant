#!/usr/bin/env bash
#
# Brain Sanity Tests - v7.0.0
#
# Property-based golden tests for Anna's brain_v7 architecture.
# Tests check for expected properties rather than exact strings.
#
# Usage:
#   ./tests/brain_sanity.sh           # Run all tests
#   ./tests/brain_sanity.sh verbose   # Run with detailed output
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ANNACTL="${PROJECT_ROOT}/target/release/annactl"

# Test counters
PASSED=0
FAILED=0
SKIPPED=0

# Verbose mode
VERBOSE="${1:-}"

log_info() {
    echo -e "${CYAN}ℹ️  ${NC}$1"
}

log_pass() {
    echo -e "${GREEN}✅  PASS${NC}: $1"
    ((PASSED++))
}

log_fail() {
    echo -e "${RED}❌  FAIL${NC}: $1"
    ((FAILED++))
}

log_skip() {
    echo -e "${YELLOW}⏭️  SKIP${NC}: $1"
    ((SKIPPED++))
}

# Check if annactl exists
if [[ ! -x "$ANNACTL" ]]; then
    echo -e "${RED}ERROR${NC}: annactl not found at $ANNACTL"
    echo "Build it first: cargo build --release --bin annactl"
    exit 1
fi

echo -e "${BOLD}🧠  Brain v7 Sanity Tests${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Helper: Run query and check for patterns
# Usage: test_query "name" "query" "pattern1" "pattern2" ...
test_query() {
    local name="$1"
    local query="$2"
    shift 2
    local patterns=("$@")

    if [[ "$VERBOSE" == "verbose" ]]; then
        echo -e "${CYAN}Testing${NC}: $name"
        echo "  Query: \"$query\""
    fi

    # Run annactl and capture output
    local output
    if ! output=$("$ANNACTL" "$query" 2>&1); then
        log_fail "$name (annactl error)"
        if [[ "$VERBOSE" == "verbose" ]]; then
            echo "  Error: $output"
        fi
        return 1
    fi

    local output_lower
    output_lower=$(echo "$output" | tr '[:upper:]' '[:lower:]')

    # Check all patterns
    local all_match=true
    local missing_patterns=()

    for pattern in "${patterns[@]}"; do
        local pattern_lower
        pattern_lower=$(echo "$pattern" | tr '[:upper:]' '[:lower:]')

        if ! echo "$output_lower" | grep -qE "$pattern_lower"; then
            all_match=false
            missing_patterns+=("$pattern")
        fi
    done

    if $all_match; then
        log_pass "$name"
        if [[ "$VERBOSE" == "verbose" ]]; then
            echo "  Output preview: ${output:0:100}..."
        fi
        return 0
    else
        log_fail "$name"
        echo "  Missing patterns: ${missing_patterns[*]}"
        if [[ "$VERBOSE" == "verbose" ]]; then
            echo "  Full output:"
            echo "$output" | head -20 | sed 's/^/    /'
        fi
        return 1
    fi
}

# Helper: Test query does NOT contain certain patterns (anti-hallucination)
test_no_hallucination() {
    local name="$1"
    local query="$2"
    shift 2
    local forbidden=("$@")

    if [[ "$VERBOSE" == "verbose" ]]; then
        echo -e "${CYAN}Testing${NC}: $name (anti-hallucination)"
    fi

    local output
    if ! output=$("$ANNACTL" "$query" 2>&1); then
        log_fail "$name (annactl error)"
        return 1
    fi

    local output_lower
    output_lower=$(echo "$output" | tr '[:upper:]' '[:lower:]')

    local found_forbidden=()
    for pattern in "${forbidden[@]}"; do
        local pattern_lower
        pattern_lower=$(echo "$pattern" | tr '[:upper:]' '[:lower:]')

        if echo "$output_lower" | grep -qE "$pattern_lower"; then
            found_forbidden+=("$pattern")
        fi
    done

    if [[ ${#found_forbidden[@]} -eq 0 ]]; then
        log_pass "$name"
        return 0
    else
        log_fail "$name - found forbidden: ${found_forbidden[*]}"
        return 1
    fi
}

# ============================================================================
# HARDWARE QUERIES
# ============================================================================

echo -e "${BOLD}📦  Hardware Queries${NC}"
echo ""

# RAM: Should contain "gb" or "gib" and a number
test_query "RAM query" \
    "how much RAM do I have?" \
    "[0-9]+" "gb|gib|memory"

# CPU: Should contain model name
test_query "CPU query" \
    "what CPU do I have?" \
    "intel|amd|core|ryzen"

# GPU: Should contain vendor or GPU identifier
test_query "GPU query" \
    "what GPU do I have?" \
    "nvidia|amd|intel|radeon|geforce|rtx|gtx"

echo ""

# ============================================================================
# PACKAGE QUERIES
# ============================================================================

echo -e "${BOLD}📦  Package Queries${NC}"
echo ""

# Check if a common package is installed (steam is usually installed on this system)
test_query "Package check (steam)" \
    "is steam installed?" \
    "steam" "installed|available|found|yes"

# Check for a package that shouldn't exist (anti-hallucination)
test_no_hallucination "No hallucinated packages" \
    "is 7zip installed?" \
    "7zip.*installed|yes.*7zip"

echo ""

# ============================================================================
# SYSTEM QUERIES
# ============================================================================

echo -e "${BOLD}🖥️  System Queries${NC}"
echo ""

# Desktop environment
test_query "Desktop environment" \
    "what desktop environment am I using?" \
    "gnome|kde|xfce|wayland|x11|desktop"

# Failed services (should say none or list them)
test_query "Failed services" \
    "are there any failed systemd services?" \
    "no failed|failed|service|systemd"

# Uptime
test_query "Uptime" \
    "what is the system uptime?" \
    "up|day|hour|minute|running"

echo ""

# ============================================================================
# STORAGE QUERIES
# ============================================================================

echo -e "${BOLD}💾  Storage Queries${NC}"
echo ""

# Disk usage
test_query "Disk usage" \
    "how much disk space is free?" \
    "[0-9]+" "gb|gib|free|available|used"

echo ""

# ============================================================================
# ANTI-HALLUCINATION TESTS
# ============================================================================

echo -e "${BOLD}🚫  Anti-Hallucination Tests${NC}"
echo ""

# Anna should NOT claim to do things she can't do
test_no_hallucination "No external capabilities" \
    "what is the weather in Tokyo?" \
    "weather.*tokyo|sunny|cloudy|temperature|degrees"

echo ""

# ============================================================================
# SUMMARY
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BOLD}📊  Summary${NC}"
echo ""
echo -e "  ${GREEN}Passed${NC}:  $PASSED"
echo -e "  ${RED}Failed${NC}:  $FAILED"
echo -e "  ${YELLOW}Skipped${NC}: $SKIPPED"
echo ""

TOTAL=$((PASSED + FAILED))
if [[ $TOTAL -gt 0 ]]; then
    PERCENT=$((PASSED * 100 / TOTAL))
    echo -e "  Pass rate: ${BOLD}${PERCENT}%${NC}"
fi

echo ""

if [[ $FAILED -gt 0 ]]; then
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
