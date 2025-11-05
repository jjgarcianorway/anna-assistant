#!/bin/bash
# Anna Assistant Uninstaller
# Cleanly removes Anna from your system

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
    CHECK="✓"; CROSS="✗"; WARN="⚠"; INFO="ℹ"; ARROW="→"
else
    # ASCII fallback
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    CHECK="[OK]"; CROSS="[X]"; WARN="[!]"; INFO="[i]"; ARROW="->"
fi

print_header() {
    echo
    echo -e "${BOLD}${CYAN}╭──────────────────────────────────────────────────────╮${RESET}"
    echo -e "${BOLD}${CYAN}│${RESET}        ${BOLD}${RED}Anna Assistant${RESET} ${CYAN}Uninstaller${RESET}          ${BOLD}${CYAN}│${RESET}"
    echo -e "${BOLD}${CYAN}╰──────────────────────────────────────────────────────╯${RESET}"
    echo
}

error_exit() {
    echo -e "${RED}${CROSS} $1${RESET}" >&2
    exit 1
}

print_header

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}${CROSS} This uninstaller requires root privileges${RESET}" >&2
    echo >&2
    echo -e "${GRAY}Please run with sudo:${RESET}" >&2
    echo -e "  ${CYAN}curl -sSL https://raw.githubusercontent.com/${REPO}/main/scripts/uninstall.sh | sudo sh${RESET}" >&2
    echo >&2
    exit 1
fi

# Ask for confirmation
echo -e "${YELLOW}${WARN}${RESET} ${BOLD}This will remove Anna Assistant from your system${RESET}"
echo
echo -e "${GRAY}The following will be removed:${RESET}"
echo -e "  ${ARROW} Daemon and client binaries"
echo -e "  ${ARROW} Systemd service"
echo -e "  ${ARROW} User data and configuration ${RED}(your settings and history will be lost!)${RESET}"
echo

read -p "$(echo -e ${YELLOW}Are you sure you want to uninstall? [y/N]: ${RESET})" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${CYAN}${INFO}${RESET} Uninstall cancelled"
    exit 0
fi

echo
echo -e "${CYAN}${ARROW}${RESET} Beginning uninstallation..."
echo

# Stop and disable service
if systemctl is-active --quiet annad 2>/dev/null; then
    echo -e "${CYAN}${ARROW}${RESET} Stopping annad service..."
    systemctl stop annad
    echo -e "${GREEN}${CHECK}${RESET} Service stopped"
fi

if systemctl is-enabled --quiet annad 2>/dev/null; then
    echo -e "${CYAN}${ARROW}${RESET} Disabling annad service..."
    systemctl disable annad
    echo -e "${GREEN}${CHECK}${RESET} Service disabled"
fi

# Kill any remaining processes
pkill -x annad 2>/dev/null && echo -e "${GREEN}${CHECK}${RESET} Stopped annad process" || true
pkill -x annactl 2>/dev/null || true

# Wait for processes to fully stop
sleep 1

# Remove systemd service
if [ -f /etc/systemd/system/annad.service ]; then
    echo -e "${CYAN}${ARROW}${RESET} Removing systemd service..."
    rm -f /etc/systemd/system/annad.service
    systemctl daemon-reload
    echo -e "${GREEN}${CHECK}${RESET} Service file removed"
fi

# Remove binaries
echo -e "${CYAN}${ARROW}${RESET} Removing binaries..."
rm -f "${INSTALL_DIR}/annad"
rm -f "${INSTALL_DIR}/annactl"
echo -e "${GREEN}${CHECK}${RESET} Binaries removed"

# Ask about user data
echo
echo -e "${YELLOW}${WARN}${RESET} ${BOLD}Remove user data?${RESET}"
echo
echo -e "${GRAY}This will delete:${RESET}"
echo -e "  ${ARROW} Configuration (${CYAN}/etc/anna/${RESET})"
echo -e "  ${ARROW} Logs (${CYAN}/var/log/anna/${RESET})"
echo -e "  ${ARROW} Runtime data (${CYAN}/run/anna/${RESET})"
echo -e "  ${ARROW} Cache (${CYAN}/var/cache/anna/${RESET})"
echo

read -p "$(echo -e ${YELLOW}Remove user data? [y/N]: ${RESET})" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo
    echo -e "${CYAN}${ARROW}${RESET} Removing user data..."

    if [ -d /etc/anna ]; then
        rm -rf /etc/anna
        echo -e "${GREEN}${CHECK}${RESET} Configuration removed"
    fi

    if [ -d /var/log/anna ]; then
        rm -rf /var/log/anna
        echo -e "${GREEN}${CHECK}${RESET} Logs removed"
    fi

    if [ -d /run/anna ]; then
        rm -rf /run/anna
        echo -e "${GREEN}${CHECK}${RESET} Runtime data removed"
    fi

    if [ -d /var/cache/anna ]; then
        rm -rf /var/cache/anna
        echo -e "${GREEN}${CHECK}${RESET} Cache removed"
    fi
else
    echo
    echo -e "${CYAN}${INFO}${RESET} User data preserved"
    echo -e "${GRAY}  You can manually remove it later from:${RESET}"
    echo -e "${GRAY}  - /etc/anna/${RESET}"
    echo -e "${GRAY}  - /var/log/anna/${RESET}"
    echo -e "${GRAY}  - /var/cache/anna/${RESET}"
fi

echo
echo -e "${BOLD}${GREEN}╭──────────────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${GREEN}│${RESET}      Anna Assistant Successfully Uninstalled      ${BOLD}${GREEN}│${RESET}"
echo -e "${BOLD}${GREEN}╰──────────────────────────────────────────────────────╯${RESET}"
echo
echo -e "${GRAY}Thanks for using Anna! We're sorry to see you go.${RESET}"
echo
echo -e "${CYAN}${INFO}${RESET} ${BOLD}To reinstall Anna in the future:${RESET}"
echo -e "  ${CYAN}curl -sSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | sudo sh${RESET}"
echo
echo -e "${CYAN}${INFO}${RESET} ${BOLD}Got feedback? We'd love to hear it:${RESET}"
echo -e "  ${CYAN}https://github.com/${REPO}/issues${RESET}"
echo
