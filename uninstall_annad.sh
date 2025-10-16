#!/usr/bin/env bash
set -e

SERVICE_PATH="/etc/systemd/system/annad.service"
BIN_PATH="/usr/local/bin/annad"

if ! systemctl list-unit-files | grep -q '^annad.service'; then
    echo "Anna is not installed."
    exit 0
fi

echo "Removing Anna..."
sudo systemctl stop annad || true
sudo systemctl disable annad || true
sudo rm -f "$SERVICE_PATH"
sudo rm -f "$BIN_PATH"
sudo systemctl daemon-reload

echo "Anna uninstalled."
