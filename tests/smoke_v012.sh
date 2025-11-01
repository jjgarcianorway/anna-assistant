#!/usr/bin/env bash
# Anna v0.12.0 - Smoke Test
# Quick validation of core functionality

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "╭────────────────────────────────────────╮"
echo "│  Anna v0.12.0 Smoke Test              │"
echo "╰────────────────────────────────────────╯"
echo ""

failures=0

test_step() {
    local name="$1"
    shift
    echo -n "  ⏳ $name ... "
    if "$@" >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        ((failures++))
        return 1
    fi
}

test_step_output() {
    local name="$1"
    local expected="$2"
    shift 2
    echo -n "  ⏳ $name ... "
    if output=$("$@" 2>&1) && echo "$output" | grep -q "$expected"; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC} (expected: $expected)"
        ((failures++))
        return 1
    fi
}

# 1. Version checks
echo "1. Version Verification"
test_step_output "annactl version" "Anna v" annactl --version
test_step_output "Daemon running" "active (running)" systemctl status annad

# 2. RPC connectivity
echo ""
echo "2. RPC Connectivity"
test_step "annactl status (human)" annactl status
test_step "annactl status (JSON)" sh -c 'annactl status --json | jq empty'
test_step "Status shows running" sh -c 'annactl status --json | jq -e ".daemon_state == \"running\""'

# 3. Core telemetry commands
echo ""
echo "3. Core Telemetry Commands"
test_step "annactl sensors" annactl sensors
test_step "annactl sensors (JSON)" sh -c 'annactl sensors --json | jq empty'
test_step "annactl net" annactl net
test_step "annactl net (JSON)" sh -c 'annactl net --json | jq empty'
test_step "annactl disk" annactl disk
test_step "annactl disk (JSON)" sh -c 'annactl disk --json | jq empty'
test_step "annactl top" annactl top
test_step "annactl top (JSON)" sh -c 'annactl top --json | jq empty'

# 4. Events
echo ""
echo "4. Events"
test_step "annactl events" annactl events --limit 5
test_step "annactl events (JSON)" sh -c 'annactl events --json | jq empty'

# 5. Export
echo ""
echo "5. Export"
test_step "annactl export" sh -c 'annactl export | jq empty'

# 6. Classification & Radars
echo ""
echo "6. Classification & Radars"
test_step "annactl classify run" annactl classify run
test_step "annactl classify (JSON)" sh -c 'annactl classify run --json | jq empty'
test_step "annactl radar show" annactl radar show
test_step "annactl radar (JSON)" sh -c 'annactl radar show --json | jq empty'

# 7. Doctor commands
echo ""
echo "7. Doctor Commands"
test_step "annactl doctor pre" annactl doctor pre --json
test_step "annactl doctor post" annactl doctor post --json
test_step "Doctor post shows ok" sh -c 'annactl doctor post --json | jq -e ".ok == true"'

# 8. Database integrity
echo ""
echo "8. Database Integrity"
if command -v sqlite3 &>/dev/null; then
    test_step "Database exists" test -f /var/lib/anna/telemetry.db
    test_step "users table exists" sh -c 'sqlite3 /var/lib/anna/telemetry.db "SELECT name FROM sqlite_master WHERE type=\"table\" AND name=\"users\";" | grep -q users'
    test_step "metrics table exists" sh -c 'sqlite3 /var/lib/anna/telemetry.db "SELECT name FROM sqlite_master WHERE type=\"table\" AND name=\"metrics\";" | grep -q metrics'
    test_step "events table exists" sh -c 'sqlite3 /var/lib/anna/telemetry.db "SELECT name FROM sqlite_master WHERE type=\"table\" AND name=\"events\";" | grep -q events'
    test_step "radar_scores table exists" sh -c 'sqlite3 /var/lib/anna/telemetry.db "SELECT name FROM sqlite_master WHERE type=\"table\" AND name=\"radar_scores\";" | grep -q radar_scores'
else
    echo -e "  ${YELLOW}⚠${NC} sqlite3 not installed, skipping database tests"
fi

# 9. Socket & logs
echo ""
echo "9. Socket & Logs"
test_step "Socket exists" test -S /run/anna/annad.sock
test_step "Socket has correct perms" sh -c 'stat -c "%a %U:%G" /run/anna/annad.sock | grep -q "770 anna:anna"'
test_step "RPC socket ready logged" sh -c 'journalctl -u annad -b --no-pager | grep -q "RPC socket ready"'
test_step "Telemetry collection active" sh -c 'journalctl -u annad --since "5 minutes ago" --no-pager | grep -q "Collected telemetry"'

# Summary
echo ""
echo "╭────────────────────────────────────────╮"
if [[ $failures -eq 0 ]]; then
    echo -e "│  ${GREEN}✓ All smoke tests passed!${NC}              │"
    echo "╰────────────────────────────────────────╯"
    exit 0
else
    echo -e "│  ${RED}✗ $failures tests failed${NC}                   │"
    echo "╰────────────────────────────────────────╯"
    exit 1
fi
