#!/usr/bin/env bash
# Quick fix for missing CAPABILITIES.toml
# This fixes installations made before v0.13.7

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Fix Missing CAPABILITIES.toml          │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check if we're in the project root
if [ ! -f "etc/CAPABILITIES.toml" ]; then
    echo "✗ Run this script from the project root directory"
    exit 1
fi

# Check if already installed
if [ -f "/usr/lib/anna/CAPABILITIES.toml" ]; then
    echo "✓ CAPABILITIES.toml already installed"
    echo ""
    echo "Checking if daemon is running..."
    if systemctl is-active --quiet annad; then
        echo "✓ Anna daemon is running"
        echo ""
        echo "Test with: annactl status"
    else
        echo "⚠ Daemon is not active"
        echo ""
        echo "Try: sudo systemctl restart annad"
        echo "Then: systemctl status annad"
    fi
    exit 0
fi

# Install the file
echo "→ Installing CAPABILITIES.toml..."
sudo install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/

echo "✓ File installed"
echo ""

# Restart daemon
echo "→ Restarting Anna daemon..."
sudo systemctl restart annad

echo ""
echo "→ Waiting for daemon to start..."
sleep 3

# Check status
if systemctl is-active --quiet annad; then
    echo "✓ Anna daemon is running!"
    echo ""
    echo "╭─────────────────────────────────────────╮"
    echo "│  ✓ Fix Applied Successfully!            │"
    echo "╰─────────────────────────────────────────╯"
    echo ""
    echo "Test with: annactl status"
else
    echo "⚠ Daemon failed to start"
    echo ""
    echo "Check logs: sudo journalctl -u annad -n 50"
    exit 1
fi
