#!/bin/bash
# Run this to fix the socket ownership issue
set -ex

# Stop daemon
sudo systemctl stop annad

# Copy the fixed binary (force overwrite)
sudo cp -f target/release/annad /usr/local/bin/annad

# Verify it was copied
sudo ls -l /usr/local/bin/annad

# Start daemon
sudo systemctl start annad

# Wait for socket to be created
sleep 3

# Check socket ownership (should be root:anna)
echo "=== Socket ownership (should be root:anna) ==="
ls -la /run/anna/annad.sock

# Check logs for the fix message
echo ""
echo "=== Checking for fix in logs (should see 'Socket group ownership set to anna') ==="
sudo journalctl -u annad --since "10 seconds ago" --no-pager | grep -i "socket"

# Test connection
echo ""
echo "=== Testing connection ==="
annactl status
