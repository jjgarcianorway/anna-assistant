#!/usr/bin/env bash
# Anna v0.10.1 Uninstaller
# Removes Anna Assistant with optional --purge of all data

set -euo pipefail

PURGE=0

# Parse arguments
for arg in "$@"; do
    case $arg in
        --purge)
            PURGE=1
            shift
            ;;
        *)
            ;;
    esac
done

# Colors
if [[ -t 1 ]] && command -v tput &>/dev/null; then
    BOLD=$(tput bold)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    RED=$(tput setaf 1)
    RESET=$(tput sgr0)
else
    BOLD="" GREEN="" YELLOW="" RED="" RESET=""
fi

log_step() {
    echo "  $1"
}

log_success() {
    echo "${GREEN}✓${RESET} $1"
}

log_warn() {
    echo "${YELLOW}⚠${RESET} $1"
}

log_error() {
    echo "${RED}✗${RESET} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run with sudo"
    exit 20
fi

echo ""
echo "${BOLD}Anna v0.10.1 Uninstaller${RESET}"
echo ""

if [[ $PURGE -eq 1 ]]; then
    log_warn "PURGE mode enabled - will remove all data!"
    echo ""
    read -p "Are you sure? Type 'yes' to continue: " confirm
    if [[ "$confirm" != "yes" ]]; then
        echo "Uninstall cancelled"
        exit 0
    fi
    echo ""
fi

# Stop and disable service
log_step "Stopping annad.service..."
if systemctl is-active --quiet annad.service; then
    systemctl stop annad.service
    log_success "Service stopped"
else
    log_step "Service not running"
fi

log_step "Disabling annad.service..."
if systemctl is-enabled --quiet annad.service; then
    systemctl disable annad.service
    log_success "Service disabled"
else
    log_step "Service not enabled"
fi

# Remove systemd unit
log_step "Removing systemd unit..."
if [[ -f /etc/systemd/system/annad.service ]]; then
    rm -f /etc/systemd/system/annad.service
    systemctl daemon-reload
    log_success "Systemd unit removed"
else
    log_step "Systemd unit not found"
fi

# Remove binaries
log_step "Removing binaries..."
rm -f /usr/local/bin/annad
rm -f /usr/local/bin/annactl
log_success "Binaries removed"

# Remove configuration (preserve unless --purge)
if [[ $PURGE -eq 1 ]]; then
    log_step "Removing configuration..."
    rm -rf /etc/anna
    rm -rf /usr/lib/anna
    log_success "Configuration removed"
else
    log_warn "Preserving configuration in /etc/anna and /usr/lib/anna"
    log_step "(use --purge to remove)"
fi

# Remove runtime directory
log_step "Removing runtime directory..."
rm -rf /run/anna
log_success "Runtime directory removed"

# Remove data and logs (preserve unless --purge)
if [[ $PURGE -eq 1 ]]; then
    log_step "Removing data and logs..."
    rm -rf /var/lib/anna
    rm -rf /var/log/anna
    log_success "Data and logs removed"
else
    log_warn "Preserving data in /var/lib/anna and logs in /var/log/anna"
    log_step "(use --purge to remove)"
fi

# Remove user (only with --purge)
if [[ $PURGE -eq 1 ]]; then
    log_step "Removing anna user..."
    if id "anna" &>/dev/null; then
        userdel anna 2>/dev/null || log_warn "Failed to remove user (may not exist)"
        log_success "User removed"
    else
        log_step "User 'anna' does not exist"
    fi
fi

echo ""
if [[ $PURGE -eq 1 ]]; then
    log_success "${BOLD}Anna completely removed${RESET}"
else
    log_success "${BOLD}Anna uninstalled (data preserved)${RESET}"
    echo ""
    echo "  To remove all data: sudo ./scripts/uninstall_v101.sh --purge"
fi
echo ""
