#!/usr/bin/env bash
# Upgrade script for v0.12.6-pre
# Installs new binaries and restarts daemon

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [ -n "${NO_COLOR:-}" ]; then
    RED=''; GREEN=''; YELLOW=''; BLUE=''; NC=''
fi

echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
echo -e "${BLUE}│  Anna Upgrade to v0.12.6-pre           │${NC}"
echo -e "${BLUE}│  Daemon Restart Fix                    │${NC}"
echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
echo ""

# Check if running with sudo/root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}✗${NC} This script must be run with sudo"
    echo ""
    echo "Usage: sudo $0"
    exit 1
fi

# Verify binaries exist
if [ ! -f "./target/release/annad" ] || [ ! -f "./target/release/annactl" ]; then
    echo -e "${RED}✗${NC} Binaries not found in ./target/release/"
    echo ""
    echo "Build first with: cargo build --release"
    exit 1
fi

# Show current state
echo "Current state:"
CURRENT_DISK=$(/usr/local/bin/annactl --version 2>&1 | awk '{print $2}' || echo "unknown")
CURRENT_DAEMON=$(timeout 2 annactl --version 2>&1 | awk '{print $2}' || echo "not responding")
NEW_VERSION=$(./target/release/annactl --version 2>&1 | awk '{print $2}' || echo "unknown")

echo -e "  Installed: ${YELLOW}v${CURRENT_DISK}${NC}"
echo -e "  Running:   ${YELLOW}v${CURRENT_DAEMON}${NC}"
echo -e "  New:       ${GREEN}v${NEW_VERSION}${NC}"
echo ""

# Confirm upgrade
read -p "Proceed with upgrade? [Y/n] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]?$ ]]; then
    echo "Upgrade cancelled."
    exit 0
fi
echo ""

# Step 1: Install new binaries
echo -e "${BLUE}→${NC} Installing new binaries..."
install -m 755 ./target/release/annad /usr/local/bin/
install -m 755 ./target/release/annactl /usr/local/bin/
echo -e "${GREEN}✓${NC} Binaries installed to /usr/local/bin/"

# Verify installation
INSTALLED_VERSION=$(/usr/local/bin/annactl --version 2>&1 | awk '{print $2}')
echo -e "${GREEN}✓${NC} Verified: v${INSTALLED_VERSION}"
echo ""

# Step 2: Restart daemon
echo -e "${BLUE}→${NC} Restarting daemon..."
if systemctl restart annad; then
    echo -e "${GREEN}✓${NC} Daemon restarted"
else
    echo -e "${RED}✗${NC} Daemon restart failed"
    systemctl status annad --no-pager -l | head -20
    exit 1
fi

# Step 3: Wait for initialization
echo -e "${BLUE}→${NC} Waiting for daemon to initialize..."
WAITED=0
MAX_WAIT=10
while [ $WAITED -lt $MAX_WAIT ]; do
    if [ -S /run/anna/annad.sock ]; then
        break
    fi
    sleep 1
    WAITED=$((WAITED + 1))
done

if [ ! -S /run/anna/annad.sock ]; then
    echo -e "${RED}✗${NC} Socket not created after ${MAX_WAIT}s"
    exit 1
fi

sleep 2  # Give daemon a moment to fully initialize

# Step 4: Verify version
echo -e "${BLUE}→${NC} Verifying daemon version..."
RUNNING_VERSION=$(timeout 5 annactl --version 2>&1 | awk '{print $2}' || echo "timeout")

if [ "$RUNNING_VERSION" = "$INSTALLED_VERSION" ]; then
    echo -e "${GREEN}✓${NC} Version verified: v${RUNNING_VERSION}"
else
    echo -e "${RED}✗${NC} Version mismatch: installed=v${INSTALLED_VERSION}, running=v${RUNNING_VERSION}"
    exit 1
fi
echo ""

# Step 5: Test RPC
echo -e "${BLUE}→${NC} Testing RPC..."
if timeout 5 annactl status &>/dev/null; then
    echo -e "${GREEN}✓${NC} RPC working (annactl status)"
else
    echo -e "${RED}✗${NC} RPC timeout"
    exit 1
fi

if timeout 5 annactl storage --help &>/dev/null; then
    echo -e "${GREEN}✓${NC} Storage command available"
else
    echo -e "${RED}✗${NC} Storage command not available"
    exit 1
fi

# Success
echo ""
echo -e "${GREEN}╭─────────────────────────────────────────╮${NC}"
echo -e "${GREEN}│  ✓ Upgrade Complete!                    │${NC}"
echo -e "${GREEN}╰─────────────────────────────────────────╯${NC}"
echo ""
echo "Upgraded to: v${RUNNING_VERSION}"
echo ""
echo "Next steps:"
echo "  1. Run validation: ./scripts/validate_v0126.sh"
echo "  2. Run smoke tests: ./tests/verify_v0122.sh"
echo "  3. Test storage: annactl storage btrfs show"
echo ""
