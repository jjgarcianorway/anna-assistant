#!/bin/bash
# Anna Assistant - Self-Update Script
# Updates annad and annactl to the latest GitHub release
#
# Usage: scripts/self_update.sh
#    Or: annactl self-update (wrapper)

set -e

REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"

# Colors
if [ -t 1 ] && command -v tput >/dev/null 2>&1 && [ "$(tput colors)" -ge 256 ]; then
    BLUE='\033[38;5;117m'
    GREEN='\033[38;5;120m'
    YELLOW='\033[38;5;228m'
    RED='\033[38;5;210m'
    CYAN='\033[38;5;159m'
    GRAY='\033[38;5;250m'
    RESET='\033[0m'
    BOLD='\033[1m'
    CHECK="✓"; CROSS="✗"; ARROW="→"
else
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    CHECK="[OK]"; CROSS="[X]"; ARROW="->"
fi

error_exit() {
    echo -e "${RED}${CROSS} $1${RESET}" >&2
    exit 1
}

# Check if running as root or can sudo
if [ "$EUID" -ne 0 ]; then
    command -v sudo >/dev/null 2>&1 || error_exit "sudo required for self-update"
fi

# Check dependencies
MISSING_DEPS=()
command -v curl >/dev/null 2>&1 || MISSING_DEPS+=("curl")
command -v jq >/dev/null 2>&1 || MISSING_DEPS+=("jq")
command -v sha256sum >/dev/null 2>&1 || MISSING_DEPS+=("coreutils")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}${ARROW}${RESET} Installing required tools: ${MISSING_DEPS[*]}"
    sudo pacman -Sy --noconfirm "${MISSING_DEPS[@]}" >/dev/null 2>&1 || \
        error_exit "Failed to install: ${MISSING_DEPS[*]}"
fi

# Get current version
CURRENT_VERSION=""
if command -v annad >/dev/null 2>&1; then
    CURRENT_VERSION=$(annad --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
fi

echo
echo -e "${BOLD}${CYAN}Anna Assistant Self-Update${RESET}"
echo -e "${GRAY}Current version: ${CURRENT_VERSION}${RESET}"
echo

# Fetch latest release
echo -e "${CYAN}${ARROW}${RESET} Checking for updates..."
RELEASE_JSON=$(curl -s "https://api.github.com/repos/${REPO}/releases" | jq '.[0]')
TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')
[ "$TAG" != "null" ] && [ -n "$TAG" ] || error_exit "No releases found"
NEW_VERSION=$(echo "$TAG" | sed 's/^v//')

echo -e "${GREEN}${CHECK}${RESET} Latest version: ${BOLD}${TAG}${RESET}"

# Check if already up to date
if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    echo
    echo -e "${GREEN}${CHECK}${RESET} ${BOLD}Already up to date!${RESET}"
    exit 0
fi

echo
echo -e "${BOLD}${GREEN}Update available:${RESET} ${CYAN}v${CURRENT_VERSION}${RESET} ${ARROW} ${GREEN}${TAG}${RESET}"
echo

# Get download URLs
echo -e "${CYAN}${ARROW}${RESET} Fetching release artifacts..."
ANNAD_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name != null and (.name == "annad" or (.name | startswith("annad-")))) | .browser_download_url' | head -1)
ANNACTL_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name != null and (.name == "annactl" or (.name | startswith("annactl-")))) | .browser_download_url' | head -1)
CHECKSUM_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name == "SHA256SUMS") | .browser_download_url' | head -1)

[ -n "$ANNAD_URL" ] && [ -n "$ANNACTL_URL" ] || error_exit "Release assets not found"

# Download binaries
echo -e "${CYAN}${ARROW}${RESET} Downloading binaries..."
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

curl -fsSL -o "$TEMP_DIR/annad" "$ANNAD_URL" || error_exit "Download failed: annad"
curl -fsSL -o "$TEMP_DIR/annactl" "$ANNACTL_URL" || error_exit "Download failed: annactl"

# Download and verify checksums if available
if [ -n "$CHECKSUM_URL" ]; then
    echo -e "${CYAN}${ARROW}${RESET} Verifying checksums..."
    curl -fsSL -o "$TEMP_DIR/SHA256SUMS" "$CHECKSUM_URL" || echo -e "${YELLOW}⚠${RESET}  Checksums not available"

    if [ -f "$TEMP_DIR/SHA256SUMS" ]; then
        cd "$TEMP_DIR"
        if sha256sum -c SHA256SUMS 2>/dev/null | grep -q "OK"; then
            echo -e "${GREEN}${CHECK}${RESET} Checksums verified"
        else
            error_exit "Checksum verification failed"
        fi
        cd - >/dev/null
    fi
fi

chmod +x "$TEMP_DIR/annad" "$TEMP_DIR/annactl"
echo -e "${GREEN}${CHECK}${RESET} Downloaded successfully"

# Stop daemon
echo -e "${CYAN}${ARROW}${RESET} Stopping daemon..."
sudo systemctl stop annad 2>/dev/null || true
sleep 1
echo -e "${GREEN}${CHECK}${RESET} Stopped"

# Atomic installation using mv (POSIX atomicity guarantee)
echo -e "${CYAN}${ARROW}${RESET} Installing update..."
sudo mv "$TEMP_DIR/annad" "$INSTALL_DIR/annad"
sudo mv "$TEMP_DIR/annactl" "$INSTALL_DIR/annactl"
sudo chmod 755 "$INSTALL_DIR/annad" "$INSTALL_DIR/annactl"
echo -e "${GREEN}${CHECK}${RESET} Installed"

# Restart daemon
echo -e "${CYAN}${ARROW}${RESET} Starting daemon..."
sudo systemctl start annad

# Wait for socket
echo -e "${CYAN}${ARROW}${RESET} Waiting for daemon..."
READY=0
for i in $(seq 1 30); do
    if [ -S /run/anna/anna.sock ]; then
        if "$INSTALL_DIR/annactl" help >/dev/null 2>&1; then
            READY=1
            echo -e "${GREEN}${CHECK}${RESET} Daemon ready"
            break
        fi
    fi
    sleep 1
done

if [ "$READY" -ne 1 ]; then
    echo -e "${YELLOW}⚠${RESET}  Daemon socket not reachable - check logs: journalctl -u annad"
fi

# Success
echo
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo -e "${BOLD}${GREEN}  ✓ Update Complete! ${TAG}${RESET}"
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo
echo -e "${GRAY}Run ${CYAN}annactl --version${GRAY} to confirm${RESET}"
echo
