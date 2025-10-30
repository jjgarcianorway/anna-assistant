#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Uninstaller
# Safe removal with automatic backup

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/anna"
BACKUP_DIR="$HOME/Documents/anna_backup_$(date +%Y%m%d_%H%M%S)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

backup_config() {
    log_info "Creating backup at $BACKUP_DIR..."

    mkdir -p "$BACKUP_DIR"

    if [[ -d "$CONFIG_DIR" ]]; then
        sudo cp -r "$CONFIG_DIR" "$BACKUP_DIR/"
        sudo chown -R "$USER:$USER" "$BACKUP_DIR"
        log_success "Configuration backed up"
    else
        log_info "No configuration to backup"
    fi
}

stop_service() {
    log_info "Stopping annad service..."

    if systemctl is-active --quiet annad.service; then
        sudo systemctl stop annad.service
        log_success "Service stopped"
    else
        log_info "Service not running"
    fi

    if systemctl is-enabled --quiet annad.service 2>/dev/null; then
        sudo systemctl disable annad.service
        log_success "Service disabled"
    fi
}

remove_files() {
    log_info "Removing Anna files..."

    # Remove binaries
    sudo rm -f "$BIN_DIR/annad" "$BIN_DIR/annactl"

    # Remove systemd service
    sudo rm -f "$SYSTEMD_DIR/annad.service"
    sudo systemctl daemon-reload

    # Remove config (already backed up)
    sudo rm -rf "$CONFIG_DIR"

    # Remove socket if it exists
    sudo rm -f /run/anna.sock

    log_success "Files removed"
}

print_completion() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}║   UNINSTALLATION COMPLETE             ║${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Backup location: $BACKUP_DIR"
    echo ""
}

confirm_uninstall() {
    echo -e "${YELLOW}This will remove Anna Assistant from your system.${NC}"
    echo "Configuration will be backed up to: $BACKUP_DIR"
    echo ""
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstall cancelled"
        exit 0
    fi
}

main() {
    echo -e "${BLUE}Anna Assistant Uninstaller${NC}"
    echo ""

    confirm_uninstall
    backup_config
    stop_service
    remove_files
    print_completion
}

main "$@"
