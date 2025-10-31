#!/usr/bin/env bash
# Anna v0.11.0 - annactl Command Matrix Test
# Tests all annactl subcommands and captures outputs

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
PASS=0
FAIL=0

echo "╭─────────────────────────────────────────╮"
echo "│  annactl Command Matrix Test           │"
echo "╰─────────────────────────────────────────╯"
echo ""

pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++))
}

info() {
    echo -e "${BLUE}→${NC} $1"
}

test_command() {
    local cmd="$1"
    local description="$2"

    info "Testing: $description"
    if timeout 10 $cmd &>/dev/null; then
        pass "$description"
    else
        fail "$description (exit code: $?)"
    fi
}

# Test commands
info "Running annactl command matrix..."
echo ""

test_command "annactl version" "annactl version"
test_command "annactl status" "annactl status"
test_command "annactl sensors" "annactl sensors"
test_command "annactl net" "annactl net"
test_command "annactl disk" "annactl disk"
test_command "annactl top" "annactl top"
test_command "annactl radar" "annactl radar"
test_command "annactl events --limit 3" "annactl events --limit 3"
test_command "annactl capabilities" "annactl capabilities"
test_command "annactl alerts" "annactl alerts"

# Doctor commands
test_command "annactl doctor pre --verbose" "annactl doctor pre"
test_command "annactl doctor post --verbose" "annactl doctor post"

echo ""
echo "═════════════════════════════════════════"
echo "SUMMARY"
echo "═════════════════════════════════════════"
echo "  Passed:  $PASS"
echo "  Failed:  $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ All annactl commands passed${NC}"
    exit 0
else
    echo -e "${RED}✗ Some annactl commands failed${NC}"
    exit 1
fi
