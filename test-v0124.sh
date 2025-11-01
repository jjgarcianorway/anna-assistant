#!/bin/bash
# Quick test script for v0.12.4

set -euo pipefail

echo "Installing v0.12.4..."
sudo systemctl stop annad
sudo cp bin/annad bin/annactl /usr/local/bin/
sudo systemctl start annad

echo "Waiting for daemon..."
for i in {1..10}; do
    if [ -S /run/anna/annad.sock ]; then
        echo "✓ Socket ready after ${i}s"
        break
    fi
    sleep 1
done

echo ""
echo "Testing annactl doctor check..."
echo ""

annactl doctor check

echo ""
echo "Testing with verbose..."
echo ""

annactl doctor check --verbose
