#!/usr/bin/env bash
set -e

# Anna Assistant Installer
# Downloads and installs annactl and annad from GitHub releases

REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"

echo "=== Anna Assistant Installer ==="
echo

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  ARCH_NAME="x86_64" ;;
    aarch64) ARCH_NAME="aarch64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Architecture: $ARCH_NAME"

# Get latest release version
echo "Fetching latest release..."
LATEST=$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: Could not fetch latest release"
    exit 1
fi

echo "Latest version: $LATEST"

# Download URLs
BASE_URL="https://github.com/$REPO/releases/download/$LATEST"
ANNACTL_URL="$BASE_URL/annactl-linux-$ARCH_NAME"
ANNAD_URL="$BASE_URL/annad-linux-$ARCH_NAME"
SUMS_URL="$BASE_URL/SHA256SUMS"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo "Downloading binaries..."

# Download files
curl -sSL "$ANNACTL_URL" -o "$TMP_DIR/annactl"
curl -sSL "$ANNAD_URL" -o "$TMP_DIR/annad"
curl -sSL "$SUMS_URL" -o "$TMP_DIR/SHA256SUMS"

# Verify checksums
echo "Verifying checksums..."
cd "$TMP_DIR"

EXPECTED_ANNACTL=$(grep "annactl-linux-$ARCH_NAME" SHA256SUMS | awk '{print $1}')
EXPECTED_ANNAD=$(grep "annad-linux-$ARCH_NAME" SHA256SUMS | awk '{print $1}')
ACTUAL_ANNACTL=$(sha256sum annactl | awk '{print $1}')
ACTUAL_ANNAD=$(sha256sum annad | awk '{print $1}')

if [ "$EXPECTED_ANNACTL" != "$ACTUAL_ANNACTL" ]; then
    echo "Error: annactl checksum mismatch"
    exit 1
fi

if [ "$EXPECTED_ANNAD" != "$ACTUAL_ANNAD" ]; then
    echo "Error: annad checksum mismatch"
    exit 1
fi

echo "Checksums OK"

# Make executable
chmod +x annactl annad

# Install (requires sudo)
echo "Installing to $INSTALL_DIR (requires sudo)..."
sudo mv annactl "$INSTALL_DIR/annactl"
sudo mv annad "$INSTALL_DIR/annad"

# v0.3.31: Create anna group if it doesn't exist
echo "Setting up anna group..."
if ! getent group anna > /dev/null 2>&1; then
    sudo groupadd anna
    echo "  Created group: anna"
fi

# Add current user to anna group
USERNAME=$(whoami)
if ! groups "$USERNAME" 2>/dev/null | grep -q "\banna\b"; then
    sudo usermod -aG anna "$USERNAME"
    echo "  Added $USERNAME to anna group"
fi

# v0.3.32: Create system directories with secure permissions
# Model: daemon (root) writes, anna group reads via socket RPC
echo "Creating system directories..."
sudo mkdir -p /etc/anna
sudo mkdir -p /var/lib/anna /var/lib/anna/backups /var/lib/anna/wiki /var/lib/anna/recipes
sudo mkdir -p /var/log/anna
sudo mkdir -p /run/anna

# Set permissions: 750 root:anna (daemon writes, group reads)
sudo chmod 755 /etc/anna
for dir in /var/lib/anna /var/lib/anna/backups /var/lib/anna/wiki /var/lib/anna/recipes /var/log/anna /run/anna; do
    sudo chown root:anna "$dir"
    sudo chmod 750 "$dir"
done

# Create tmpfiles.d config for /run/anna persistence
sudo tee /etc/tmpfiles.d/anna.conf > /dev/null << 'TMPEOF'
# Anna runtime directory - daemon writes, anna group can connect to socket
d /run/anna 0750 root anna -
TMPEOF

# Create/update systemd service
echo "Creating systemd service..."
sudo tee /etc/systemd/system/annad.service > /dev/null << 'EOF'
[Unit]
Description=Anna Assistant Daemon
After=network.target ollama.service
Wants=ollama.service

[Service]
Type=notify
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=3
# Watchdog: annad must ping every 30s or get killed
WatchdogSec=60
# Kill frozen process after 10s
TimeoutStopSec=10
# Resource limits
MemoryMax=2G
# Environment
Environment=RUST_BACKTRACE=1
# v0.3.32: System-wide paths - runtime directory
RuntimeDirectory=anna
RuntimeDirectoryMode=0750

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload

# Start/restart service
echo "Starting annad service..."
sudo systemctl enable annad
sudo systemctl restart annad

echo
echo "=== Installation complete! ==="
echo
echo "Version: $LATEST"
echo "Binaries: $INSTALL_DIR/annactl, $INSTALL_DIR/annad"
echo
echo "Usage:"
echo "  annactl                  # Start interactive mode"
echo "  annactl \"your question\"  # Ask a question directly"
echo
echo "Service status: sudo systemctl status annad"
