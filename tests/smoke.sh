#!/bin/bash
# Anna v0.10.1 Smoke Tests
# Tests all 7 CLI commands in pretty and JSON modes

set -e

C_GREEN="\e[32m"
C_RED="\e[31m"
C_RESET="\e[0m"

PASS=0
FAIL=0

pass() {
    echo -e "${C_GREEN}✓${C_RESET} $1"
    ((PASS++))
}

fail() {
    echo -e "${C_RED}✗${C_RESET} $1"
    ((FAIL++))
}

test_cmd() {
    local name=$1
    local cmd=$2
    local expect_json=$3

    echo -n "Testing: $name ... "

    if output=$(eval "$cmd" 2>&1); then
        if [[ "$expect_json" == "json" ]]; then
            if echo "$output" | jq . >/dev/null 2>&1; then
                pass "$name (valid JSON)"
            else
                fail "$name (invalid JSON)"
            fi
        else
            pass "$name"
        fi
    else
        fail "$name (exit code $?)"
    fi
}

echo "╭─ Anna v0.10.1 Smoke Tests ──────────────────"
echo "│"

# Check daemon is running
if systemctl is-active --quiet annad; then
    pass "Daemon is active"
else
    fail "Daemon is not active"
    echo "│"
    echo "╰──────────────────────────────────────────────"
    echo ""
    echo "Daemon must be running for smoke tests."
    echo "Run: sudo systemctl start annad"
    exit 1
fi

# Wait for telemetry
echo "│  Waiting for telemetry (10s)..."
sleep 10

# Test CLI commands (pretty mode)
test_cmd "annactl version" "annactl version" ""
test_cmd "annactl status" "annactl status" ""
test_cmd "annactl sensors" "annactl sensors" ""
test_cmd "annactl net" "annactl net" ""
test_cmd "annactl disk" "annactl disk" ""
test_cmd "annactl top" "annactl top" ""
test_cmd "annactl radar" "annactl radar" ""

# Test JSON export
test_cmd "annactl export" "annactl export" "json"

# Check database exists
if [[ -f /var/lib/anna/telemetry.db ]]; then
    pass "Database exists"
else
    fail "Database not found"
fi

# Check socket exists
if [[ -S /run/anna/annad.sock ]]; then
    pass "Socket exists"
else
    fail "Socket not found"
fi

# Performance checks
cpu_usage=$(ps aux | grep '[a]nnad' | awk '{print $3}')
mem_usage=$(ps aux | grep '[a]nnad' | awk '{print $6}')

if (( $(echo "$cpu_usage < 5.0" | bc -l 2>/dev/null || echo 0) )); then
    pass "CPU usage OK (<5%): ${cpu_usage}%"
else
    fail "CPU usage high: ${cpu_usage}%"
fi

mem_mb=$((mem_usage / 1024))
if (( mem_mb < 80 )); then
    pass "Memory usage OK (<80MB): ${mem_mb}MB"
else
    fail "Memory usage high: ${mem_mb}MB"
fi

echo "│"
echo "╰──────────────────────────────────────────────"
echo ""

# Summary
if (( FAIL > 0 )); then
    echo -e "${C_RED}FAILED${C_RESET}: $PASS passed, $FAIL failed"
    exit 1
else
    echo -e "${C_GREEN}SUCCESS${C_RESET}: All $PASS tests passed"
    exit 0
fi
