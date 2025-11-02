#!/usr/bin/env bash
# Validation script for v0.12.6-pre daemon restart fix
# This script validates that the daemon restart logic works correctly

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if NO_COLOR is set
if [ -n "${NO_COLOR:-}" ]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
echo -e "${BLUE}│  Anna v0.12.6-pre Validation           │${NC}"
echo -e "${BLUE}│  Daemon Restart Fix Verification       │${NC}"
echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
echo ""

FAILED=0
PASSED=0

# Function to print test result
test_result() {
    local name="$1"
    local status="$2"
    local details="${3:-}"

    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✓${NC} $name"
        [ -n "$details" ] && echo -e "  ${details}"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $name"
        [ -n "$details" ] && echo -e "  ${RED}${details}${NC}"
        ((FAILED++))
    fi
}

# Test 1: Check binary versions on disk
echo "Test 1: Checking installed binary versions..."
ANNAD_VERSION=$(/usr/local/bin/annad --version 2>&1 | grep -oP 'v\d+\.\d+\.\d+(-\w+)?' || echo "unknown")
ANNACTL_DISK_VERSION=$(/usr/local/bin/annactl --version 2>&1 | awk '{print $2}' || echo "unknown")

if [[ "$ANNACTL_DISK_VERSION" == "0.12.6-pre" ]]; then
    test_result "Installed annactl version" "PASS" "v$ANNACTL_DISK_VERSION"
else
    test_result "Installed annactl version" "FAIL" "Expected 0.12.6-pre, got $ANNACTL_DISK_VERSION"
fi

# Test 2: Check daemon process
echo ""
echo "Test 2: Checking daemon process..."
if systemctl is-active --quiet annad; then
    DAEMON_PID=$(pgrep -x annad || echo "")
    if [ -n "$DAEMON_PID" ]; then
        DAEMON_START=$(ps -p "$DAEMON_PID" -o lstart= || echo "unknown")
        test_result "Daemon running" "PASS" "PID: $DAEMON_PID, Started: $DAEMON_START"
    else
        test_result "Daemon running" "FAIL" "systemctl says active but no PID found"
    fi
else
    test_result "Daemon running" "FAIL" "Daemon is not active"
fi

# Test 3: Check daemon version via RPC
echo ""
echo "Test 3: Checking daemon version via RPC..."
DAEMON_VERSION=$(timeout 5 annactl --version 2>&1 | awk '{print $2}' || echo "timeout")

if [[ "$DAEMON_VERSION" == "0.12.6-pre" ]]; then
    test_result "Daemon RPC version" "PASS" "v$DAEMON_VERSION"
elif [[ "$DAEMON_VERSION" == "timeout" ]]; then
    test_result "Daemon RPC version" "FAIL" "RPC timeout (daemon not responding)"
else
    test_result "Daemon RPC version" "FAIL" "Version mismatch: disk=0.12.6-pre, running=$DAEMON_VERSION"
fi

# Test 4: Version alignment check
echo ""
echo "Test 4: Checking version alignment..."
if [[ "$ANNACTL_DISK_VERSION" == "$DAEMON_VERSION" ]]; then
    test_result "Version alignment" "PASS" "Disk and daemon both at v$DAEMON_VERSION"
else
    test_result "Version alignment" "FAIL" "Disk: v$ANNACTL_DISK_VERSION, Daemon: v$DAEMON_VERSION (restart needed)"
fi

# Test 5: Socket exists and is accessible
echo ""
echo "Test 5: Checking Unix socket..."
if [ -S /run/anna/annad.sock ]; then
    SOCKET_PERMS=$(stat -c '%U:%G %a' /run/anna/annad.sock)
    test_result "Socket file exists" "PASS" "$SOCKET_PERMS"
else
    test_result "Socket file exists" "FAIL" "/run/anna/annad.sock not found"
fi

# Test 6: Basic RPC functionality
echo ""
echo "Test 6: Testing basic RPC..."
if timeout 5 annactl status &>/dev/null; then
    test_result "RPC: annactl status" "PASS" "Daemon responding"
else
    test_result "RPC: annactl status" "FAIL" "Timeout or error"
fi

# Test 7: Storage command availability
echo ""
echo "Test 7: Testing storage command..."
if timeout 5 annactl storage --help &>/dev/null; then
    test_result "RPC: annactl storage" "PASS" "Storage command recognized"
else
    STORAGE_ERROR=$(timeout 5 annactl storage --help 2>&1 || echo "timeout")
    if [[ "$STORAGE_ERROR" =~ "unrecognized subcommand" ]]; then
        test_result "RPC: annactl storage" "FAIL" "Old daemon - storage command not available (restart needed)"
    else
        test_result "RPC: annactl storage" "FAIL" "$STORAGE_ERROR"
    fi
fi

# Test 8: Installer restart logic
echo ""
echo "Test 8: Checking installer restart logic..."
if grep -q 'DAEMON_WAS_RUNNING' scripts/install.sh && \
   grep -q 'systemctl restart annad' scripts/install.sh && \
   grep -q 'Version verified' scripts/install.sh; then
    test_result "Installer restart logic" "PASS" "Upgrade detection and restart code present"
else
    test_result "Installer restart logic" "FAIL" "Missing restart logic in scripts/install.sh"
fi

# Summary
echo ""
echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
echo -e "${BLUE}│  Validation Summary                     │${NC}"
echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
echo ""
echo -e "  Passed: ${GREEN}${PASSED}${NC}"
echo -e "  Failed: ${RED}${FAILED}${NC}"
echo ""

# Recommendations
if [ $FAILED -gt 0 ]; then
    echo -e "${YELLOW}⚠ Issues detected${NC}"
    echo ""

    # Check if version mismatch is the main issue
    if [[ "$ANNACTL_DISK_VERSION" == "0.12.6-pre" ]] && [[ "$DAEMON_VERSION" != "0.12.6-pre" ]]; then
        echo "The daemon needs to be restarted to load new binaries:"
        echo ""
        echo -e "${YELLOW}  sudo systemctl restart annad${NC}"
        echo "  sleep 3"
        echo "  annactl status"
        echo ""
    fi

    exit 1
else
    echo -e "${GREEN}✓ All validations passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run full smoke tests: ./tests/verify_v0122.sh"
    echo "  2. Run Btrfs tests: ./tests/arch_btrfs_smoke.sh"
    echo "  3. Proceed to v0.12.7 milestone"
    echo ""
    exit 0
fi
