#!/usr/bin/env bash
# Anna v0.11.0 - Installation Fix Script
# Run as root or with working sudo to fix permission and directory issues

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Anna v0.11.0 Installation Repair      │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "✗ This script must be run as root"
    echo "  Try: sudo bash $0"
    exit 1
fi

echo "→ Step 1: Stop daemon if running..."
systemctl stop annad || true
echo "✓ Daemon stopped"
echo ""

echo "→ Step 2: Create and fix directory permissions..."

# Create directories with correct permissions
mkdir -p /var/lib/anna
mkdir -p /var/log/anna
mkdir -p /run/anna
mkdir -p /etc/anna
mkdir -p /usr/lib/anna

# Set ownership to anna:anna
chown anna:anna /var/lib/anna
chown anna:anna /var/log/anna
chown anna:anna /run/anna
chown root:anna /etc/anna
chown root:root /usr/lib/anna

# Set permissions
chmod 0750 /var/lib/anna
chmod 0750 /var/log/anna
chmod 0750 /run/anna
chmod 0755 /etc/anna
chmod 0755 /usr/lib/anna

echo "✓ Directories created with correct permissions"
echo ""

echo "→ Step 3: Fix existing database if present..."
if [ -f /var/lib/anna/telemetry.db ]; then
    chown anna:anna /var/lib/anna/telemetry.db
    chmod 0640 /var/lib/anna/telemetry.db
    echo "✓ Database permissions fixed"
else
    echo "○ No existing database (will be created on first run)"
fi
echo ""

echo "→ Step 4: Install CAPABILITIES.toml..."
if [ -f etc/CAPABILITIES.toml ]; then
    cp etc/CAPABILITIES.toml /usr/lib/anna/
    chown root:root /usr/lib/anna/CAPABILITIES.toml
    chmod 0644 /usr/lib/anna/CAPABILITIES.toml
    echo "✓ CAPABILITIES.toml installed"
elif [ -f /home/lhoqvso/anna-assistant/etc/CAPABILITIES.toml ]; then
    cp /home/lhoqvso/anna-assistant/etc/CAPABILITIES.toml /usr/lib/anna/
    chown root:root /usr/lib/anna/CAPABILITIES.toml
    chmod 0644 /usr/lib/anna/CAPABILITIES.toml
    echo "✓ CAPABILITIES.toml installed"
else
    echo "⚠ CAPABILITIES.toml not found in project"
    echo "  Creating minimal version..."
    cat > /usr/lib/anna/CAPABILITIES.toml <<'EOF'
[meta]
version = "0.11.0"
description = "Anna telemetry capability registry"

[[modules]]
name = "lm_sensors"
description = "Temperature and voltage monitoring"
category = "sensors"
required = true

[[modules.deps]]
binaries = ["sensors"]
packages = ["lm_sensors"]

[[modules]]
name = "iproute2"
description = "Network interface monitoring"
category = "net"
required = true

[[modules.deps]]
binaries = ["ip"]
packages = ["iproute2"]
EOF
    chown root:root /usr/lib/anna/CAPABILITIES.toml
    chmod 0644 /usr/lib/anna/CAPABILITIES.toml
    echo "✓ Minimal CAPABILITIES.toml created"
fi
echo ""

echo "→ Step 5: Verify systemd service configuration..."
if grep -q "RuntimeDirectory=anna" /etc/systemd/system/annad.service; then
    echo "✓ RuntimeDirectory configured correctly"
else
    echo "⚠ RuntimeDirectory not set, adding..."
    # This shouldn't happen, but add it if missing
    sed -i '/\[Service\]/a RuntimeDirectory=anna\nRuntimeDirectoryMode=0750' /etc/systemd/system/annad.service
fi
echo ""

echo "→ Step 6: Reload systemd and restart daemon..."
systemctl daemon-reload
systemctl restart annad
echo "✓ Daemon restarted"
echo ""

echo "→ Step 7: Wait for daemon to initialize..."
sleep 5
echo ""

echo "→ Step 8: Verify daemon is running..."
if systemctl is-active --quiet annad; then
    echo "✓ Daemon is running"

    # Check if socket exists
    if [ -S /run/anna/annad.sock ]; then
        echo "✓ RPC socket created"
        stat -c '  Permissions: %a %U:%G' /run/anna/annad.sock
    else
        echo "⚠ RPC socket not created yet (may take a few seconds)"
    fi
else
    echo "✗ Daemon failed to start"
    echo ""
    echo "Recent logs:"
    journalctl -u annad -n 20 --no-pager
    exit 1
fi
echo ""

echo "╭─────────────────────────────────────────╮"
echo "│  ✓ Installation Repair Complete        │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Verification:"
echo "  • Check status:  systemctl status annad"
echo "  • View logs:     journalctl -u annad -f"
echo "  • Test CLI:      annactl status"
echo ""
