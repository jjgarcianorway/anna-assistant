#!/bin/bash
# Quick local install script for Anna Assistant

set -e

echo "⚠️  WARNING: This script is for DEVELOPMENT ONLY!"
echo "   For production, use one of these methods:"
echo "   - annactl update --install"
echo "   - curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh"
echo ""
echo "🔄 Installing Anna Assistant beta.49..."
echo

# Stop daemon
echo "→ Stopping daemon..."
systemctl stop annad 2>/dev/null || true

# Copy binaries
echo "→ Installing binaries to /usr/local/bin..."
cp ./target/release/annad /usr/local/bin/
cp ./target/release/annactl /usr/local/bin/
chmod +x /usr/local/bin/annad
chmod +x /usr/local/bin/annactl

# Start daemon
echo "→ Starting daemon..."
systemctl start annad

echo
echo "✓ Installation complete!"
echo
echo "Verify version:"
echo "  annactl status"
