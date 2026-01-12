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
