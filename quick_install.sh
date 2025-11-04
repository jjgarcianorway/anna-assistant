#!/bin/bash
set -e
sudo cp target/release/annad /usr/local/bin/annad
sudo systemctl restart annad
sleep 2
echo "=== Socket ownership after restart ==="
ls -la /run/anna/annad.sock
echo ""
echo "=== Testing connection ==="
annactl status
