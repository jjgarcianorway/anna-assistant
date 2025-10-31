#!/usr/bin/env bash
# Anna v0.11.0 Emergency Repair Script
# Fixes common installation issues: directory ownership, CAPABILITIES.toml, socket permissions
#
# Usage: sudo bash scripts/fix_v011_installation.sh

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Anna v0.11.0 Emergency Repair          │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "✗ This script must be run as root (use sudo)"
    exit 1
fi

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "→ Stopping annad service..."
systemctl stop annad || true
echo "✓ Service stopped"
echo ""

echo "→ Fixing directory ownership and permissions..."

# Create directories with correct ownership using install
install -d -m 0750 -o anna -g anna /var/lib/anna || true
install -d -m 0750 -o anna -g anna /var/log/anna || true
install -d -m 0755 -o root -g root /usr/lib/anna || true
install -d -m 0755 -o root -g root /etc/anna || true

# Fix ownership recursively (defensive)
chown -R anna:anna /var/lib/anna /var/log/anna
chmod 0750 /var/lib/anna /var/log/anna
echo "✓ Directory ownership: anna:anna for /var/lib/anna and /var/log/anna"
echo "✓ Directory permissions: 0750"
echo ""

echo "→ Installing/verifying CAPABILITIES.toml..."
if [ -f "$PROJECT_ROOT/etc/CAPABILITIES.toml" ]; then
    install -m 0644 -o root -g root "$PROJECT_ROOT/etc/CAPABILITIES.toml" /usr/lib/anna/CAPABILITIES.toml
    echo "✓ CAPABILITIES.toml installed from repo"
elif [ ! -f /usr/lib/anna/CAPABILITIES.toml ]; then
    echo "⚠ etc/CAPABILITIES.toml not found, creating minimal version..."
    cat > /usr/lib/anna/CAPABILITIES.toml <<'EOF'
[meta]
version = "0.11.0"
description = "Anna telemetry capability registry (minimal emergency version)"
EOF
    chmod 0644 /usr/lib/anna/CAPABILITIES.toml
    chown root:root /usr/lib/anna/CAPABILITIES.toml
    echo "✓ Minimal CAPABILITIES.toml created"
else
    echo "✓ CAPABILITIES.toml already exists"
fi
echo ""

echo "→ Reloading systemd configuration..."
systemctl daemon-reload
echo "✓ Systemd reloaded"
echo ""

echo "→ Restarting annad service..."
systemctl restart annad
sleep 3
echo "✓ Service restarted"
echo ""

echo "→ Verifying installation..."
echo ""

# Check daemon status
if systemctl is-active --quiet annad; then
    echo "✓ annad service: active (running)"
else
    echo "✗ annad service: not running"
    echo ""
    echo "Check logs:"
    echo "  sudo journalctl -u annad -n 20 --no-pager"
    exit 1
fi

# Check socket (wait up to 10 seconds)
SOCKET_FOUND=false
for i in {1..10}; do
    if [ -S /run/anna/annad.sock ]; then
        SOCKET_FOUND=true
        break
    fi
    sleep 1
done

if [ "$SOCKET_FOUND" = true ]; then
    echo "✓ RPC socket: /run/anna/annad.sock exists"
else
    echo "✗ RPC socket: not found after 10 seconds"
    echo ""
    echo "Check logs:"
    echo "  sudo journalctl -u annad -n 20 --no-pager"
    exit 1
fi

# Check directory ownership
VAR_LIB_OWNER=$(stat -c '%U:%G' /var/lib/anna 2>/dev/null || stat -f '%Su:%Sg' /var/lib/anna)
VAR_LOG_OWNER=$(stat -c '%U:%G' /var/log/anna 2>/dev/null || stat -f '%Su:%Sg' /var/log/anna)

if [ "$VAR_LIB_OWNER" = "anna:anna" ]; then
    echo "✓ /var/lib/anna ownership: anna:anna"
else
    echo "⚠ /var/lib/anna ownership: $VAR_LIB_OWNER (expected anna:anna)"
fi

if [ "$VAR_LOG_OWNER" = "anna:anna" ]; then
    echo "✓ /var/log/anna ownership: anna:anna"
else
    echo "⚠ /var/log/anna ownership: $VAR_LOG_OWNER (expected anna:anna)"
fi

# Check CAPABILITIES.toml
if [ -f /usr/lib/anna/CAPABILITIES.toml ]; then
    echo "✓ /usr/lib/anna/CAPABILITIES.toml: present"
else
    echo "✗ /usr/lib/anna/CAPABILITIES.toml: missing"
fi

# Test annactl connectivity
echo ""
echo "→ Testing annactl connectivity..."
if timeout 5 annactl status &>/dev/null; then
    echo "✓ annactl can communicate with daemon"
else
    echo "✗ annactl cannot reach daemon"
    echo ""
    echo "Troubleshooting:"
    echo "  - Check logs: sudo journalctl -u annad -n 30"
    echo "  - Check socket: ls -la /run/anna/annad.sock"
    echo "  - Check DB: ls -la /var/lib/anna/telemetry.db"
    exit 1
fi

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  ✓ Repair Complete                      │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Verification commands:"
echo "  annactl status"
echo "  annactl events --limit 5"
echo "  systemctl status annad"
echo ""
