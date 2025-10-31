#!/usr/bin/env bash
# Emergency fan fix - run this NOW to quiet your fans

set -e

echo "🔧 Emergency Fan Fix"
echo ""

# 1. Set fans to quiet immediately
echo "→ Setting fans to quiet mode..."
asusctl profile -P quiet
echo "✓ Fans should be quieter now"
echo ""

# 2. Install fixed daemon
echo "→ Installing fixed daemon..."
sudo install -m 755 target/release/annad /usr/local/bin/
echo "✓ Fixed daemon installed"
echo ""

# 3. Fix permissions
echo "→ Fixing Anna directories..."
sudo mkdir -p /run/anna /var/lib/anna /var/log/anna
sudo chown anna:anna /run/anna /var/lib/anna /var/log/anna
sudo chmod 0770 /run/anna /var/lib/anna /var/log/anna
echo "✓ Permissions fixed"
echo ""

# 4. Restart daemon
echo "→ Restarting Anna daemon..."
sudo systemctl restart annad
sleep 2

if systemctl is-active --quiet annad; then
    echo "✓ Anna daemon is now running!"
    echo ""
    echo "Check status: systemctl status annad"
    echo "Your fans should stay quiet now. Anna is managing them."
else
    echo "⚠ Daemon still having issues."
    echo "Check logs: journalctl -u annad -n 20"
fi
