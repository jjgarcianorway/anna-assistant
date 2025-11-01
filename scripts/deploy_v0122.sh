#!/bin/bash
# Anna v0.12.2 Deployment Script
# Handles stop -> deploy -> start -> test workflow

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "╭───────────────────────────────────────────╮"
echo "│  Anna v0.12.2 Deployment Script          │"
echo "╰───────────────────────────────────────────╯"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}ERROR: This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Step 1: Stop daemon
echo -e "${YELLOW}→${NC} Stopping annad..."
systemctl stop annad || true
sleep 1

# Step 2: Deploy binaries
echo -e "${YELLOW}→${NC} Deploying binaries..."
if [[ ! -f target/release/annad ]]; then
    echo -e "${RED}ERROR: target/release/annad not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

if [[ ! -f target/release/annactl ]]; then
    echo -e "${RED}ERROR: target/release/annactl not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

cp -v target/release/annad /usr/local/bin/annad
cp -v target/release/annactl /usr/local/bin/annactl
chmod +x /usr/local/bin/annad
chmod +x /usr/local/bin/annactl

# Step 3: Reload systemd
echo -e "${YELLOW}→${NC} Reloading systemd..."
systemctl daemon-reload

# Step 4: Start daemon
echo -e "${YELLOW}→${NC} Starting annad..."
systemctl start annad

# Step 5: Wait for socket
echo -e "${YELLOW}→${NC} Waiting for socket..."
for i in {1..10}; do
    if [[ -S /run/anna/annad.sock ]]; then
        echo -e "${GREEN}✓${NC} Socket ready at /run/anna/annad.sock"
        break
    fi
    if [[ $i -eq 10 ]]; then
        echo -e "${RED}✗ Socket not ready after 10 seconds${NC}"
        echo "Check logs: journalctl -u annad -n 20"
        exit 1
    fi
    sleep 1
done

# Step 6: Verify version
echo -e "${YELLOW}→${NC} Verifying version..."
VERSION=$(/usr/local/bin/annactl --version | awk '{print $2}')
if [[ "$VERSION" == "0.12.2" ]]; then
    echo -e "${GREEN}✓${NC} Version: $VERSION"
else
    echo -e "${RED}✗ Unexpected version: $VERSION (expected 0.12.2)${NC}"
    exit 1
fi

# Step 7: Test basic connectivity
echo -e "${YELLOW}→${NC} Testing connectivity..."
if /usr/local/bin/annactl status >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} annactl status works"
else
    echo -e "${RED}✗ annactl status failed${NC}"
    exit 1
fi

# Step 8: Test new commands
echo -e "${YELLOW}→${NC} Testing new commands..."

# Test collect
if /usr/local/bin/annactl collect --limit 1 --json >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} annactl collect works"
else
    echo -e "${RED}✗ annactl collect failed${NC}"
    exit 1
fi

# Test classify
if /usr/local/bin/annactl classify --json >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} annactl classify works"
else
    echo -e "${RED}✗ annactl classify failed${NC}"
    exit 1
fi

# Test radar show
if /usr/local/bin/annactl radar show --json >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} annactl radar show works"
else
    echo -e "${RED}✗ annactl radar show failed${NC}"
    exit 1
fi

# Step 9: Check daemon status
echo -e "${YELLOW}→${NC} Checking daemon status..."
if systemctl is-active --quiet annad; then
    echo -e "${GREEN}✓${NC} Daemon is active"
else
    echo -e "${RED}✗ Daemon is not active${NC}"
    exit 1
fi

echo
echo -e "${GREEN}╭───────────────────────────────────────────╮${NC}"
echo -e "${GREEN}│  Deployment successful! ✓                 │${NC}"
echo -e "${GREEN}╰───────────────────────────────────────────╯${NC}"
echo
echo "Next steps:"
echo "  1. Run smoke test: sudo ./tests/smoke_v0122.sh"
echo "  2. Check logs: sudo journalctl -u annad -n 20"
echo "  3. Try commands:"
echo "       annactl collect --limit 1"
echo "       annactl classify"
echo "       annactl radar show"
echo

exit 0
