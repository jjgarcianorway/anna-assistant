#!/bin/bash
# Quick fix to get v0.12.2 running properly

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Fixing Anna v0.12.2 Installation      │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Step 1: Stop old daemon
echo "→ Stopping old daemon..."
sudo systemctl stop annad
echo "✓ Stopped"
echo ""

# Step 2: Update bin/ directory with new v0.12.2 binaries
echo "→ Updating bin/ with v0.12.2 binaries..."
cp target/release/annad target/release/annactl bin/
chmod +x bin/*
echo "✓ Binaries updated"
echo ""

# Step 3: Verify version
echo "→ Checking version..."
VERSION=$(./bin/annactl --version | awk '{print $2}')
echo "✓ bin/ now has version: $VERSION"
echo ""

# Step 4: Install
echo "→ Installing v0.12.2..."
echo "  (This will start the daemon with the new binaries)"
echo ""

# Let install.sh do its thing
./scripts/install.sh

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  Installation Complete                  │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Test with:"
echo "  annactl status"
echo "  annactl collect --limit 1"
echo "  annactl classify"
echo "  annactl radar show"
echo ""
