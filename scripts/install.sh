#!/bin/bash
# Anna Assistant Installer
# Fetches and installs the latest release from GitHub

set -e

REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"

# Colors (pastel theme with ASCII fallback)
if [ -t 1 ] && command -v tput >/dev/null 2>&1 && [ "$(tput colors)" -ge 256 ]; then
    BLUE='\033[38;5;117m'
    GREEN='\033[38;5;120m'
    YELLOW='\033[38;5;228m'
    RED='\033[38;5;210m'
    CYAN='\033[38;5;159m'
    GRAY='\033[38;5;250m'
    RESET='\033[0m'
    BOLD='\033[1m'
    BOX_TL="╭"; BOX_TR="╮"; BOX_BL="╰"; BOX_BR="╯"; BOX_H="─"; BOX_V="│"
    CHECK="✓"; CROSS="✗"; WARN="⚠"; INFO="ℹ"; ARROW="→"
else
    # ASCII fallback
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    BOX_TL="+"; BOX_TR="+"; BOX_BL="+"; BOX_BR="+"; BOX_H="-"; BOX_V="|"
    CHECK="[OK]"; CROSS="[X]"; WARN="[!]"; INFO="[i]"; ARROW="->"
fi

print_header() {
    echo -e "${BOLD}${BLUE}${BOX_TL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_TR}${RESET}"
    echo -e "${BOLD}${BLUE}${BOX_V}    Anna Assistant Installer v1.0        ${BOX_V}${RESET}"
    echo -e "${BOLD}${BLUE}${BOX_BL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_BR}${RESET}"
    echo
}

error_exit() {
    echo -e "${RED}${CROSS} $1${RESET}" >&2
    exit 1
}

print_header

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}${CROSS} This installer requires root privileges${RESET}" >&2
    echo >&2
    echo -e "${GRAY}Please run with sudo:${RESET}" >&2
    echo -e "  ${CYAN}curl -sSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | sudo sh${RESET}" >&2
    echo >&2
    exit 1
fi

echo -e "${GREEN}${CHECK}${RESET} Running as root"

# Check dependencies
command -v curl >/dev/null 2>&1 || error_exit "curl is required (install with: pacman -S curl)"
command -v jq >/dev/null 2>&1 || error_exit "jq is required (install with: pacman -S jq)"
command -v tar >/dev/null 2>&1 || error_exit "tar is required"

echo -e "${GREEN}${CHECK}${RESET} Dependencies satisfied"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    error_exit "Unsupported architecture: $ARCH (only x86_64 supported)"
fi

echo -e "${GREEN}${CHECK}${RESET} Architecture: ${ARCH}"

# Fetch latest release info
echo -e "${CYAN}${ARROW}${RESET} Fetching latest release from GitHub..."
RELEASE_JSON=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest")
TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')

if [ "$TAG" = "null" ] || [ -z "$TAG" ]; then
    error_exit "No releases found - this may be an alpha/beta version"
fi

echo -e "${GREEN}${CHECK}${RESET} Found release: ${BOLD}${TAG}${RESET}"

# Get asset URLs
ANNAD_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name | contains("annad")) | .browser_download_url')
ANNACTL_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name | contains("annactl")) | .browser_download_url')

if [ -z "$ANNAD_URL" ] || [ -z "$ANNACTL_URL" ]; then
    error_exit "Release assets not found - build may still be in progress"
fi

echo -e "${CYAN}${ARROW}${RESET} Downloading binaries..."

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Download annad
curl -L -o "$TEMP_DIR/annad" "$ANNAD_URL" 2>/dev/null || error_exit "Failed to download annad"
echo -e "${GREEN}${CHECK}${RESET} Downloaded annad"

# Download annactl
curl -L -o "$TEMP_DIR/annactl" "$ANNACTL_URL" 2>/dev/null || error_exit "Failed to download annactl"
echo -e "${GREEN}${CHECK}${RESET} Downloaded annactl"

# Verify binaries are executable
chmod +x "$TEMP_DIR/annad" "$TEMP_DIR/annactl"

# Verify version matches
ANNAD_VERSION=$("$TEMP_DIR/annad" --version 2>/dev/null | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
ANNACTL_VERSION=$("$TEMP_DIR/annactl" --version 2>/dev/null | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")

if [ "$ANNAD_VERSION" != "$TAG" ]; then
    echo -e "${YELLOW}${WARN} Version mismatch: annad reports ${ANNAD_VERSION}, expected ${TAG}${RESET}"
fi

if [ "$ANNACTL_VERSION" != "$TAG" ]; then
    echo -e "${YELLOW}${WARN} Version mismatch: annactl reports ${ANNACTL_VERSION}, expected ${TAG}${RESET}"
fi

echo -e "${CYAN}${ARROW}${RESET} Stopping any running instances..."

# Stop systemd service if exists
systemctl stop annad 2>/dev/null && echo -e "${GREEN}${CHECK}${RESET} Stopped annad service" || true

# Kill any remaining processes
pkill -x annad 2>/dev/null && echo -e "${GREEN}${CHECK}${RESET} Stopped annad process" || true
pkill -x annactl 2>/dev/null || true

# Wait for processes to fully stop
sleep 1

echo -e "${CYAN}${ARROW}${RESET} Installing to ${INSTALL_DIR}..."

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Install binaries
cp "$TEMP_DIR/annad" "$INSTALL_DIR/annad"
cp "$TEMP_DIR/annactl" "$INSTALL_DIR/annactl"
chmod 755 "$INSTALL_DIR/annad" "$INSTALL_DIR/annactl"

echo -e "${GREEN}${CHECK}${RESET} Binaries installed"

# Restart service if it was enabled before
if systemctl is-enabled --quiet annad 2>/dev/null; then
    echo -e "${CYAN}${ARROW}${RESET} Restarting annad service..."
    systemctl start annad
    echo -e "${GREEN}${CHECK}${RESET} Service restarted"
fi

# Verify installation
if ! command -v annactl >/dev/null 2>&1; then
    echo -e "${YELLOW}${WARN} annactl not in PATH - you may need to add ${INSTALL_DIR} to PATH${RESET}"
fi

echo
echo -e "${BOLD}${GREEN}${BOX_TL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_TR}${RESET}"
echo -e "${BOLD}${GREEN}${BOX_V}   Installation Complete! ${TAG}         ${BOX_V}${RESET}"
echo -e "${BOLD}${GREEN}${BOX_BL}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_H}${BOX_BR}${RESET}"
echo
echo -e "${GRAY}Try it out:${RESET}"
echo -e "  ${CYAN}${ARROW}${RESET} annactl --version"
echo -e "  ${CYAN}${ARROW}${RESET} annactl status"
echo -e "  ${CYAN}${ARROW}${RESET} annactl advise"
echo
