#!/usr/bin/env bash
# Anna Assistant Installer v0.9.6-alpha.7
# Runs as user, escalates only when needed

set -euo pipefail

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Assistant Installer              │"
echo "│  v0.9.6-alpha.7                         │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Check we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "✗ Please run from anna-assistant project root"
    exit 1
fi

echo "→ Building binaries..."
if ! cargo build --release --quiet 2>&1; then
    echo "✗ Build failed"
    exit 1
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
echo "→ Creating anna system user..."
if ! id -u anna &>/dev/null; then
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin anna
    echo "✓ User 'anna' created"
else
    echo "✓ User 'anna' exists"
fi

if ! groups anna | grep -q anna; then
    sudo groupadd anna 2>/dev/null || true
    echo "✓ Group 'anna' ready"
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

sudo chown anna:anna /run/anna
sudo chown anna:anna /var/lib/anna
sudo chown anna:anna /var/log/anna
sudo chmod 0750 /run/anna /var/lib/anna /var/log/anna
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
    echo "✓ Default policies installed"
else
    echo "✓ Policies exist"
fi

echo ""
echo "→ Installing systemd service..."
sudo cp etc/systemd/annad.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable annad
echo "✓ Service installed and enabled"

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
echo "│  ✓ Installation Complete                │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Anna is ready! Try:"
echo "  annactl status"
echo "  annactl profile show"
echo "  annactl doctor check"
echo ""
echo "Note: Log out and back in for group membership to take effect"
echo ""
