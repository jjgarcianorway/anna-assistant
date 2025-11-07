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

# Compact header
echo
echo -e "${BOLD}${CYAN}╭──────────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${CYAN}│${RESET}  ${BOLD}${BLUE}🌟 Anna Assistant${RESET} ${GRAY}- Your Friendly Arch Admin${RESET}  ${BOLD}${CYAN}│${RESET}"
echo -e "${BOLD}${CYAN}╰──────────────────────────────────────────────────╯${RESET}"
echo

# Quick confirmation
echo -e "${YELLOW}${ARROW}${RESET} ${GRAY}Installing to ${INSTALL_DIR} (requires sudo)${RESET}"
read -p "$(echo -e ${BOLD}Continue? [y/N]:${RESET} )" -r < /dev/tty
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${GRAY}Installation cancelled${RESET}"
    exit 0
fi

# Check sudo
command -v sudo >/dev/null 2>&1 || error_exit "sudo required"

# Dependencies (silent install)
echo -e "${CYAN}${ARROW}${RESET} Checking dependencies..."
MISSING_DEPS=()
command -v curl >/dev/null 2>&1 || MISSING_DEPS+=("curl")
command -v jq >/dev/null 2>&1 || MISSING_DEPS+=("jq")
command -v tar >/dev/null 2>&1 || MISSING_DEPS+=("tar")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    sudo pacman -Sy --noconfirm "${MISSING_DEPS[@]}" >/dev/null 2>&1 || \
        error_exit "Failed to install: ${MISSING_DEPS[*]}"
fi
echo -e "${GREEN}${CHECK}${RESET} Dependencies ready"

# Architecture check
ARCH=$(uname -m)
[ "$ARCH" = "x86_64" ] || error_exit "Only x86_64 supported"

# Fetch release
echo -e "${CYAN}${ARROW}${RESET} Fetching latest release..."
RELEASE_JSON=$(curl -s "https://api.github.com/repos/${REPO}/releases" | jq '.[0]')
TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')
[ "$TAG" != "null" ] && [ -n "$TAG" ] || error_exit "No releases found"
echo -e "${GREEN}${CHECK}${RESET} Found ${BOLD}${TAG}${RESET}"

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

# Compact success message
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
