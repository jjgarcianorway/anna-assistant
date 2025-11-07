#!/bin/bash
# Anna Assistant - One-line Installer
# curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

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
    # ASCII fallback
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    CHECK="[OK]"; CROSS="[X]"; ARROW="->"
fi

error_exit() {
    echo -e "${RED}${CROSS} $1${RESET}" >&2
    exit 1
}

# Header
echo
echo -e "${BOLD}${CYAN}╭──────────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${CYAN}│${RESET}  ${BOLD}${BLUE}🌟 Anna Assistant${RESET} ${GRAY}- Your Friendly Arch Admin${RESET}  ${BOLD}${CYAN}│${RESET}"
echo -e "${BOLD}${CYAN}╰──────────────────────────────────────────────────╯${RESET}"
echo

# Brief intro
echo -e "${GRAY}Anna monitors your Arch Linux system and provides personalized${RESET}"
echo -e "${GRAY}recommendations for security, performance, and configuration.${RESET}"
echo

# Check if already installed
CURRENT_VERSION=""
if command -v annad >/dev/null 2>&1; then
    CURRENT_VERSION=$(annad --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "")
fi

# Check sudo early
command -v sudo >/dev/null 2>&1 || error_exit "sudo required"

# Check dependencies for fetching release info
MISSING_DEPS=()
command -v curl >/dev/null 2>&1 || MISSING_DEPS+=("curl")
command -v jq >/dev/null 2>&1 || MISSING_DEPS+=("jq")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}${ARROW}${RESET} Installing required tools: ${MISSING_DEPS[*]}"
    sudo pacman -Sy --noconfirm "${MISSING_DEPS[@]}" >/dev/null 2>&1 || \
        error_exit "Failed to install: ${MISSING_DEPS[*]}"
    echo -e "${GREEN}${CHECK}${RESET} Tools installed"
    echo
fi

# Fetch latest release
echo -e "${CYAN}${ARROW}${RESET} Checking latest version..."
RELEASE_JSON=$(curl -s "https://api.github.com/repos/${REPO}/releases" | jq '.[0]')
TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')
[ "$TAG" != "null" ] && [ -n "$TAG" ] || error_exit "No releases found"
NEW_VERSION=$(echo "$TAG" | sed 's/^v//')
echo -e "${GREEN}${CHECK}${RESET} Latest version: ${BOLD}${TAG}${RESET}"
echo

# Show installation plan
if [ -n "$CURRENT_VERSION" ]; then
    echo -e "${BOLD}${YELLOW}Update Plan:${RESET}"
    echo -e "  Current version: ${CYAN}v${CURRENT_VERSION}${RESET}"
    echo -e "  New version:     ${GREEN}${TAG}${RESET}"

    if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
        echo
        echo -e "${YELLOW}${ARROW}${RESET} ${GRAY}Already on latest version, will reinstall${RESET}"
    fi
else
    echo -e "${BOLD}${GREEN}Installation Plan:${RESET}"
    echo -e "  Version: ${GREEN}${TAG}${RESET}"
fi

echo
echo -e "${BOLD}What will be done:${RESET}"
echo -e "  ${ARROW} Install ${CYAN}annad${RESET} and ${CYAN}annactl${RESET} to ${INSTALL_DIR}"
echo -e "  ${ARROW} Install systemd service (${CYAN}annad.service${RESET})"
echo -e "  ${ARROW} Enable and start the daemon"
echo -e "  ${ARROW} Install shell completions (bash/zsh/fish)"
echo

# Confirmation
read -p "$(echo -e ${BOLD}${GREEN}Continue with installation? [y/N]:${RESET} )" -r < /dev/tty
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${GRAY}Installation cancelled${RESET}"
    exit 0
fi

# Architecture check
ARCH=$(uname -m)
[ "$ARCH" = "x86_64" ] || error_exit "Only x86_64 supported"

# Get download URLs
ANNAD_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name != null and (.name == "annad" or (.name | startswith("annad-")))) | .browser_download_url' | head -1)
ANNACTL_URL=$(echo "$RELEASE_JSON" | jq -r '.assets[] | select(.name != null and (.name == "annactl" or (.name | startswith("annactl-")))) | .browser_download_url' | head -1)
[ -n "$ANNAD_URL" ] && [ -n "$ANNACTL_URL" ] || error_exit "Release assets not found"

# Download
echo -e "${CYAN}${ARROW}${RESET} Downloading binaries..."
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT
curl -fsSL -o "$TEMP_DIR/annad" "$ANNAD_URL" || error_exit "Download failed"
curl -fsSL -o "$TEMP_DIR/annactl" "$ANNACTL_URL" || error_exit "Download failed"
chmod +x "$TEMP_DIR/annad" "$TEMP_DIR/annactl"
echo -e "${GREEN}${CHECK}${RESET} Downloaded successfully"

# Stop running instances
echo -e "${CYAN}${ARROW}${RESET} Stopping running instances..."
sudo systemctl stop annad 2>/dev/null || true
sudo pkill -x annad 2>/dev/null || true
sudo pkill -x annactl 2>/dev/null || true
sleep 1
echo -e "${GREEN}${CHECK}${RESET} Stopped"

# Install binaries
echo -e "${CYAN}${ARROW}${RESET} Installing to ${INSTALL_DIR}..."
sudo mkdir -p "$INSTALL_DIR"
sudo cp "$TEMP_DIR/annad" "$INSTALL_DIR/annad"
sudo cp "$TEMP_DIR/annactl" "$INSTALL_DIR/annactl"
sudo chmod 755 "$INSTALL_DIR/annad" "$INSTALL_DIR/annactl"
echo -e "${GREEN}${CHECK}${RESET} Binaries installed"

# Shell completions (silent)
echo -e "${CYAN}${ARROW}${RESET} Installing shell completions..."
COMP_COUNT=0
if [ -d "/usr/share/bash-completion/completions" ]; then
    "$INSTALL_DIR/annactl" completions bash 2>/dev/null | sudo tee /usr/share/bash-completion/completions/annactl > /dev/null 2>&1 && COMP_COUNT=$((COMP_COUNT + 1))
fi
if [ -d "/usr/share/zsh/site-functions" ]; then
    "$INSTALL_DIR/annactl" completions zsh 2>/dev/null | sudo tee /usr/share/zsh/site-functions/_annactl > /dev/null 2>&1 && COMP_COUNT=$((COMP_COUNT + 1))
fi
if [ -d "/usr/share/fish/vendor_completions.d" ]; then
    "$INSTALL_DIR/annactl" completions fish 2>/dev/null | sudo tee /usr/share/fish/vendor_completions.d/annactl.fish > /dev/null 2>&1 && COMP_COUNT=$((COMP_COUNT + 1))
fi
echo -e "${GREEN}${CHECK}${RESET} Completions installed (${COMP_COUNT} shells)"

# Systemd service
echo -e "${CYAN}${ARROW}${RESET} Installing systemd service..."
curl -fsSL -o "$TEMP_DIR/annad.service" "https://raw.githubusercontent.com/${REPO}/main/annad.service" || error_exit "Failed to download service"
sudo cp "$TEMP_DIR/annad.service" /etc/systemd/system/annad.service
sudo chmod 644 /etc/systemd/system/annad.service
sudo systemctl daemon-reload
echo -e "${GREEN}${CHECK}${RESET} Service installed"

# Enable and start
echo -e "${CYAN}${ARROW}${RESET} Starting daemon..."
if systemctl is-enabled --quiet annad 2>/dev/null; then
    sudo systemctl restart annad
else
    sudo systemctl enable --now annad
fi
echo -e "${GREEN}${CHECK}${RESET} Daemon running"

# Success message
echo
echo -e "${BOLD}${GREEN}╭──────────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${GREEN}│${RESET}      ${BOLD}${GREEN}✓ Installation Complete!${RESET} ${BOLD}${TAG}${RESET}           ${BOLD}${GREEN}│${RESET}"
echo -e "${BOLD}${GREEN}╰──────────────────────────────────────────────────╯${RESET}"
echo
echo -e "${BOLD}${CYAN}Quick Start:${RESET}"
echo -e "  ${CYAN}annactl advise${RESET}  ${GRAY}# Get personalized recommendations${RESET}"
echo -e "  ${CYAN}annactl status${RESET}  ${GRAY}# Check system health${RESET}"
echo -e "  ${CYAN}annactl report${RESET}  ${GRAY}# Full system report${RESET}"
echo
echo -e "${GRAY}${ARROW} Full docs: ${CYAN}https://github.com/${REPO}${RESET}"
echo
