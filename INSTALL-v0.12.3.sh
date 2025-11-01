#!/bin/bash
# Install v0.12.3 with complete RPC methods

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Installing Anna v0.12.3               │"
echo "│  With collectors, radars, RPC methods  │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Stop daemon
echo "→ Stopping daemon..."
sudo systemctl stop annad
echo "✓ Stopped"
echo ""

# Install binaries
echo "→ Installing binaries..."
sudo cp target/release/annad target/release/annactl /usr/local/bin/
sudo chmod +x /usr/local/bin/anna*
echo "✓ Installed"
echo ""

# Verify version
echo "→ Checking version..."
VERSION=$(/usr/local/bin/annactl --version | awk '{print $2}')
echo "✓ Version: $VERSION"
echo ""

# Start daemon
echo "→ Starting daemon..."
sudo systemctl start annad
echo "✓ Started"
echo ""

# Wait for socket
echo "→ Waiting for socket..."
for i in {1..10}; do
    if [ -S /run/anna/annad.sock ]; then
        echo "✓ Socket ready"
        break
    fi
    sleep 1
done
echo ""

# Test daemon
echo "→ Testing daemon..."
if timeout 5 annactl status &>/dev/null; then
    echo "✓ Daemon responding"
else
    echo "✗ Daemon not responding"
    echo ""
    echo "Checking logs:"
    sudo journalctl -u annad --no-pager -n 10
    exit 1
fi
echo ""

echo "╭─────────────────────────────────────────╮"
echo "│  Installation Complete                  │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Test the new commands:"
echo "  annactl collect --limit 1"
echo "  annactl classify"
echo "  annactl radar show"
echo ""
