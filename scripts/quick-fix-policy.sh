#!/usr/bin/env bash
# Quick fix: Install missing policy.toml
# This gets Anna running immediately while you prepare the release

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Quick Fix: Install policy.toml         │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check if we're in project root
if [ ! -f "etc/policy.toml" ]; then
    echo "✗ Run this from the anna-assistant project root"
    exit 1
fi

# Check if already installed
if [ -f "/etc/anna/policy.toml" ]; then
    echo "✓ policy.toml already exists"
else
    echo "→ Installing policy.toml..."
    sudo install -m 0644 etc/policy.toml /etc/anna/
    echo "✓ Installed"
fi

echo ""
echo "→ Restarting daemon..."
sudo systemctl restart annad

echo "→ Waiting for daemon..."
sleep 3

echo ""
if annactl status &>/dev/null; then
    echo "╭─────────────────────────────────────────╮"
    echo "│  ✓ Anna is Running!                     │"
    echo "╰─────────────────────────────────────────╯"
    echo ""
    echo "Test it: annactl status"
else
    echo "⚠ Still having issues. Check logs:"
    echo "  sudo journalctl -u annad -n 30"
    exit 1
fi
