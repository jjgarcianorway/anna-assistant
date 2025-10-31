#!/usr/bin/env bash
# Anna v0.11.0 - Socket Persistence Verification Script
# Tests socket creation across multiple daemon restarts

set -euo pipefail

# Configuration
SOCKET_PATH="/run/anna/annad.sock"
LOG_FILE="/tmp/anna_socket_persistence.log"
ITERATIONS=5
WAIT_SECONDS=5
EXPECTED_OWNER="anna:anna"
EXPECTED_MODE="0660"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Initialize log
echo "╭─────────────────────────────────────────────────────────────────" | tee "$LOG_FILE"
echo "│  Anna Socket Persistence Verification" | tee -a "$LOG_FILE"
echo "│  $(date)" | tee -a "$LOG_FILE"
echo "╰─────────────────────────────────────────────────────────────────" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

PASS=0
FAIL=0

for i in $(seq 1 $ITERATIONS); do
    echo -e "${YELLOW}→${NC} Iteration $i/$ITERATIONS: Restarting annad..." | tee -a "$LOG_FILE"

    # Restart daemon
    if ! sudo systemctl restart annad 2>&1 | tee -a "$LOG_FILE"; then
        echo -e "${RED}✗${NC} Failed to restart daemon" | tee -a "$LOG_FILE"
        ((FAIL++))
        continue
    fi

    # Wait for socket (up to WAIT_SECONDS)
    SOCKET_FOUND=false
    for wait in $(seq 1 $WAIT_SECONDS); do
        if [ -S "$SOCKET_PATH" ]; then
            SOCKET_FOUND=true
            echo "  Socket appeared after ${wait}s" | tee -a "$LOG_FILE"
            break
        fi
        sleep 1
    done

    if [ "$SOCKET_FOUND" != true ]; then
        echo -e "${RED}✗${NC} Socket not found after ${WAIT_SECONDS}s" | tee -a "$LOG_FILE"
        ((FAIL++))
        continue
    fi

    # Verify ownership and permissions
    OWNER=$(stat -c "%U:%G" "$SOCKET_PATH")
    MODE=$(stat -c "%a" "$SOCKET_PATH")

    if [ "$OWNER" != "$EXPECTED_OWNER" ]; then
        echo -e "${RED}✗${NC} Wrong owner: $OWNER (expected: $EXPECTED_OWNER)" | tee -a "$LOG_FILE"
        ((FAIL++))
        continue
    fi

    if [ "$MODE" != "$EXPECTED_MODE" ]; then
        echo -e "${RED}✗${NC} Wrong mode: $MODE (expected: $EXPECTED_MODE)" | tee -a "$LOG_FILE"
        ((FAIL++))
        continue
    fi

    echo -e "${GREEN}✓${NC} Socket verified: $OWNER $MODE" | tee -a "$LOG_FILE"
    ((PASS++))
    echo "" | tee -a "$LOG_FILE"
done

# Check journal logs for "RPC socket ready" messages
echo "─────────────────────────────────────────────────────────────────" | tee -a "$LOG_FILE"
echo -e "${YELLOW}→${NC} Checking journal logs..." | tee -a "$LOG_FILE"

READY_COUNT=$(sudo journalctl -u annad --since "5 min ago" 2>/dev/null | grep -c "RPC socket ready" || echo "0")
echo "  Found $READY_COUNT 'RPC socket ready' messages (expected: $ITERATIONS)" | tee -a "$LOG_FILE"

if [ "$READY_COUNT" -lt "$ITERATIONS" ]; then
    echo -e "${YELLOW}⚠${NC} Fewer socket ready messages than expected" | tee -a "$LOG_FILE"
fi

# Check for "readonly database" errors
READONLY_COUNT=$(sudo journalctl -u annad --since "5 min ago" 2>/dev/null | grep -c "readonly database" || echo "0")
if [ "$READONLY_COUNT" -gt 0 ]; then
    echo -e "${RED}✗${NC} Found $READONLY_COUNT 'readonly database' errors" | tee -a "$LOG_FILE"
    ((FAIL++))
else
    echo -e "${GREEN}✓${NC} No 'readonly database' errors found" | tee -a "$LOG_FILE"
fi

echo "" | tee -a "$LOG_FILE"

# Summary
echo "═════════════════════════════════════════════════════════════════" | tee -a "$LOG_FILE"
echo "SUMMARY" | tee -a "$LOG_FILE"
echo "═════════════════════════════════════════════════════════════════" | tee -a "$LOG_FILE"
echo "  Passed:  $PASS/$ITERATIONS" | tee -a "$LOG_FILE"
echo "  Failed:  $FAIL/$ITERATIONS" | tee -a "$LOG_FILE"
echo "  Log Messages: $READY_COUNT/$ITERATIONS" | tee -a "$LOG_FILE"
echo "  Storage Errors: $READONLY_COUNT" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ Socket persistence: $PASS/$ITERATIONS${NC}" | tee -a "$LOG_FILE"
    echo -e "${GREEN}✓ Permissions repaired automatically${NC}" | tee -a "$LOG_FILE"
    echo -e "${GREEN}✓ Storage writable${NC}" | tee -a "$LOG_FILE"
    echo "" | tee -a "$LOG_FILE"
    echo "Log saved to: $LOG_FILE" | tee -a "$LOG_FILE"
    exit 0
else
    echo -e "${RED}✗ Socket persistence failed: $FAIL failures${NC}" | tee -a "$LOG_FILE"
    echo "" | tee -a "$LOG_FILE"
    echo "Log saved to: $LOG_FILE" | tee -a "$LOG_FILE"
    echo "Check: sudo journalctl -u annad -n 50" | tee -a "$LOG_FILE"
    exit 1
fi
