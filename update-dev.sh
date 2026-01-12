#!/bin/bash
# Development update script - run with sudo

set -e

echo "=== Stopping annad service ==="
systemctl stop annad || true

# Kill any stray annad processes
pkill -9 -f "annad" 2>/dev/null || true

echo "=== Updating Anna binaries ==="
cp /home/lhoqvso/anna-assistant/target/release/annad /usr/local/bin/annad
cp /home/lhoqvso/anna-assistant/target/release/annactl /usr/local/bin/annactl

echo "=== Updating systemd service with watchdog ==="
cat > /etc/systemd/system/annad.service << 'EOF'
[Unit]
Description=Anna Assistant Daemon
After=network.target ollama.service
Wants=ollama.service

[Service]
Type=notify
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=3
# Watchdog: annad must ping every 30s or get killed and restarted
WatchdogSec=60
# Kill frozen process after 10s
TimeoutStopSec=10
# Resource limits
MemoryMax=2G
# Environment
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
systemctl daemon-reload

echo "=== Installing ollama-cuda for GPU support ==="
pacman -S --noconfirm ollama-cuda 2>/dev/null || echo "Already installed or unavailable"

echo "=== Restarting services ==="
systemctl restart ollama
systemctl restart annad

echo "=== Waiting for services to start ==="
sleep 5

echo "=== Checking GPU status ==="
ollama ps

echo "=== Service status ==="
systemctl status annad --no-pager | head -15

echo "=== Done ==="
