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
    echo
    echo -e "${BOLD}${CYAN}╭──────────────────────────────────────────────────────╮${RESET}"
    echo -e "${BOLD}${CYAN}│${RESET}     ${BOLD}${BLUE}🌟 Anna Assistant${RESET} ${CYAN}Installation${RESET}        ${BOLD}${CYAN}│${RESET}"
    echo -e "${BOLD}${CYAN}├──────────────────────────────────────────────────────┤${RESET}"
    echo -e "${BOLD}${CYAN}│${RESET}  ${GRAY}Your friendly Arch Linux system administrator${RESET}   ${BOLD}${CYAN}│${RESET}"
    echo -e "${BOLD}${CYAN}╰──────────────────────────────────────────────────────╯${RESET}"
    echo
    echo -e "${GRAY}Anna speaks plain English, explains everything she suggests,"
    echo -e "and keeps your system secure, fast, and well-maintained.${RESET}"
    echo
    echo -e "${BOLD}${BLUE}What Anna does (130+ detection rules):${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}System security (SSH hardening, firewall, microcode, updates)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}8 desktop environments (GNOME, KDE, Cinnamon, XFCE, MATE, i3, Hyprland, Sway)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}Hardware support (printers, webcams, gamepads, Bluetooth, WiFi)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}Development tools (Docker, virtualization, LSP servers, gaming)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}Laptop optimization (battery, touchpad, backlight, power management)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}Privacy & security (VPN, password managers, backups, encryption)${RESET}"
    echo -e "  ${GREEN}${CHECK}${RESET} ${GRAY}Automatically monitors and refreshes on system changes${RESET}"
    echo
    echo -e "${CYAN}${ARROW}${RESET} ${BOLD}Starting installation...${RESET}"
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

# Check and install dependencies
echo -e "${CYAN}${ARROW}${RESET} Checking dependencies..."

MISSING_DEPS=()

if ! command -v curl >/dev/null 2>&1; then
    MISSING_DEPS+=("curl")
fi

if ! command -v jq >/dev/null 2>&1; then
    MISSING_DEPS+=("jq")
fi

if ! command -v tar >/dev/null 2>&1; then
    MISSING_DEPS+=("tar")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}${WARN}${RESET} Missing dependencies: ${MISSING_DEPS[*]}"
    echo -e "${CYAN}${ARROW}${RESET} Installing missing packages..."

    if pacman -Sy --noconfirm "${MISSING_DEPS[@]}" 2>/dev/null; then
        echo -e "${GREEN}${CHECK}${RESET} Dependencies installed"
    else
        error_exit "Failed to install dependencies. Please install manually: pacman -S ${MISSING_DEPS[*]}"
    fi
else
    echo -e "${GREEN}${CHECK}${RESET} All dependencies satisfied"
fi

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

# Verify version matches (disable errexit temporarily for version checks)
set +e
ANNAD_VERSION=$("$TEMP_DIR/annad" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?')
if [ -z "$ANNAD_VERSION" ]; then
    ANNAD_VERSION="unknown"
else
    # Add v prefix if not present
    if [[ ! "$ANNAD_VERSION" =~ ^v ]]; then
        ANNAD_VERSION="v${ANNAD_VERSION}"
    fi
fi

ANNACTL_VERSION=$("$TEMP_DIR/annactl" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?')
if [ -z "$ANNACTL_VERSION" ]; then
    ANNACTL_VERSION="unknown"
else
    # Add v prefix if not present
    if [[ ! "$ANNACTL_VERSION" =~ ^v ]]; then
        ANNACTL_VERSION="v${ANNACTL_VERSION}"
    fi
fi
set -e

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

# Install systemd service
echo -e "${CYAN}${ARROW}${RESET} Installing systemd service..."

# Download service file from GitHub
curl -L -o "$TEMP_DIR/annad.service" "https://raw.githubusercontent.com/${REPO}/main/annad.service" 2>/dev/null || error_exit "Failed to download service file"

cp "$TEMP_DIR/annad.service" /etc/systemd/system/annad.service
chmod 644 /etc/systemd/system/annad.service

systemctl daemon-reload
echo -e "${GREEN}${CHECK}${RESET} Service file installed"

# Enable and start service
if systemctl is-enabled --quiet annad 2>/dev/null; then
    echo -e "${CYAN}${ARROW}${RESET} Restarting annad service..."
    systemctl restart annad
    echo -e "${GREEN}${CHECK}${RESET} Service restarted"
else
    echo -e "${CYAN}${ARROW}${RESET} Enabling and starting annad service..."
    systemctl enable --now annad
    echo -e "${GREEN}${CHECK}${RESET} Service enabled and started"
fi

# Verify installation
if ! command -v annactl >/dev/null 2>&1; then
    echo -e "${YELLOW}${WARN} annactl not in PATH - you may need to add ${INSTALL_DIR} to PATH${RESET}"
fi

echo

# Calculate box width properly
BOX_WIDTH=60
TITLE_TEXT="   Installation Complete! ${TAG}   "
# Use printf to create properly sized box
printf "${BOLD}${GREEN}%s%s%s${RESET}\n" "${BOX_TL}" "$(printf '%*s' $((BOX_WIDTH-2)) '' | tr ' ' "${BOX_H}")" "${BOX_TR}"
printf "${BOLD}${GREEN}%s%-*s%s${RESET}\n" "${BOX_V}" $((BOX_WIDTH-2)) "${TITLE_TEXT}" "${BOX_V}"
printf "${BOLD}${GREEN}%s%s%s${RESET}\n" "${BOX_BL}" "$(printf '%*s' $((BOX_WIDTH-2)) '' | tr ' ' "${BOX_H}")" "${BOX_BR}"
echo
echo -e "${BOLD}${CYAN}What Anna Can Do:${RESET}"
echo
echo -e "  ${BOLD}${BLUE}🔒 Security${RESET}"
echo -e "    ${GRAY}${ARROW} CPU microcode updates (Spectre/Meltdown protection)${RESET}"
echo -e "    ${GRAY}${ARROW} Missing security patches detection${RESET}"
echo
echo -e "  ${BOLD}${BLUE}⚡ Performance${RESET}"
echo -e "    ${GRAY}${ARROW} Btrfs compression (20-30% space savings)${RESET}"
echo -e "    ${GRAY}${ARROW} SSD TRIM optimization${RESET}"
echo -e "    ${GRAY}${ARROW} pacman parallel downloads (5x faster)${RESET}"
echo
echo -e "  ${BOLD}${BLUE}💻 Development${RESET}"
echo -e "    ${GRAY}${ARROW} Smart detection of active projects (Python, Rust, Go)${RESET}"
echo -e "    ${GRAY}${ARROW} LSP servers for your actual workflow${RESET}"
echo -e "    ${GRAY}${ARROW} Missing config detection (git, bat, starship, zoxide)${RESET}"
echo
echo -e "  ${BOLD}${BLUE}🎨 Beautification${RESET}"
echo -e "    ${GRAY}${ARROW} Modern CLI tools (eza, bat, ripgrep, fd, fzf)${RESET}"
echo -e "    ${GRAY}${ARROW} Shell enhancements (starship, zoxide)${RESET}"
echo -e "    ${GRAY}${ARROW} Colorful terminal output${RESET}"
echo
echo -e "  ${BOLD}${BLUE}🧹 Maintenance${RESET}"
echo -e "    ${GRAY}${ARROW} Orphaned packages cleanup${RESET}"
echo -e "    ${GRAY}${ARROW} System update notifications${RESET}"
echo -e "    ${GRAY}${ARROW} Failed systemd units monitoring${RESET}"
echo -e "    ${GRAY}${ARROW} GPU driver recommendations${RESET}"
echo
echo -e "${BOLD}${YELLOW}🎉 What's New in ${TAG}:${RESET}"
echo
# Fetch release notes from GitHub using jq
RELEASE_DATA=$(curl -sL "https://api.github.com/repos/${REPO}/releases/tags/${TAG}" 2>/dev/null)
RELEASE_NOTES=$(echo "$RELEASE_DATA" | jq -r '.body' 2>/dev/null)

if [ -n "$RELEASE_NOTES" ] && [ "$RELEASE_NOTES" != "null" ]; then
    # Parse and display the release notes with colors
    echo "$RELEASE_NOTES" | while IFS= read -r line; do
        # Headers with emoji
        if echo "$line" | grep -q "^### "; then
            echo -e "${BOLD}${CYAN}$line${RESET}"
        # Bold sections
        elif echo "$line" | grep -q "^\*\*"; then
            echo -e "${BOLD}${GREEN}$line${RESET}"
        # Bullet points
        elif echo "$line" | grep -q "^- "; then
            echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}$line${RESET}" | sed 's/^- //'
        # Regular text
        else
            echo -e "${GRAY}$line${RESET}"
        fi
    done | head -30
    echo
    echo -e "${GRAY}${ARROW} See full changelog: ${CYAN}https://github.com/${REPO}/releases/tag/${TAG}${RESET}"
else
    echo -e "${BOLD}${GREEN}🚀 Major Improvements:${RESET}"
    echo
    echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}Enhanced system intelligence and recommendations${RESET}"
    echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}Improved detection accuracy${RESET}"
    echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}Better Arch Wiki integration${RESET}"
    echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}16 intelligent detection rules${RESET}"
    echo -e "  ${YELLOW}${ARROW}${RESET} ${GRAY}Human-friendly messages throughout${RESET}"
fi
echo
echo -e "${BOLD}${CYAN}Get Started:${RESET}"
echo -e "  ${CYAN}${ARROW}${RESET} annactl status      ${GRAY}# Check daemon status${RESET}"
echo -e "  ${CYAN}${ARROW}${RESET} annactl advise      ${GRAY}# Get personalized recommendations${RESET}"
echo -e "  ${CYAN}${ARROW}${RESET} annactl report      ${GRAY}# System health overview${RESET}"
echo
