#!/usr/bin/env bash
# Anna v0.11.0 - Persistence and Recovery Test
# Tests socket persistence across restarts and permission recovery

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

echo "╭─────────────────────────────────────────────────────────────────"
echo "│  Anna v0.11.0 Persistence & Recovery Test"
echo "╰─────────────────────────────────────────────────────────────────"
echo ""

# ============================================================================
# SECTION 1: Socket Persistence Test
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 1: Socket Persistence"
echo "─────────────────────────────────────────"
echo ""

info "1.1 Running socket persistence verification..."
if ./scripts/verify_socket_persistence.sh; then
    pass "Socket persistence: 5/5 restarts successful"
else
    fail "Socket persistence verification failed"
fi
echo ""

# ============================================================================
# SECTION 2: Journal Log Verification
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 2: Journal Log Verification"
echo "─────────────────────────────────────────"
echo ""

info "2.1 Checking for 'RPC socket ready' messages..."
SOCKET_READY_COUNT=$(sudo journalctl -u annad --since "5 min ago" --no-pager | grep -c "RPC socket ready: /run/anna/annad.sock" || true)

if [ "$SOCKET_READY_COUNT" -ge 5 ]; then
    pass "Found $SOCKET_READY_COUNT 'RPC socket ready' messages (≥5)"
else
    fail "Found only $SOCKET_READY_COUNT 'RPC socket ready' messages (expected ≥5)"
fi

info "2.2 Checking for 'readonly database' errors..."
READONLY_COUNT=$(sudo journalctl -u annad --since "5 min ago" --no-pager | grep -c "readonly database" || true)

if [ "$READONLY_COUNT" -eq 0 ]; then
    pass "No 'readonly database' errors found"
else
    fail "Found $READONLY_COUNT 'readonly database' errors (expected 0)"
fi
echo ""

# ============================================================================
# SECTION 3: Permission Loss Recovery Simulation
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 3: Permission Recovery Simulation"
echo "─────────────────────────────────────────"
echo ""

info "3.1 Simulating permission loss on /var/lib/anna..."
# Save original permissions
ORIG_MODE=$(stat -c "%a" /var/lib/anna)
info "   Original mode: $ORIG_MODE"

# Change to restrictive permissions
sudo chmod 0550 /var/lib/anna
info "   Changed mode to: 0550 (read-only)"

# Restart daemon (should fail or log errors)
info "3.2 Restarting daemon with broken permissions..."
sudo systemctl restart annad || true
sleep 2

# Check for permission errors in logs
PERM_ERROR=$(sudo journalctl -u annad --since "10 sec ago" --no-pager | grep -E "(readonly|permission denied|not writable)" -i || true)

if [ -n "$PERM_ERROR" ]; then
    pass "Detected permission error as expected"
    info "   Error: $(echo "$PERM_ERROR" | head -1 | cut -c1-80)..."
else
    fail "No permission error detected (should have failed)"
fi

# Restore permissions
info "3.3 Restoring correct permissions (0750)..."
sudo chmod 0750 /var/lib/anna
sudo chown -R anna:anna /var/lib/anna

# Restart daemon (should succeed)
info "3.4 Restarting daemon with fixed permissions..."
sudo systemctl restart annad
sleep 3

# Verify socket appears
if [ -S /run/anna/annad.sock ]; then
    pass "Socket created after permission fix"
else
    fail "Socket not created after permission fix"
fi

# Check for recovery in logs
RECOVERY_LOG=$(sudo journalctl -u annad --since "10 sec ago" --no-pager | grep "RPC socket ready" || true)

if [ -n "$RECOVERY_LOG" ]; then
    pass "Recovery confirmed: daemon started successfully"
else
    fail "No recovery log found"
fi
echo ""

# ============================================================================
# SECTION 4: Doctor Repair Validation
# ============================================================================

echo "─────────────────────────────────────────"
echo "SECTION 4: Doctor Repair Validation"
echo "─────────────────────────────────────────"
echo ""

info "4.1 Running doctor post-check..."
if annactl doctor post --verbose > /tmp/doctor_post.log 2>&1; then
    pass "Doctor post-check passed"

    # Check for storage writable
    if grep -q "Database.*is writable" /tmp/doctor_post.log; then
        pass "Storage confirmed writable"
    else
        fail "Storage not confirmed writable"
    fi
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 12 ]; then
        pass "Doctor post-check passed with warnings (exit 12)"
    else
        fail "Doctor post-check failed (exit $EXIT_CODE)"
    fi
fi
echo ""

# ============================================================================
# SUMMARY
# ============================================================================

echo "═════════════════════════════════════════════════════════════════"
echo "SUMMARY"
echo "═════════════════════════════════════════════════════════════════"
echo "  Passed:  $PASS"
echo "  Failed:  $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ All persistence and recovery tests passed${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some persistence and recovery tests failed${NC}"
    echo ""
    exit 1
fi
