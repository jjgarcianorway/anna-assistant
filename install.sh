#!/usr/bin/env bash
set -e

# Anna Assistant Installer
# Downloads and installs annactl and annad from GitHub releases
# v0.3.39: Complete installer that verifies everything works

REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
SOCKET_PATH="/run/anna/anna.sock"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Anna Assistant Installer ==="
echo

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  ARCH_NAME="x86_64" ;;
    aarch64) ARCH_NAME="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo "Architecture: $ARCH_NAME"

# Get latest release version
echo "Fetching latest release..."
if command -v jq &> /dev/null; then
    LATEST=$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name')
else
    LATEST=$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" | grep -m1 '"tag_name"' | cut -d'"' -f4)
fi

if [ -z "$LATEST" ]; then
    echo -e "${RED}Error: Could not fetch latest release${NC}"
    exit 1
fi

echo -e "Latest version: ${GREEN}$LATEST${NC}"

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
curl -sSL "$ANNACTL_URL" -o "$TMP_DIR/annactl" || { echo -e "${RED}Failed to download annactl${NC}"; exit 1; }
curl -sSL "$ANNAD_URL" -o "$TMP_DIR/annad" || { echo -e "${RED}Failed to download annad${NC}"; exit 1; }
curl -sSL "$SUMS_URL" -o "$TMP_DIR/SHA256SUMS" || { echo -e "${RED}Failed to download checksums${NC}"; exit 1; }

# Verify checksums
echo "Verifying checksums..."
cd "$TMP_DIR"

EXPECTED_ANNACTL=$(grep "annactl-linux-$ARCH_NAME" SHA256SUMS | awk '{print $1}')
EXPECTED_ANNAD=$(grep "annad-linux-$ARCH_NAME" SHA256SUMS | awk '{print $1}')
ACTUAL_ANNACTL=$(sha256sum annactl | awk '{print $1}')
ACTUAL_ANNAD=$(sha256sum annad | awk '{print $1}')

if [ "$EXPECTED_ANNACTL" != "$ACTUAL_ANNACTL" ]; then
    echo -e "${RED}Error: annactl checksum mismatch${NC}"
    exit 1
fi

if [ "$EXPECTED_ANNAD" != "$ACTUAL_ANNAD" ]; then
    echo -e "${RED}Error: annad checksum mismatch${NC}"
    exit 1
fi

echo -e "${GREEN}Checksums OK${NC}"

# Make executable
chmod +x annactl annad

# === CLEANUP OLD BINARIES ===
echo "Checking for old binaries..."

# Common locations where old anna binaries might exist
OLD_LOCATIONS=(
    "$HOME/.local/bin/annactl"
    "$HOME/.local/bin/annad"
    "$HOME/bin/annactl"
    "$HOME/bin/annad"
    "$HOME/.anna/bin/annactl"
    "$HOME/.anna/bin/annad"
)

for loc in "${OLD_LOCATIONS[@]}"; do
    if [ -f "$loc" ]; then
        echo -e "  ${YELLOW}Removing old binary: $loc${NC}"
        rm -f "$loc"
    fi
done

# === INSTALL (requires sudo) ===
echo "Installing to $INSTALL_DIR (requires sudo)..."

# Stop existing service first
sudo systemctl stop annad 2>/dev/null || true

# Remove old socket if exists
sudo rm -f "$SOCKET_PATH" 2>/dev/null || true

# Install new binaries
sudo mv annactl "$INSTALL_DIR/annactl"
sudo mv annad "$INSTALL_DIR/annad"

# === SETUP GROUP ===
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
    echo -e "  ${YELLOW}Note: You may need to log out and back in for group changes to take effect${NC}"
fi

# === CREATE DIRECTORIES ===
echo "Creating system directories..."
sudo mkdir -p /etc/anna
sudo mkdir -p /var/lib/anna /var/lib/anna/backups /var/lib/anna/wiki /var/lib/anna/recipes
sudo mkdir -p /var/log/anna
sudo mkdir -p /run/anna

# Set permissions
sudo chmod 755 /etc/anna
for dir in /var/lib/anna /var/lib/anna/backups /var/lib/anna/wiki /var/lib/anna/recipes /var/log/anna /run/anna; do
    sudo chown root:anna "$dir"
    sudo chmod 750 "$dir"
done

# Verify /run/anna was created
if [ ! -d "/run/anna" ]; then
    echo -e "${RED}Error: Failed to create /run/anna${NC}"
    exit 1
fi

# Create tmpfiles.d for persistence across reboots
sudo tee /etc/tmpfiles.d/anna.conf > /dev/null << 'TMPEOF'
d /run/anna 0750 root anna -
TMPEOF

# === SYSTEMD SERVICE ===
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
WatchdogSec=60
TimeoutStopSec=10
MemoryMax=2G
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload

# === START SERVICE ===
echo "Starting annad service..."
sudo systemctl enable annad
sudo systemctl start annad

# === VERIFY SOCKET EXISTS ===
echo "Waiting for daemon to be ready..."
MAX_WAIT=15
for i in $(seq 1 $MAX_WAIT); do
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "  ${GREEN}Socket ready${NC}"
        break
    fi
    if [ $i -eq $MAX_WAIT ]; then
        echo -e "${RED}Error: Socket not created after ${MAX_WAIT}s${NC}"
        echo "Checking daemon logs..."
        sudo journalctl -u annad -n 20 --no-pager
        exit 1
    fi
    sleep 1
done

# === VERIFY ANNACTL WORKS ===
echo "Verifying installation..."

# Check which annactl is found
FOUND_ANNACTL=$(which annactl 2>/dev/null || echo "")
if [ "$FOUND_ANNACTL" != "$INSTALL_DIR/annactl" ]; then
    if [ -n "$FOUND_ANNACTL" ]; then
        echo -e "${YELLOW}Warning: Another annactl found at $FOUND_ANNACTL${NC}"
        echo -e "${YELLOW}This may shadow $INSTALL_DIR/annactl${NC}"
        echo -e "${YELLOW}Remove it with: rm $FOUND_ANNACTL${NC}"
    fi
fi

# Test annactl status
if "$INSTALL_DIR/annactl" --version > /dev/null 2>&1; then
    VERSION=$("$INSTALL_DIR/annactl" --version)
    echo -e "  ${GREEN}$VERSION${NC}"
else
    echo -e "${RED}Error: annactl --version failed${NC}"
    exit 1
fi

echo
echo -e "${GREEN}=== Installation complete! ===${NC}"
echo
echo "Version: $LATEST"
echo "Binaries: $INSTALL_DIR/annactl, $INSTALL_DIR/annad"
echo "Socket: $SOCKET_PATH"
echo
echo "Usage:"
echo "  annactl                  # Start interactive mode"
echo "  annactl \"your question\"  # Ask a question directly"
echo "  annactl status           # Check daemon status"
echo
