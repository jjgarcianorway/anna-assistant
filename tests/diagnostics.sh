#!/usr/bin/env bash
# Anna System Diagnostics
# Simple health check for installed Anna system

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}╭─────────────────────────────────────────╮${RESET}"
echo -e "${BLUE}│  Anna System Diagnostics                │${RESET}"
echo -e "${BLUE}╰─────────────────────────────────────────╯${RESET}"
echo ""

PASS=0
FAIL=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${RESET} $1"
    ((PASS++))
}

fail() {
    echo -e "${RED}✗${RESET} $1"
    ((FAIL++))
}

warn() {
    echo -e "${YELLOW}⚠${RESET} $1"
}

# Test 1: Binaries installed
echo "→ Checking binaries..."
if command -v annactl &>/dev/null && command -v annad &>/dev/null; then
    VERSION=$(annactl --version 2>&1 | head -1 || echo "unknown")
    pass "Binaries installed: $VERSION"
else
    fail "Binaries not found in PATH"
fi
echo ""

# Test 2: Systemd service
echo "→ Checking systemd service..."
if systemctl list-unit-files | grep -q annad.service; then
    if systemctl is-active --quiet annad; then
        pass "Daemon is running"
    elif systemctl is-enabled --quiet annad; then
        warn "Daemon is enabled but not running"
        echo "   Try: sudo systemctl start annad"
    else
        fail "Daemon is not enabled"
        echo "   Try: sudo systemctl enable --now annad"
    fi
else
    fail "Service not installed"
fi
echo ""

# Test 3: Socket
echo "→ Checking RPC socket..."
if [ -S /run/anna/annad.sock ]; then
    pass "Socket exists: /run/anna/annad.sock"
else
    fail "Socket not found (daemon may not be running)"
fi
echo ""

# Test 4: Configuration
echo "→ Checking configuration..."
if [ -f /etc/anna/config.toml ]; then
    pass "Config exists: /etc/anna/config.toml"
else
    fail "Config not found"
fi

if [ -f /usr/lib/anna/CAPABILITIES.toml ]; then
    pass "Capabilities registry installed"
else
    fail "CAPABILITIES.toml missing (run: ./scripts/fix-capabilities.sh)"
fi
echo ""

# Test 5: Directories
echo "→ Checking directories..."
for dir in /var/lib/anna /var/log/anna /run/anna; do
    if [ -d "$dir" ]; then
        pass "Directory exists: $dir"
    else
        fail "Directory missing: $dir"
    fi
done
echo ""

# Test 6: annactl commands
echo "→ Testing annactl commands (without daemon)..."
if annactl doctor pre &>/dev/null; then
    pass "annactl doctor pre works"
else
    warn "annactl doctor pre failed"
fi

if annactl --version &>/dev/null; then
    pass "annactl --version works"
else
    fail "annactl --version failed"
fi
echo ""

# Test 7: Daemon connection (if running)
if systemctl is-active --quiet annad; then
    echo "→ Testing daemon connection..."
    if annactl status &>/dev/null; then
        pass "annactl status connects to daemon"
    else
        fail "Cannot connect to daemon"
    fi
    echo ""
fi

# Summary
echo "╭─────────────────────────────────────────╮"
if [ $FAIL -eq 0 ]; then
    echo -e "│ ${GREEN}✓ All checks passed${RESET} ($PASS passed)        │"
else
    echo -e "│ ${RED}✗ Some checks failed${RESET} ($FAIL failed, $PASS passed) │"
fi
echo "╰─────────────────────────────────────────╯"
echo ""

# Recommendations
if [ $FAIL -gt 0 ]; then
    echo "Recommendations:"
    echo ""
    if ! command -v annactl &>/dev/null; then
        echo "  • Install Anna: ./scripts/install.sh"
    fi
    if [ ! -f /usr/lib/anna/CAPABILITIES.toml ]; then
        echo "  • Fix missing file: ./scripts/fix-capabilities.sh"
    fi
    if ! systemctl is-active --quiet annad; then
        echo "  • Start daemon: sudo systemctl start annad"
        echo "  • Check logs: sudo journalctl -u annad -n 50"
    fi
    echo ""
fi

exit $FAIL
