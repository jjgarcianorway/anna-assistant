#!/usr/bin/env bash
# Anna v0.11.0 - Event Engine Smoke Tests
#
# Tests event-driven intelligence system with simulated triggers.
# Run after fresh installation to verify system health.

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0
SKIP=0

echo "╭─────────────────────────────────────────╮"
echo "│  Anna v0.11.0 Event Engine Smoke Test  │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++))
}

skip() {
    echo -e "${YELLOW}○${NC} $1"
    ((SKIP++))
}

info() {
    echo -e "${BLUE}→${NC} $1"
}

# ============================================================================
# SECTION 1: Prerequisites
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 1: Prerequisites"
echo "─────────────────────────────────────────"
echo ""

# Test 1.1: Check binaries exist
info "1.1 Checking binaries..."
if [ -x /usr/local/bin/annad ] && [ -x /usr/local/bin/annactl ]; then
    pass "Binaries installed and executable"
else
    fail "Binaries missing or not executable"
fi

# Test 1.2: Check anna user exists
info "1.2 Checking anna user..."
if id anna &>/dev/null; then
    pass "User 'anna' exists"
else
    fail "User 'anna' not found"
fi

# Test 1.3: Check directories
info "1.3 Checking directories..."
DIRS_OK=true
for dir in /var/lib/anna /var/log/anna /run/anna /etc/anna; do
    if [ ! -d "$dir" ]; then
        DIRS_OK=false
        echo "   Missing: $dir"
    fi
done

if [ "$DIRS_OK" = true ]; then
    pass "Required directories exist"
else
    fail "Some directories missing"
fi

# Test 1.4: Check systemd service
info "1.4 Checking systemd service..."
if [ -f /etc/systemd/system/annad.service ]; then
    pass "Systemd service file installed"
else
    fail "Systemd service file missing"
fi

# Test 1.5: Check policy.toml
info "1.5 Checking policy.toml..."
if [ -f /etc/anna/policy.toml ]; then
    pass "Event auto-repair policy installed"
else
    fail "policy.toml missing"
fi

echo ""

# ============================================================================
# SECTION 2: Daemon Health
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 2: Daemon Health"
echo "─────────────────────────────────────────"
echo ""

# Test 2.1: Check daemon is running
info "2.1 Checking daemon status..."
if systemctl is-active --quiet annad; then
    pass "Daemon is running"
else
    fail "Daemon is not running"
    info "   Try: sudo systemctl start annad"
fi

# Test 2.2: Check socket exists (poll up to 15 seconds)
info "2.2 Checking RPC socket (polling up to 15s)..."
SOCKET_FOUND=false
for i in {1..15}; do
    if [ -S /run/anna/annad.sock ]; then
        SOCKET_FOUND=true
        break
    fi
    sleep 1
done

if [ "$SOCKET_FOUND" = true ]; then
    pass "RPC socket exists"
else
    fail "RPC socket missing after 15 seconds"
fi

# Test 2.3: Test annactl status
info "2.3 Testing 'annactl status'..."
if timeout 5 annactl status &>/dev/null; then
    pass "annactl status works"
else
    fail "annactl status failed"
fi

# Test 2.4: Test annactl events
info "2.4 Testing 'annactl events --limit 1'..."
if timeout 5 annactl events --limit 1 &>/dev/null; then
    pass "annactl events works"
else
    fail "annactl events failed"
fi

# Test 2.5: Check daemon resource usage
info "2.5 Checking resource usage..."
if systemctl is-active --quiet annad; then
    PID=$(systemctl show -p MainPID --value annad)
    if [ -n "$PID" ] && [ "$PID" != "0" ]; then
        # Get RSS in KB
        RSS_KB=$(ps -o rss= -p "$PID" | tr -d ' ')
        RSS_MB=$((RSS_KB / 1024))

        if [ "$RSS_MB" -lt 80 ]; then
            pass "Memory usage: ${RSS_MB}MB (under 80MB limit)"
        else
            fail "Memory usage: ${RSS_MB}MB (exceeds 80MB limit)"
        fi
    else
        skip "Could not determine daemon PID"
    fi
else
    skip "Daemon not running, skipping resource check"
fi

echo ""

# ============================================================================
# SECTION 3: CLI Commands
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 3: CLI Commands"
echo "─────────────────────────────────────────"
echo ""

# Test 3.1: annactl version
info "3.1 Testing 'annactl version'..."
if timeout 5 annactl version >/dev/null 2>&1; then
    pass "annactl version works"
else
    fail "annactl version failed"
fi

# Test 3.2: annactl status
info "3.2 Testing 'annactl status'..."
if timeout 5 annactl status >/dev/null 2>&1; then
    pass "annactl status works"
else
    fail "annactl status failed"
fi

# Test 3.3: annactl sensors
info "3.3 Testing 'annactl sensors'..."
if timeout 5 annactl sensors >/dev/null 2>&1; then
    pass "annactl sensors works"
else
    fail "annactl sensors failed"
fi

# Test 3.4: annactl events (v0.11.0)
info "3.4 Testing 'annactl events'..."
if timeout 5 annactl events --limit 10 >/dev/null 2>&1; then
    pass "annactl events works"
else
    fail "annactl events failed"
fi

# Test 3.5: annactl capabilities
info "3.5 Testing 'annactl capabilities'..."
if timeout 5 annactl capabilities >/dev/null 2>&1; then
    pass "annactl capabilities works"
else
    fail "annactl capabilities failed"
fi

# Test 3.6: annactl alerts
info "3.6 Testing 'annactl alerts'..."
if timeout 5 annactl alerts >/dev/null 2>&1; then
    pass "annactl alerts works"
else
    fail "annactl alerts failed"
fi

echo ""

# ============================================================================
# SECTION 4: Event Simulation (if daemon is running)
# ============================================================================

if systemctl is-active --quiet annad; then
    echo "─────────────────────────────────────────"
    echo "SECTION 4: Event Simulation"
    echo "─────────────────────────────────────────"
    echo ""

    # Test 4.1: Trigger config event
    info "4.1 Simulating config event..."
    TEST_FILE="/tmp/anna_test_config.txt"
    echo "test" > "$TEST_FILE"
    if [ -f /etc/resolv.conf ]; then
        # Touch resolv.conf to trigger config watcher
        sudo touch /etc/resolv.conf
        sleep 2
        pass "Config event triggered (check with 'annactl events')"
    else
        skip "resolv.conf not found"
    fi
    rm -f "$TEST_FILE"

    # Test 4.2: Check event history
    info "4.2 Checking event history..."
    EVENT_COUNT=$(annactl events --limit 100 2>/dev/null | grep -c "│  " || true)
    if [ "$EVENT_COUNT" -gt 0 ]; then
        pass "Event history contains $EVENT_COUNT events"
    else
        skip "No events recorded yet (system may need more time)"
    fi

    # Test 4.3: Check for alerts
    info "4.3 Checking alerts..."
    if [ -f /var/lib/anna/alerts.json ]; then
        ALERT_COUNT=$(jq '.alerts | length' /var/lib/anna/alerts.json 2>/dev/null || echo 0)
        if [ "$ALERT_COUNT" -gt 0 ]; then
            info "   Found $ALERT_COUNT alerts in alerts.json"
            pass "Alerts system is active"
        else
            pass "Alerts system is active (no alerts currently)"
        fi
    else
        skip "alerts.json not created yet"
    fi

    echo ""
else
    echo "─────────────────────────────────────────"
    echo "SECTION 4: Event Simulation"
    echo "─────────────────────────────────────────"
    echo ""
    skip "Daemon not running, skipping event simulation"
    echo ""
fi

# ============================================================================
# SECTION 5: Event Listeners
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 5: Event Listeners"
echo "─────────────────────────────────────────"
echo ""

# Test 5.1: Check pacman database watchable
info "5.1 Checking packages listener..."
if [ -d /var/lib/pacman/local ]; then
    pass "Pacman database accessible"
else
    skip "Not an Arch system (pacman listener disabled)"
fi

# Test 5.2: Check config files watchable
info "5.2 Checking config listener..."
CONFIG_FILES_OK=true
for file in /etc/resolv.conf /etc/fstab /etc/hostname; do
    if [ ! -r "$file" ]; then
        CONFIG_FILES_OK=false
    fi
done

if [ "$CONFIG_FILES_OK" = true ]; then
    pass "Critical config files readable"
else
    skip "Some config files not accessible"
fi

# Test 5.3: Check mountinfo readable
info "5.3 Checking storage listener..."
if [ -r /proc/self/mountinfo ]; then
    pass "Mount state readable"
else
    fail "Cannot read /proc/self/mountinfo"
fi

# Test 5.4: Check network state
info "5.4 Checking network listener..."
if command -v ip &>/dev/null && ip link show &>/dev/null; then
    pass "Network state accessible"
else
    skip "iproute2 not available"
fi

echo ""

# ============================================================================
# SECTION 6: Logging and Telemetry
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 6: Logging and Telemetry"
echo "─────────────────────────────────────────"
echo ""

# Test 6.1: Check journal logs
info "6.1 Checking journal logs..."
if journalctl -u annad -n 1 &>/dev/null; then
    LOG_COUNT=$(journalctl -u annad --no-pager | wc -l)
    pass "Journal logs accessible ($LOG_COUNT lines)"
else
    fail "Cannot read journal logs"
fi

# Test 6.2: Check telemetry database
info "6.2 Checking telemetry database..."
if [ -f /var/lib/anna/telemetry.db ]; then
    DB_SIZE=$(stat -f%z /var/lib/anna/telemetry.db 2>/dev/null || stat -c%s /var/lib/anna/telemetry.db 2>/dev/null || echo 0)
    if [ "$DB_SIZE" -gt 0 ]; then
        pass "Telemetry database exists (${DB_SIZE} bytes)"
    else
        skip "Telemetry database empty"
    fi
else
    skip "Telemetry database not created yet"
fi

# Test 6.3: Check log directory writable
info "6.3 Checking log directory..."
if sudo -u anna test -w /var/log/anna; then
    pass "Log directory writable by anna user"
else
    fail "Log directory not writable by anna user"
fi

echo ""

# ============================================================================
# SUMMARY
# ============================================================================

echo "═════════════════════════════════════════"
echo "SUMMARY"
echo "═════════════════════════════════════════"
echo ""
echo "  Passed:  $PASS"
echo "  Failed:  $FAIL"
echo "  Skipped: $SKIP"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  • Check event stream:  annactl events"
    echo "  • Watch live events:   annactl watch"
    echo "  • Check capabilities:  annactl capabilities"
    echo "  • View alerts:         annactl alerts"
    echo ""
    exit 0
elif [ $FAIL -lt 5 ]; then
    echo -e "${YELLOW}⚠ Some tests failed, but system may still work${NC}"
    echo ""
    exit 1
else
    echo -e "${RED}✗ Critical failures detected${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  • Check daemon: sudo systemctl status annad"
    echo "  • View logs:    sudo journalctl -u annad -n 50"
    echo "  • Reinstall:    sudo ./scripts/install.sh"
    echo ""
    exit 2
fi
