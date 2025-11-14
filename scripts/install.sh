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

# Clean header
echo
echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo -e "${BOLD}${BLUE}    🌟 Anna Assistant Installer${RESET}"
echo -e "${GRAY}    Your Friendly Arch Linux System Administrator${RESET}"
echo -e "${BOLD}${CYAN}====================================================${RESET}"
echo

# Get username for personalized greeting
USERNAME=${SUDO_USER:-${USER}}

# Warm greeting (Phase 5.1: Conversational UX)
echo -e "${BOLD}Hello ${GREEN}${USERNAME}${RESET}${BOLD},${RESET}"
echo
echo "Thank you for giving me the chance to live on your computer 😉"
echo
echo "My name is Anna and my main goal is to be a bridge between"
echo "the technical documentation and you, only for this machine:"
echo "your hardware, software and how you actually use it."
echo

echo -e "${BOLD}${BLUE}How do I work?${RESET}"
echo
echo "- I watch your system locally."
echo "- I compare what I see with best practices from the Arch Wiki."
echo "- I suggest improvements, explain them in plain English,"
echo "  and only change things after you approve them."
echo

echo -e "${BOLD}${BLUE}What about privacy?${RESET}"
echo
echo "- I do not send your data anywhere."
echo "- I keep telemetry and notes on this machine only."
echo "- I read the Arch Wiki and official documentation when needed."
echo "- I never run commands behind your back."
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
        echo -e "${YELLOW}${ARROW}${RESET} ${BOLD}Already on version ${TAG}${RESET}"
        echo
        read -p "$(echo -e ${BOLD}${YELLOW}Reinstall anyway? [y/N]:${RESET} )" -r < /dev/tty
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${GRAY}Installation cancelled - already up to date${RESET}"
            exit 0
        fi
        echo
    fi
else
    echo -e "${BOLD}${GREEN}Installation Plan:${RESET}"
    echo -e "  Version: ${GREEN}${TAG}${RESET}"
fi

# Show release notes
RELEASE_NOTES=$(echo "$RELEASE_JSON" | jq -r '.body // empty' | head -20)
if [ -n "$RELEASE_NOTES" ]; then
    echo
    echo -e "${BOLD}${CYAN}What's New in ${TAG}:${RESET}"
    echo -e "${GRAY}────────────────────────────────────────────────────${RESET}"
    echo "$RELEASE_NOTES" | head -15
    echo -e "${GRAY}────────────────────────────────────────────────────${RESET}"
fi

echo
echo -e "${BOLD}This is what I am doing now:${RESET}"
echo -e "  ${ARROW} Installing binaries (${CYAN}annad${RESET} and ${CYAN}annactl${RESET}) to ${INSTALL_DIR}"
echo -e "  ${ARROW} Setting up systemd service"
echo -e "  ${ARROW} Verifying permissions and groups"
echo -e "  ${ARROW} Installing shell completions"
echo

# Warm confirmation prompt
echo -e "${BOLD}Do you want me to continue with the installation and setup? [y/N]:${RESET} "
read -r REPLY < /dev/tty
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo
    echo -e "${GRAY}No problem! If you change your mind, just run this installer again.${RESET}"
    echo -e "${GRAY}Have a great day, ${USERNAME}!${RESET}"
    echo
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

# Stop running instances and clean up old daemon binaries (rc.13.1 compatibility fix)
echo -e "${CYAN}${ARROW}${RESET} Stopping running instances..."
sudo systemctl stop annad 2>/dev/null || true
sudo pkill -x annad 2>/dev/null || true
sudo pkill -x annactl 2>/dev/null || true
sudo rm -f /usr/local/bin/annad-old /usr/local/bin/annactl-old 2>/dev/null || true
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

# Phase 0.4: Security setup - create anna group and secure directories
echo -e "${CYAN}${ARROW}${RESET} Setting up security configuration..."
curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/scripts/setup-security.sh" | sudo bash || error_exit "Failed to setup security"
echo -e "${GREEN}${CHECK}${RESET} Security configured"

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

# Wait for daemon to be ready (up to 30 seconds) - rc.13.1 improved readiness check
echo -e "${CYAN}${ARROW}${RESET} Waiting for daemon to be ready..."
READY=0
WAIT_SECS=30
for i in $(seq 1 $WAIT_SECS); do
    if [ -S /run/anna/anna.sock ]; then
        # Try a lightweight call; help will return success or permission error
        if "$INSTALL_DIR/annactl" help >/dev/null 2>&1; then
            READY=1
            echo -e "${GREEN}${CHECK}${RESET} Daemon ready (${i}s)"
            break
        fi
    fi
    sleep 1
done

if [ "$READY" -ne 1 ]; then
    echo -e "${YELLOW}⚠${RESET}  Daemon socket not reachable after ${WAIT_SECS}s"
fi

# Group access guidance (rc.13.1 user experience improvement)
if ! id -nG "$USER" 2>/dev/null | tr ' ' '\n' | grep -qx anna; then
    echo
    echo -e "${BOLD}${YELLOW}[INFO]${RESET} Your user is not in the 'anna' group."
    echo -e "       To enable socket access now, run:"
    echo -e "       ${CYAN}sudo usermod -aG anna \"$USER\" && newgrp anna${RESET}"
fi

# Success banner
echo
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo -e "${BOLD}${GREEN}  ✓ Installation Complete! ${TAG}${RESET}"
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo
echo -e "${BOLD}${CYAN}Let's Get Started:${RESET}"
echo
echo "Just run:"
echo -e "  ${CYAN}annactl${RESET}"
echo
echo "Then you can talk to me naturally, for example:"
echo -e "  ${GRAY}\"Anna, can you tell me my average CPU usage in the last 3 days\"${RESET}"
echo -e "  ${GRAY}\"Anna, my computer feels slower than usual, did you see any reason\"${RESET}"
echo -e "  ${GRAY}\"How are you, any problems with my system\"${RESET}"
echo -e "  ${GRAY}\"What do you store about me\"${RESET}"
echo
echo -e "${BOLD}I'm ready to help you keep this machine healthy!${RESET}"
echo
echo -e "${GRAY}${ARROW} Full docs: ${CYAN}https://github.com/${REPO}${RESET}"
echo
