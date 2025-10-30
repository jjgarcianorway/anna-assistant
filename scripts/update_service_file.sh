#!/usr/bin/env bash
# Helper script to update the systemd service file
# Run with: sudo ./scripts/update_service_file.sh

set -euo pipefail

if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run as root"
    echo "Usage: sudo ./scripts/update_service_file.sh"
    exit 1
fi

echo "Updating Anna systemd service file..."
echo ""

# Stop the service
echo "→ Stopping service..."
systemctl stop annad || true

# Copy new service file
echo "→ Installing new service file..."
cp etc/systemd/annad.service /etc/systemd/system/annad.service

# Reload systemd
echo "→ Reloading systemd..."
systemctl daemon-reload

# Start the service
echo "→ Starting service..."
systemctl start annad

# Check status
sleep 2
if systemctl is-active --quiet annad; then
    echo ""
    echo "✓ Service updated and running successfully!"
    echo ""
    echo "Verify with:"
    echo "  systemctl status annad"
    echo "  annactl status"
else
    echo ""
    echo "✗ Service failed to start"
    echo ""
    echo "Check logs:"
    echo "  journalctl -u annad -n 50"
    exit 1
fi
