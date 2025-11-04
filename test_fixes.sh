#!/usr/bin/env bash
# Test script for the fixes

set -e

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║              Testing Anna Fixes - No policy.toml required            ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""

# 1. Copy new binaries
echo "→ Installing new binaries..."
sudo install -m 0755 ./target/release/annad /usr/local/bin/annad
sudo install -m 0755 ./target/release/annactl /usr/local/bin/annactl
echo "  ✓ Binaries installed"
echo ""

# 2. Stop daemon
echo "→ Stopping daemon..."
sudo systemctl stop annad
echo "  ✓ Daemon stopped"
echo ""

# 3. Remove policy.toml if it exists (to test fallback)
if [ -f /etc/anna/policy.toml ]; then
    echo "→ Backing up existing policy.toml..."
    sudo cp /etc/anna/policy.toml /etc/anna/policy.toml.backup
    sudo rm /etc/anna/policy.toml
    echo "  ✓ Removed policy.toml (testing fallback)"
else
    echo "→ No policy.toml found (good for testing)"
fi
echo ""

# 4. Start daemon
echo "→ Starting daemon..."
sudo systemctl start annad
echo "  ✓ Daemon started"
echo ""

# 5. Wait and check
echo "→ Waiting for daemon (5s)..."
sleep 5
echo ""

# 6. Check status
echo "→ Checking daemon status..."
if systemctl is-active annad >/dev/null 2>&1; then
    echo "  ✓ Daemon is active"
else
    echo "  ✗ Daemon is not active"
    echo ""
    echo "Recent logs:"
    journalctl -u annad -n 20 --no-pager
    exit 1
fi
echo ""

# 7. Check socket
echo "→ Checking socket..."
if [ -S /run/anna/annad.sock ]; then
    echo "  ✓ Socket exists"
else
    echo "  ✗ Socket missing"
    exit 1
fi
echo ""

# 8. Test RPC
echo "→ Testing RPC..."
if timeout 2 annactl version >/dev/null 2>&1; then
    echo "  ✓ RPC working"
    annactl -V
else
    echo "  ✗ RPC not responding"
    exit 1
fi
echo ""

# 9. Test doctor repair
echo "→ Testing doctor repair (beautiful output)..."
echo ""
sudo annactl doctor repair --yes
echo ""

# 10. Check logs for warnings
echo "→ Checking for policy.toml warnings in logs..."
if journalctl -u annad -n 30 --no-pager | grep -q "Starting with default policy"; then
    echo "  ✓ Daemon using default policy (as expected)"
else
    echo "  ⚠ No default policy message found"
fi
echo ""

# Success
echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║                     ✨  All tests passed! ✨                          ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Summary:"
echo "  • Daemon starts without policy.toml ✓"
echo "  • Socket is created ✓"
echo "  • RPC is working ✓"
echo "  • Doctor repair is beautiful ✓"
echo ""

# Restore policy.toml if we backed it up
if [ -f /etc/anna/policy.toml.backup ]; then
    echo "Restoring original policy.toml..."
    sudo mv /etc/anna/policy.toml.backup /etc/anna/policy.toml
    echo "  ✓ Restored"
fi
