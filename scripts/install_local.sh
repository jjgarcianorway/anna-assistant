#!/usr/bin/env bash
# Install from LOCAL binaries (for development)
set -Eeuo pipefail

# Must run as root
[[ $EUID -eq 0 ]] || { echo "Run as root: sudo $0"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo ""
echo "═══════════════════════════════════════════"
echo "  Anna Local Installer"
echo "═══════════════════════════════════════════"
echo ""

# Check binaries exist
if [[ ! -f "$REPO_ROOT/target/release/annad" || ! -f "$REPO_ROOT/target/release/annactl" ]]; then
    echo "✗ No binaries found at target/release/"
    echo ""
    echo "Run: ./scripts/release.sh"
    exit 1
fi

echo "→ Using local binaries..."

# Stop service
systemctl stop annad 2>/dev/null || true

# Install
install -m 755 "$REPO_ROOT/target/release/annad" /usr/local/bin/annad
install -m 755 "$REPO_ROOT/target/release/annactl" /usr/local/bin/annactl

# Create directories
mkdir -p /etc/anna/policies.d /var/lib/anna/{telemetry,backups} /var/log/anna /run/anna
chmod 755 /etc/anna /etc/anna/policies.d /var/lib/anna /var/lib/anna/telemetry /var/lib/anna/backups /var/log/anna /run/anna

VERSION=$("$REPO_ROOT/target/release/annactl" --version | awk '{print $NF}')
echo "v$VERSION" > /etc/anna/version

# Systemd service
cat > /etc/systemd/system/annad.service <<'EOF'
[Unit]
Description=Anna Assistant Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable annad 2>/dev/null || true
systemctl start annad 2>/dev/null || true

echo ""
echo "══════════════════════════════════════════════"
echo "  ✓ Installation Complete!"
echo "══════════════════════════════════════════════"
echo ""
echo "Version: v$VERSION (local build)"
echo "Check:   annactl status"
echo ""
