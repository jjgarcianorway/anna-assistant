#!/usr/bin/env bash
# Fix systemd service RuntimeDirectory conflict
# Removes redundant ExecStartPre that conflicts with RuntimeDirectory=anna

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Fix Anna Daemon Socket Issue          │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "✗ This script must be run with sudo"
    echo "  Usage: sudo $0"
    exit 1
fi

SERVICE_FILE="/etc/systemd/system/annad.service"
SOURCE_FILE="etc/systemd/annad.service"

# Check if service file exists
if [ ! -f "$SERVICE_FILE" ]; then
    echo "✗ Service file not found: $SERVICE_FILE"
    exit 1
fi

# Check if we're in the project root
if [ ! -f "$SOURCE_FILE" ]; then
    echo "✗ Run this script from the anna-assistant project root"
    exit 1
fi

echo "→ Stopping Anna daemon..."
systemctl stop annad || true

echo "→ Installing fixed service file..."
install -m 0644 "$SOURCE_FILE" "$SERVICE_FILE"

echo "→ Reloading systemd..."
systemctl daemon-reload

echo "→ Starting Anna daemon..."
systemctl start annad

echo ""
echo "→ Waiting for daemon to start..."
sleep 2

# Check if socket exists
if [ -S "/run/anna/annad.sock" ]; then
    echo "✓ Socket created successfully!"
    echo ""
    echo "╭─────────────────────────────────────────╮"
    echo "│  ✓ Fix Applied Successfully!            │"
    echo "╰─────────────────────────────────────────╯"
    echo ""
    echo "Test with: annactl status"
else
    echo "✗ Socket not created"
    echo ""
    echo "Check logs: sudo journalctl -u annad -n 50"
    echo "Check status: sudo systemctl status annad"
    exit 1
fi
