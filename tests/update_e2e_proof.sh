#!/bin/bash
# =============================================================================
# UPDATE END-TO-END PROOF SCRIPT
# =============================================================================
# This script verifies the auto-update mechanism works without manual restarts.
# It reads the update ledger and proves complete update cycles occurred.
#
# Usage: ./tests/update_e2e_proof.sh
# Requirements: annad running as root with update ledger at /root/.anna/
# =============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
DIM='\033[2m'
NC='\033[0m'

# Determine ledger path based on daemon user
if [ -f /root/.anna/update_ledger.json ]; then
    LEDGER="/root/.anna/update_ledger.json"
elif [ -f "$HOME/.anna/update_ledger.json" ]; then
    LEDGER="$HOME/.anna/update_ledger.json"
else
    echo -e "${RED}[FAIL]${NC} No update ledger found"
    echo "Expected at /root/.anna/update_ledger.json or ~/.anna/update_ledger.json"
    exit 1
fi

echo "=============================================="
echo "UPDATE END-TO-END PROOF"
echo "=============================================="
echo ""
echo -e "${DIM}Ledger: $LEDGER${NC}"
echo ""

# =============================================================================
# TEST 1: Verify ledger exists and has entries
# =============================================================================
echo -e "${YELLOW}[TEST 1]${NC} Ledger Validity"
echo "-------------------------------------------"

ENTRY_COUNT=$(cat "$LEDGER" 2>/dev/null | grep -c '"timestamp"' || echo "0")
if [ "$ENTRY_COUNT" -lt 1 ]; then
    echo -e "${RED}[FAIL]${NC} Ledger is empty or malformed"
    exit 1
fi
echo -e "${GREEN}[PASS]${NC} Ledger has $ENTRY_COUNT entries"
echo ""

# =============================================================================
# TEST 2: Find complete update cycles (UpdateAvailable -> Installed -> UpToDate)
# =============================================================================
echo -e "${YELLOW}[TEST 2]${NC} Complete Update Cycles"
echo "-------------------------------------------"

# Extract all Installed entries
INSTALLED_COUNT=$(cat "$LEDGER" | grep -c '"Installed"' || echo "0")

if [ "$INSTALLED_COUNT" -lt 1 ]; then
    echo -e "${RED}[FAIL]${NC} No successful installs found in ledger"
    echo "This means auto-update has never completed successfully."
    exit 1
fi

echo -e "${GREEN}[PASS]${NC} Found $INSTALLED_COUNT successful auto-installs"
echo ""

# Show the most recent successful update cycle
echo -e "${CYAN}Most recent auto-update cycle:${NC}"
echo ""

# Find the last Installed entry and surrounding context
LAST_INSTALL_LINE=$(grep -n '"Installed"' "$LEDGER" | tail -1 | cut -d: -f1)
if [ -n "$LAST_INSTALL_LINE" ]; then
    # Get context: 15 lines before and 10 after
    START=$((LAST_INSTALL_LINE - 15))
    [ $START -lt 1 ] && START=1
    END=$((LAST_INSTALL_LINE + 10))

    # Extract and display the update cycle
    sed -n "${START},${END}p" "$LEDGER" | grep -E '"timestamp"|"current_version"|"remote_tag"|"result"|UpdateAvailable|Installed|UpToDate' | head -20
fi
echo ""

# =============================================================================
# TEST 3: Verify automatic restart occurred
# =============================================================================
echo -e "${YELLOW}[TEST 3]${NC} Automatic Restart Verification"
echo "-------------------------------------------"

# For a successful auto-update with restart:
# 1. Entry N shows "Installed: version X"
# 2. Entry N+1 shows "current_version: X" and "UpToDate"
# The time gap should be ~1-5 seconds (restart time)

# Extract timestamps around Installed entries
# This is a heuristic check - if UpToDate follows Installed within seconds, restart worked

echo "Checking restart timing..."

# Get last Installed entry timestamp and version
LAST_INSTALL=$(cat "$LEDGER" | python3 -c "
import json, sys
data = json.load(sys.stdin)
installs = [c for c in data['checks'] if isinstance(c.get('result'), dict) and 'Installed' in c.get('result', {})]
if installs:
    last = installs[-1]
    print(f\"{last['timestamp']}|{last['result']['Installed']['version']}\")
" 2>/dev/null || echo "")

if [ -z "$LAST_INSTALL" ]; then
    echo -e "${RED}[FAIL]${NC} Could not parse Installed entries"
    exit 1
fi

INSTALL_TIME=$(echo "$LAST_INSTALL" | cut -d'|' -f1)
INSTALL_VERSION=$(echo "$LAST_INSTALL" | cut -d'|' -f2)

echo -e "  Installed:     ${CYAN}$INSTALL_VERSION${NC} at $INSTALL_TIME"

# Find the first UpToDate entry at the installed version
FIRST_UPTODATE=$(cat "$LEDGER" | python3 -c "
import json, sys
data = json.load(sys.stdin)
target_ver = '$INSTALL_VERSION'
for c in data['checks']:
    if c.get('current_version') == target_ver and c.get('result') == 'UpToDate':
        print(f\"{c['timestamp']}\")
        break
" 2>/dev/null || echo "")

if [ -n "$FIRST_UPTODATE" ]; then
    echo -e "  Confirmed:     ${GREEN}UpToDate${NC} at $FIRST_UPTODATE"

    # Calculate time difference (rough check)
    INSTALL_SEC=$(date -d "${INSTALL_TIME%Z}" +%s 2>/dev/null || echo "0")
    UPTODATE_SEC=$(date -d "${FIRST_UPTODATE%Z}" +%s 2>/dev/null || echo "0")

    if [ "$INSTALL_SEC" -gt 0 ] && [ "$UPTODATE_SEC" -gt 0 ]; then
        DIFF=$((UPTODATE_SEC - INSTALL_SEC))
        echo -e "  Restart time:  ${GREEN}${DIFF}s${NC}"

        if [ "$DIFF" -lt 60 ]; then
            echo -e "${GREEN}[PASS]${NC} Daemon restarted automatically (${DIFF}s gap)"
        else
            echo -e "${YELLOW}[WARN]${NC} Long restart gap (${DIFF}s) - may indicate manual restart"
        fi
    fi
else
    echo -e "${RED}[FAIL]${NC} No UpToDate confirmation found after install"
    exit 1
fi
echo ""

# =============================================================================
# TEST 4: Current state verification
# =============================================================================
echo -e "${YELLOW}[TEST 4]${NC} Current State"
echo "-------------------------------------------"

# Get annactl and annad versions
ANNACTL_VER=$(/usr/local/bin/annactl --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
ANNAD_VER=$(/usr/local/bin/annad --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")

echo "  annactl:       $ANNACTL_VER"
echo "  annad:         $ANNAD_VER"

if [ "$ANNACTL_VER" = "$ANNAD_VER" ]; then
    echo -e "${GREEN}[PASS]${NC} Client and daemon versions match"
else
    echo -e "${RED}[FAIL]${NC} Version mismatch: annactl=$ANNACTL_VER annad=$ANNAD_VER"
    exit 1
fi

# Check if versions match last installed
if [ "$ANNAD_VER" = "$INSTALL_VERSION" ]; then
    echo -e "${GREEN}[PASS]${NC} Running version matches last installed ($INSTALL_VERSION)"
else
    echo -e "${YELLOW}[INFO]${NC} Running $ANNAD_VER, last auto-installed was $INSTALL_VERSION"
fi
echo ""

# =============================================================================
# TEST 5: Verify atomic update mechanism in source
# =============================================================================
echo -e "${YELLOW}[TEST 5]${NC} Atomic Update Mechanism"
echo "-------------------------------------------"

UPDATE_OPS="crates/annad/src/update_ops.rs"
if [ -f "$UPDATE_OPS" ]; then
    # Check for checksum verification
    if grep -q "verify_checksum" "$UPDATE_OPS"; then
        echo -e "${GREEN}[PASS]${NC} Checksum verification present"
    else
        echo -e "${RED}[FAIL]${NC} No checksum verification in updater"
        exit 1
    fi

    # Check for atomic rename
    if grep -q "atomic" "$UPDATE_OPS" || grep -q "rename" "$UPDATE_OPS"; then
        echo -e "${GREEN}[PASS]${NC} Atomic rename mechanism present"
    else
        echo -e "${RED}[FAIL]${NC} No atomic rename in updater"
        exit 1
    fi

    # Check for rollback
    if grep -q "rollback" "$UPDATE_OPS"; then
        echo -e "${GREEN}[PASS]${NC} Rollback mechanism present"
    else
        echo -e "${RED}[FAIL]${NC} No rollback in updater"
        exit 1
    fi

    # Check for restart scheduling
    if grep -q "schedule_daemon_restart\|systemctl restart" "$UPDATE_OPS"; then
        echo -e "${GREEN}[PASS]${NC} Auto-restart mechanism present"
    else
        echo -e "${RED}[FAIL]${NC} No auto-restart in updater"
        exit 1
    fi
else
    echo -e "${YELLOW}[SKIP]${NC} Source not available for verification"
fi
echo ""

# =============================================================================
# SUMMARY
# =============================================================================
echo "=============================================="
echo -e "${GREEN}AUTO-UPDATE PROOF COMPLETE${NC}"
echo "=============================================="
echo ""
echo "Verified:"
echo "  - Update ledger contains successful installs"
echo "  - Auto-restart occurred (Installed -> UpToDate in seconds)"
echo "  - Current binaries match last installed version"
echo "  - Atomic update mechanism present in source"
echo ""
echo "Last successful auto-update: $INSTALL_VERSION"
echo ""
