#!/bin/bash
# Anna Uninstaller
# Usage: sudo bash uninstall.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "Please run as root: sudo bash uninstall.sh"
        exit 1
    fi
}

# Stop and disable service
stop_service() {
    log_info "Stopping Anna daemon..."

    if systemctl is-active --quiet annad 2>/dev/null; then
        systemctl stop annad
        log_info "Service stopped"
    fi

    if systemctl is-enabled --quiet annad 2>/dev/null; then
        systemctl disable annad
        log_info "Service disabled"
    fi
}

# Remove systemd service file
remove_service() {
    log_info "Removing systemd service..."

    if [ -f /etc/systemd/system/annad.service ]; then
        rm -f /etc/systemd/system/annad.service
        systemctl daemon-reload
        log_info "Service file removed"
    fi
}

# Remove binaries
remove_binaries() {
    log_info "Removing binaries..."

    rm -f /usr/local/bin/annad
    rm -f /usr/local/bin/annactl
    log_info "Binaries removed"
}

# Remove data directories
remove_data() {
    log_info "Removing system data directories..."

    rm -rf /var/lib/anna
    rm -rf /run/anna
    log_info "System data directories removed"
}

# Remove user data (v0.3.24: includes backups)
remove_user_data() {
    log_info "Checking for user data..."

    # Check all users with home directories
    for user_home in /home/*; do
        if [ -d "$user_home" ]; then
            username=$(basename "$user_home")

            # Check for ~/.anna directory
            anna_dir="$user_home/.anna"
            if [ -d "$anna_dir" ]; then
                # Check for backups specifically
                backup_dir="$anna_dir/backups"
                if [ -d "$backup_dir" ]; then
                    backup_count=$(find "$backup_dir" -maxdepth 1 -type d | wc -l)
                    backup_count=$((backup_count - 1))  # Subtract 1 for the directory itself
                    if [ "$backup_count" -gt 0 ]; then
                        log_warn "Found $backup_count backup(s) in $backup_dir"
                        read -p "Remove backups for user $username? [y/N] " -n 1 -r
                        echo
                        if [[ $REPLY =~ ^[Yy]$ ]]; then
                            rm -rf "$backup_dir"
                            log_info "Backups removed for $username"
                        else
                            log_info "Backups preserved at $backup_dir"
                        fi
                    fi
                fi

                # Remove rest of ~/.anna (excluding backups if preserved)
                read -p "Remove all Anna data for user $username ($anna_dir)? [y/N] " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    rm -rf "$anna_dir"
                    log_info "User data removed for $username"
                fi
            fi

            # Check for ~/.local/share/anna (tickets, fix history)
            local_anna="$user_home/.local/share/anna"
            if [ -d "$local_anna" ]; then
                read -p "Remove local data for user $username ($local_anna)? [y/N] " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    rm -rf "$local_anna"
                    log_info "Local data removed for $username"
                fi
            fi
        fi
    done
}

# Optionally remove anna group
cleanup_group() {
    read -p "Remove anna group? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if getent group anna > /dev/null 2>&1; then
            groupdel anna
            log_info "Group removed"
        fi
    fi
}

# Print completion message
print_complete() {
    echo
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}  Anna uninstalled successfully${NC}"
    echo -e "${GREEN}======================================${NC}"
    echo
    echo "Note: Ollama was not removed. To remove Ollama:"
    echo "  sudo rm /usr/local/bin/ollama"
    echo "  sudo rm -rf ~/.ollama"
    echo "  sudo systemctl stop ollama"
    echo "  sudo systemctl disable ollama"
    echo "  sudo rm /etc/systemd/system/ollama.service"
    echo
}

# Main uninstall flow
main() {
    echo
    echo "================================"
    echo "   Anna Uninstaller"
    echo "================================"
    echo

    check_root

    echo "This will remove Anna and all its data."
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Uninstall cancelled."
        exit 0
    fi

    stop_service
    remove_service
    remove_binaries
    remove_data
    remove_user_data
    cleanup_group
    print_complete
}

main "$@"
