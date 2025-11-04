#!/bin/bash
# Fix: Socket group ownership issue
# This script rebuilds annad and reinstalls it with the socket ownership fix

set -e

echo "=== Building annad with socket fix ==="
cargo build --release --bin annad

echo ""
echo "=== Installing new annad binary ==="
sudo install -m 755 target/release/annad /usr/local/bin/

echo ""
echo "=== Restarting annad service ==="
sudo systemctl restart annad

echo ""
echo "=== Waiting for daemon to start ==="
sleep 2

echo ""
echo "=== Checking socket ownership ==="
ls -la /run/anna/annad.sock

echo ""
echo "=== Testing connection ==="
annactl status

echo ""
echo "✅ Fix applied successfully!"
