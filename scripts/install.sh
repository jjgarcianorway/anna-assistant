#!/usr/bin/env bash
# Anna Assistant Installer v0.11.0
# Runs as user, escalates only when needed

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Assistant Installer              │"
echo "│  v0.11.0 - Event-Driven Intelligence   │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Find project root and cd to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Verify we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "✗ Could not find project root (expected Cargo.toml)"
    exit 1
fi

echo "→ Building binaries (this may take 2-5 minutes on first build)..."
if cargo build --release 2>&1 | grep -E "error|warning: unused" | head -5; then
    if [ ${PIPESTATUS[0]} -ne 0 ]; then
        echo "✗ Build failed"
        exit 1
    fi
fi
echo "✓ Build complete"
echo ""

echo "The following steps require elevated privileges:"
echo "  • Create anna system user and group"
echo "  • Install binaries to /usr/local/bin"
echo "  • Install systemd service"
echo "  • Create directories and set permissions"
echo ""
read -p "Proceed? [Y/n] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]?$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "→ Creating anna system user and group..."

# Check if group exists first
if ! getent group anna &>/dev/null; then
    GROUP_EXISTS=false
else
    GROUP_EXISTS=true
    echo "✓ Group 'anna' exists"
fi

# Create user (handle group existence)
if ! id -u anna &>/dev/null; then
    if [ "$GROUP_EXISTS" = true ]; then
        # Group exists, so use it explicitly
        sudo useradd --system --no-create-home --shell /usr/sbin/nologin -g anna anna
        echo "✓ User 'anna' created (using existing group)"
    else
        # Neither exists, let useradd create both
        sudo useradd --system --no-create-home --shell /usr/sbin/nologin anna
        echo "✓ User 'anna' created"
        echo "✓ Group 'anna' created"
    fi
else
    echo "✓ User 'anna' exists"
fi

echo ""
echo "→ Adding current user to anna group..."
if ! groups | grep -q anna; then
    sudo usermod -aG anna "$USER"
    echo "✓ Added to group (log out/in for changes to take effect)"
else
    echo "✓ Already in anna group"
fi

echo ""
echo "→ Installing binaries..."
sudo install -m 755 target/release/annad /usr/local/bin/
sudo install -m 755 target/release/annactl /usr/local/bin/
echo "✓ Binaries installed to /usr/local/bin"

echo ""
echo "→ Creating directories..."
sudo mkdir -p /etc/anna/policies.d
sudo mkdir -p /etc/anna/personas.d
sudo mkdir -p /run/anna
sudo mkdir -p /var/lib/anna
sudo mkdir -p /var/log/anna
sudo mkdir -p /usr/lib/anna

sudo chown anna:anna /run/anna
sudo chown anna:anna /var/lib/anna
sudo chown anna:anna /var/log/anna
sudo chown root:anna /etc/anna
sudo chown root:root /usr/lib/anna
sudo chmod 0750 /run/anna /var/lib/anna /var/log/anna
sudo chmod 0755 /etc/anna /usr/lib/anna
echo "✓ Directories created with correct ownership"

echo ""
echo "→ Installing configuration..."
if [ ! -f /etc/anna/config.toml ]; then
    cat <<'EOF' | sudo tee /etc/anna/config.toml >/dev/null
# Hi, I'm Anna! Please use 'annactl config set' to change settings.
# Don't edit me by hand - I track who changed what and why.

[autonomy]
level = "low"

[ui]
emojis = false
color = true

[telemetry]
enabled = false
collection_interval_sec = 60

[persona]
active = "dev"
EOF
    echo "✓ Default config installed"
else
    echo "✓ Config exists"
fi

echo ""
echo "→ Installing policies..."

# v0.11.0 Event Auto-Repair Policy
if [ ! -f /etc/anna/policy.toml ]; then
    if [ -f etc/policy.toml ]; then
        sudo cp etc/policy.toml /etc/anna/
        echo "✓ Event auto-repair policy installed"
    else
        echo "⚠ etc/policy.toml not found, skipping"
    fi
else
    echo "✓ Event policy exists"
fi

if [ ! -f /etc/anna/policies.d/00-bootstrap.yaml ]; then
    cat <<'EOF' | sudo tee /etc/anna/policies.d/00-bootstrap.yaml >/dev/null
# Hi, I'm Anna! Use 'annactl policy' to manage these rules.

- when: "telemetry.cpu_usage > 90"
  then: "alert"
  message: "High CPU usage"
  enabled: true

- when: "telemetry.mem_usage > 95"
  then: "alert"
  message: "Critical memory"
  enabled: true

- when: "always"
  then: "log"
  message: "Policy engine ready"
  enabled: true
EOF
    echo "✓ Bootstrap policies installed"
else
    echo "✓ Bootstrap policies exist"
fi

# Install thermal management policies
if [ ! -f /etc/anna/policies.d/thermal.yaml ]; then
    if [ -f etc/policies.d/thermal.yaml ]; then
        sudo cp etc/policies.d/thermal.yaml /etc/anna/policies.d/
        echo "✓ Thermal management policies installed"
    else
        echo "○ Thermal policies not found, skipping"
    fi
else
    echo "✓ Thermal policies exist"
fi

echo ""
echo "→ Installing capability registry..."
if [ ! -f /usr/lib/anna/CAPABILITIES.toml ]; then
    if [ -f etc/CAPABILITIES.toml ]; then
        sudo cp etc/CAPABILITIES.toml /usr/lib/anna/
        sudo chmod 0644 /usr/lib/anna/CAPABILITIES.toml
        echo "✓ CAPABILITIES.toml installed"
    else
        echo "⚠ CAPABILITIES.toml not found in project"
        echo "  Creating minimal version..."
        cat <<'EOF' | sudo tee /usr/lib/anna/CAPABILITIES.toml >/dev/null
[meta]
version = "0.11.0"
description = "Anna telemetry capability registry"

[[modules]]
name = "lm_sensors"
description = "Temperature and voltage monitoring"
category = "sensors"
required = true

[[modules.deps]]
binaries = ["sensors"]
packages = ["lm_sensors"]

[[modules]]
name = "iproute2"
description = "Network interface monitoring"
category = "net"
required = true

[[modules.deps]]
binaries = ["ip"]
packages = ["iproute2"]
EOF
        echo "✓ Minimal CAPABILITIES.toml created"
    fi
else
    echo "✓ CAPABILITIES.toml exists"
fi

echo ""
echo "→ Detecting hardware..."

# Detect ASUS hardware
IS_ASUS=false
if [ -d /sys/devices/platform/asus-nb-wmi ] || [ -d /sys/devices/platform/asus_wmi ] || grep -qi asus /sys/class/dmi/id/board_vendor 2>/dev/null; then
    IS_ASUS=true
    echo "✓ ASUS hardware detected"
else
    echo "✓ Generic system detected"
fi

# Install essential sensors
if ! command -v sensors &>/dev/null; then
    echo "→ Installing lm-sensors..."
    if command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm lm_sensors >/dev/null 2>&1 && echo "✓ Sensors installed" || echo "⚠ Could not install sensors"
    fi
else
    echo "✓ Sensors already installed"
fi

echo ""
echo "→ Installing systemd services..."
if [ -f etc/systemd/annad.service ]; then
    sudo cp etc/systemd/annad.service /etc/systemd/system/
    sudo cp etc/systemd/anna-fans.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable annad
    echo "✓ Services installed and enabled"
else
    echo "✗ Service files not found in etc/systemd/"
    echo "  Working directory: $(pwd)"
    exit 1
fi

# Install thermal management scripts
if [ "$IS_ASUS" = true ]; then
    sudo mkdir -p /usr/local/share/anna
    if [ -f scripts/anna_fans_asus.sh ]; then
        sudo cp scripts/anna_fans_asus.sh /usr/local/share/anna/
        sudo chmod +x /usr/local/share/anna/anna_fans_asus.sh
        echo "✓ Thermal management scripts installed"
    fi
fi

echo ""
echo "→ Starting daemon..."
if sudo systemctl start annad; then
    sleep 2
    if systemctl is-active --quiet annad; then
        echo "✓ Daemon started successfully"
    else
        echo "⚠ Daemon may still be starting..."
        echo "  Check status: systemctl status annad"
    fi
else
    echo "✗ Failed to start daemon"
    echo "  Check logs: journalctl -u annad -n 20"
    exit 1
fi

echo ""
echo "╭─────────────────────────────────────────╮"
echo "│  ✓ Anna Installed Successfully          │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Next steps:"
echo ""
echo "  1. Run system health check:"
echo "     annactl doctor check"
echo ""
echo "  2. Let Anna help with system setup:"
echo "     annactl doctor setup"
echo ""
echo "  3. Check current status:"
echo "     annactl status"
echo ""
echo "Note: Log out and back in for full permissions"
echo ""
