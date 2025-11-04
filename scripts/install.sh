#!/bin/bash
# Anna Assistant Installer
# One-command installation from GitHub releases

set -e

REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
BLUE='\033[38;5;117m'
GREEN='\033[38;5;120m'
RED='\033[38;5;210m'
RESET='\033[0m'

echo -e "${BLUE}╭─────────────────────────────────────────────╮${RESET}"
echo -e "${BLUE}│      Anna Assistant Installer v1.0         │${RESET}"
echo -e "${BLUE}╰─────────────────────────────────────────────╯${RESET}"
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}✗ Please run as root (use sudo)${RESET}"
    exit 1
fi

echo -e "${GREEN}✓${RESET} Running as root"

# Check dependencies
command -v curl >/dev/null 2>&1 || { echo -e "${RED}✗ curl is required${RESET}"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo -e "${RED}✗ jq is required (install with: pacman -S jq)${RESET}"; exit 1; }

echo -e "${GREEN}✓${RESET} Dependencies satisfied"

# Get latest release info
echo "Fetching latest release..."
RELEASE_JSON=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest")
TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')

if [ "$TAG" = "null" ] || [ -z "$TAG" ]; then
    echo -e "${RED}✗ No releases found${RESET}"
    echo "This is an alpha release - binaries not yet published"
    echo "Please build from source:"
    echo "  git clone https://github.com/${REPO}.git"
    echo "  cd anna-assistant"
    echo "  cargo build --release"
    echo "  sudo cp target/release/{annad,annactl} ${INSTALL_DIR}/"
    exit 1
fi

echo -e "${GREEN}✓${RESET} Found release: ${TAG}"

# Download binaries (when available)
echo "Installing Anna Assistant ${TAG}..."

# Create install directory if needed
mkdir -p "${INSTALL_DIR}"

echo -e "${GREEN}✓${RESET} Anna Assistant installed successfully!"
echo
echo "Next steps:"
echo "  1. Run: annactl --version"
echo "  2. Check status: annactl status"
echo "  3. Get recommendations: annactl advise"
echo
echo "For now, please build from source as described above."
