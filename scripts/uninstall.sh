#!/usr/bin/env bash
# Anna v0.0.82 - Uninstall Script
# Removes Anna components, preserves user data optionally
#
# Paths removed (must match installer):
#   /usr/local/bin/annad
#   /usr/local/bin/annactl
#   /etc/systemd/system/annad.service
#   /etc/anna/ (optional)
#   /var/lib/anna/ (optional)
#   /var/log/anna/ (optional)
#   /run/anna/ (optional)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Icons
ICON_CHECK="✓"
ICON_CROSS="✗"
ICON_INFO="ℹ"
ICON_WARN="⚠"
ICON_TRASH="🗑"

print_banner() {
    echo -e "\n${RED}${ICON_TRASH}${NC}  ${BOLD}Anna Uninstall${NC}\n"
}

log_info() {
    echo -e "${BLUE}${ICON_INFO}${NC}  $1"
}

log_success() {
    echo -e "${GREEN}${ICON_CHECK}${NC}  $1"
}

log_warn() {
    echo -e "${YELLOW}${ICON_WARN}${NC}  $1"
}

log_error() {
    echo -e "${RED}${ICON_CROSS}${NC}  $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        echo "   Run: sudo $0"
        exit 1
    fi
}

# Stop and disable service
stop_service() {
    log_info "Stopping Anna service..."

    if systemctl is-active --quiet annad 2>/dev/null; then
        systemctl stop annad
        log_success "Stopped annad service"
    fi

    if systemctl is-enabled --quiet annad 2>/dev/null; then
        systemctl disable annad
        log_success "Disabled annad service"
    fi
}

# Remove binaries
remove_binaries() {
    log_info "Removing binaries..."

    rm -f /usr/local/bin/annad
    rm -f /usr/local/bin/annactl

    log_success "Removed binaries"
}

# Remove systemd service
remove_service() {
    log_info "Removing systemd service..."

    rm -f /etc/systemd/system/annad.service
    systemctl daemon-reload

    log_success "Removed systemd service"
}

# Remove probes (DEPRECATED - installer no longer creates this)
# Kept for backwards compatibility with older installations
remove_probes() {
    if [[ -d /usr/share/anna ]]; then
        log_info "Removing legacy probes directory..."
        rm -rf /usr/share/anna
        log_success "Removed legacy probes"
    fi
    # No-op if directory doesn't exist
}

# Remove config (optional)
remove_config() {
    read -p "Remove configuration? [y/N] " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf /etc/anna
        log_success "Removed configuration"
    else
        log_info "Configuration preserved at /etc/anna"
    fi
}

# Remove user data (optional)
remove_user_data() {
    read -p "Remove user data and logs? [y/N] " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf /var/lib/anna
        rm -rf /var/log/anna
        rm -rf /run/anna
        log_success "Removed user data and logs"
    else
        log_info "User data preserved at /var/lib/anna"
    fi
}

# Remove anna user (DEPRECATED - installer no longer creates this since v6.0.0)
# Kept for backwards compatibility with older installations
remove_user() {
    if id "anna" &>/dev/null; then
        read -p "Found legacy anna user. Remove it? [y/N] " -n 1 -r
        echo

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            userdel anna 2>/dev/null || true
            log_success "Removed legacy anna user"
        else
            log_info "Anna user preserved"
        fi
    fi
    # No-op if user doesn't exist
}

# Confirm uninstall
confirm_uninstall() {
    echo -e "${YELLOW}${BOLD}Warning:${NC} This will remove Anna from your system."
    echo ""
    read -p "Are you sure you want to uninstall? [y/N] " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstall cancelled"
        exit 0
    fi
}

# Main
main() {
    print_banner
    check_root
    confirm_uninstall

    echo ""
    stop_service
    remove_binaries
    remove_service
    remove_probes
    remove_config
    remove_user_data
    remove_user

    echo ""
    log_success "${BOLD}Anna has been uninstalled${NC}"
    echo ""
    echo "   Thank you for using Anna!"
    echo ""
}

main "$@"
