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

# Shell completions (check if already installed)
COMP_EXISTS=0
COMP_INSTALLED=0
[ -f "/usr/share/bash-completion/completions/annactl" ] && COMP_EXISTS=$((COMP_EXISTS + 1))
[ -f "/usr/share/zsh/site-functions/_annactl" ] && COMP_EXISTS=$((COMP_EXISTS + 1))
[ -f "/usr/share/fish/vendor_completions.d/annactl.fish" ] && COMP_EXISTS=$((COMP_EXISTS + 1))

if [ "$COMP_EXISTS" -gt 0 ]; then
    echo -e "${GREEN}${CHECK}${RESET} Shell completions already installed (${COMP_EXISTS} shells)"
else
    echo -e "${CYAN}${ARROW}${RESET} Installing shell completions..."
    if [ -d "/usr/share/bash-completion/completions" ]; then
        "$INSTALL_DIR/annactl" completions bash 2>/dev/null | sudo tee /usr/share/bash-completion/completions/annactl > /dev/null 2>&1 && COMP_INSTALLED=$((COMP_INSTALLED + 1))
    fi
    if [ -d "/usr/share/zsh/site-functions" ]; then
        "$INSTALL_DIR/annactl" completions zsh 2>/dev/null | sudo tee /usr/share/zsh/site-functions/_annactl > /dev/null 2>&1 && COMP_INSTALLED=$((COMP_INSTALLED + 1))
    fi
    if [ -d "/usr/share/fish/vendor_completions.d" ]; then
        "$INSTALL_DIR/annactl" completions fish 2>/dev/null | sudo tee /usr/share/fish/vendor_completions.d/annactl.fish > /dev/null 2>&1 && COMP_INSTALLED=$((COMP_INSTALLED + 1))
    fi
    echo -e "${GREEN}${CHECK}${RESET} Completions installed (${COMP_INSTALLED} shells)"
fi

# Phase 0.4: Security setup - create anna group and secure directories
echo -e "${CYAN}${ARROW}${RESET} Setting up security configuration..."
curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/scripts/setup-security.sh" | sudo bash || error_exit "Failed to setup security"
echo -e "${GREEN}${CHECK}${RESET} Security configured"

# LLM Setup - Ollama installation and model configuration
echo
echo -e "${BOLD}${BLUE}LLM Setup${RESET}"
echo -e "${GRAY}────────────────────────────────────────────────────${RESET}"
echo
echo "Anna requires an LLM (Language Model) to understand natural language."
echo "Let me set up Ollama with a local model suitable for your hardware."
echo

# Detect hardware
echo -e "${CYAN}${ARROW}${RESET} Detecting hardware..."
CPU_CORES=$(nproc 2>/dev/null || echo "4")
TOTAL_RAM_KB=$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}')
TOTAL_RAM_GB=$((TOTAL_RAM_KB / 1024 / 1024))
HAS_GPU=0
if lspci 2>/dev/null | grep -iE "(VGA|3D|Display).*NVIDIA" >/dev/null; then
    HAS_GPU=1
fi

echo -e "${GREEN}${CHECK}${RESET} CPU: ${CPU_CORES} cores"
echo -e "${GREEN}${CHECK}${RESET} RAM: ${TOTAL_RAM_GB} GB"
if [ "$HAS_GPU" -eq 1 ]; then
    echo -e "${GREEN}${CHECK}${RESET} GPU: NVIDIA detected"
else
    echo -e "${GRAY}  No GPU detected (CPU-only mode)${RESET}"
fi
echo

# Select appropriate model based on hardware
if [ "$TOTAL_RAM_GB" -ge 16 ] && [ "$HAS_GPU" -eq 1 ] && [ "$CPU_CORES" -ge 8 ]; then
    MODEL="llama3.1:8b"
    echo -e "${CYAN}Selected model:${RESET} ${BOLD}${MODEL}${RESET} ${GREEN}(8B parameter model - powerful)${RESET}"
    echo -e "${GRAY}  RAM: ${TOTAL_RAM_GB}GB | GPU: NVIDIA | Cores: ${CPU_CORES}${RESET}"
elif [ "$TOTAL_RAM_GB" -ge 8 ]; then
    MODEL="llama3.2:3b"
    echo -e "${CYAN}Selected model:${RESET} ${MODEL} (3B parameter model)"
else
    MODEL="llama3.2:1b"
    echo -e "${CYAN}Selected model:${RESET} ${MODEL} (lightweight 1B model)"
fi
echo

# Check if Ollama is installed
if ! command -v ollama >/dev/null 2>&1; then
    echo -e "${CYAN}${ARROW}${RESET} Installing Ollama..."

    # Install Ollama using official installer
    if curl -fsSL https://ollama.com/install.sh | sh; then
        echo -e "${GREEN}${CHECK}${RESET} Ollama installed"
    else
        echo
        error_exit "Failed to install Ollama. Anna requires an LLM to function."
    fi
else
    echo -e "${GREEN}${CHECK}${RESET} Ollama already installed"
fi

# Start Ollama service
echo -e "${CYAN}${ARROW}${RESET} Starting Ollama service..."
sudo systemctl enable ollama 2>/dev/null || true
sudo systemctl start ollama 2>/dev/null || true

# Wait for Ollama to be ready
sleep 2

# Pull the selected model
echo
echo -e "${CYAN}${ARROW}${RESET} Downloading LLM model: ${MODEL}"
echo -e "${GRAY}This may take a few minutes depending on your connection...${RESET}"

ollama pull "$MODEL" 2>&1 | tee /tmp/ollama-pull.log
PULL_EXIT_CODE=${PIPESTATUS[0]}

if [ $PULL_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}${CHECK}${RESET} Model downloaded successfully"
else
    echo
    echo -e "${RED}${CROSS}${RESET} Failed to download LLM model (exit code: $PULL_EXIT_CODE)"
    if grep -qi "cloudflare\|500\|error" /tmp/ollama-pull.log 2>/dev/null; then
        echo -e "${YELLOW}⚠${RESET}  Detected network/server error. You can:"
        echo "  1. Try again later: ollama pull $MODEL"
        echo "  2. Continue without LLM (limited functionality)"
        echo
        read -p "Continue without LLM? [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            error_exit "Installation cancelled"
        fi
    else
        error_exit "Failed to download LLM model. Anna requires an LLM to function."
    fi
fi

# Verify model is available
echo -e "${CYAN}${ARROW}${RESET} Verifying model..."
if ollama list | grep -q "$MODEL"; then
    echo -e "${GREEN}${CHECK}${RESET} Model ${MODEL} is ready"
else
    error_exit "Model verification failed"
fi

# Configure Anna to use this model
echo -e "${CYAN}${ARROW}${RESET} Configuring Anna to use ${MODEL}..."
# This will be done by annad on first startup via the database
# For now, we just verify Ollama is working
if curl -s -f http://localhost:11434/api/version >/dev/null 2>&1; then
    echo -e "${GREEN}${CHECK}${RESET} Ollama API is responding"
else
    echo -e "${YELLOW}⚠${RESET}  Ollama API not responding (will retry on startup)"
fi

echo
echo -e "${GREEN}${CHECK}${RESET} LLM setup complete"
echo

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

# Wait for daemon to be ready and verify health
echo -e "${CYAN}${ARROW}${RESET} Waiting for daemon to be ready..."
READY=0
WAIT_SECS=30
for i in $(seq 1 $WAIT_SECS); do
    if [ -S /run/anna/anna.sock ]; then
        # Check health using annactl status
        if "$INSTALL_DIR/annactl" status >/dev/null 2>&1; then
            READY=1
            echo -e "${GREEN}${CHECK}${RESET} Daemon ready and healthy (${i}s)"
            break
        fi
    fi
    sleep 1
done

if [ "$READY" -ne 1 ]; then
    echo
    echo -e "${RED}${CROSS}${RESET} ${BOLD}Daemon failed health check${RESET}"
    echo
    echo -e "${YELLOW}Troubleshooting:${RESET}"
    echo -e "  1. Check daemon status: ${CYAN}systemctl status annad${RESET}"
    echo -e "  2. View logs: ${CYAN}journalctl -u annad -n 50${RESET}"
    echo -e "  3. Verify permissions: ${CYAN}ls -la /run/anna${RESET}"
    echo
    echo -e "If you're in a new shell, you may need to reload your groups:"
    echo -e "  ${CYAN}newgrp anna${RESET}"
    echo
    error_exit "Installation completed but daemon is not healthy"
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
