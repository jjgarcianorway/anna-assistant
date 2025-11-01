#!/usr/bin/env bash
# Anna v0.12.0 - Persistence Test
# Verify socket persistence across 5 daemon restarts

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "╭────────────────────────────────────────╮"
echo "│  Anna v0.12.0 Persistence Test         │"
echo "│  5 restarts, verify socket each time   │"
echo "╰────────────────────────────────────────╯"
echo ""

# Check if running as root/sudo
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}✗${NC} This test requires sudo/root privileges"
   echo "  Run: sudo ./tests/persistence_v012.sh"
   exit 1
fi

failures=0

for i in {1..5}; do
    echo "Restart $i/5:"

    # Restart daemon
    echo -n "  ⏳ Restarting daemon ... "
    if systemctl restart annad 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        ((failures++))
        continue
    fi

    # Wait for startup
    echo -n "  ⏳ Waiting for startup ... "
    sleep 2
    echo -e "${GREEN}✓${NC}"

    # Check socket exists
    echo -n "  ⏳ Socket exists ... "
    if [[ -S /run/anna/annad.sock ]]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC} Socket missing!"
        ((failures++))
        continue
    fi

    # Check socket permissions
    echo -n "  ⏳ Socket permissions ... "
    perms=$(stat -c "%a %U:%G" /run/anna/annad.sock 2>/dev/null || echo "")
    if [[ "$perms" == "770 anna:anna" ]]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠${NC} Got: $perms (expected: 770 anna:anna)"
    fi

    # Check RPC works
    echo -n "  ⏳ RPC connectivity ... "
    if timeout 3s annactl status >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC} RPC failed!"
        ((failures++))
        continue
    fi

    # Check socket ready in logs
    echo -n "  ⏳ Socket ready logged ... "
    if journalctl -u annad --since "10 seconds ago" --no-pager | grep -q "RPC socket ready"; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        ((failures++))
    fi

    echo ""
done

# Count total socket ready messages
echo "Final verification:"
echo -n "  ⏳ Counting socket ready messages ... "
count=$(journalctl -u annad -b --no-pager 2>/dev/null | grep -c "RPC socket ready" || echo "0")

if [[ $count -ge 5 ]]; then
    echo -e "${GREEN}✓${NC} Found $count messages (≥5 required)"
else
    echo -e "${RED}✗${NC} Only found $count messages (expected ≥5)"
    ((failures++))
fi

# Summary
echo ""
echo "╭────────────────────────────────────────╮"
if [[ $failures -eq 0 ]]; then
    echo -e "│  ${GREEN}✓ Persistence test passed!${NC}            │"
    echo "│    Socket survived 5 restarts         │"
    echo "╰────────────────────────────────────────╯"
    exit 0
else
    echo -e "│  ${RED}✗ $failures checks failed${NC}                 │"
    echo "╰────────────────────────────────────────╯"
    exit 1
fi
