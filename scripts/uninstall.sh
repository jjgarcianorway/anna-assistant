#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Uninstaller - Sprint 1
# Safe removal with automatic backup

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"
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

    # Backup system config
    if [[ -d "$CONFIG_DIR" ]]; then
        sudo cp -r "$CONFIG_DIR" "$BACKUP_DIR/etc_anna"
        sudo chown -R "$USER:$USER" "$BACKUP_DIR/etc_anna"
        log_success "System configuration backed up"
    fi

    # Backup user config if it exists
    if [[ -d "$HOME/.config/anna" ]]; then
        cp -r "$HOME/.config/anna" "$BACKUP_DIR/user_config"
        log_success "User configuration backed up"
    fi

    # Backup telemetry events if they exist
    if [[ -d "$HOME/.local/share/anna" ]]; then
        cp -r "$HOME/.local/share/anna" "$BACKUP_DIR/user_data"
        log_success "User data backed up"
    fi

    # Create restore instructions
    cat > "$BACKUP_DIR/README-RESTORE.md" <<'EOF'
# Anna Assistant Backup - Restore Instructions

This directory contains a backup of your Anna Assistant configuration and data.

## Contents

- `etc_anna/` - System-wide configuration from /etc/anna/
- `user_config/` - Your user configuration from ~/.config/anna/
- `user_data/` - Your telemetry events from ~/.local/share/anna/

## To Restore

### System Configuration
```bash
sudo cp -r etc_anna/anna /etc/
```

### User Configuration
```bash
cp -r user_config/anna ~/.config/
```

### User Data
```bash
cp -r user_data/anna ~/.local/share/
```

## After Restoring

If you reinstall Anna, the restored configuration will be used automatically.

No restart is needed for user configuration changes.
System configuration changes require:
```bash
sudo systemctl restart annad
```

---
Backup created: $(date)
EOF

    log_success "Restore instructions created: $BACKUP_DIR/README-RESTORE.md"
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

    # Remove polkit policy
    sudo rm -f "$POLKIT_DIR/com.anna.policy"

    # Remove bash completion
    sudo rm -f "$COMPLETION_DIR/annactl"

    # Remove config (already backed up)
    sudo rm -rf "$CONFIG_DIR"

    # Remove runtime files
    sudo rm -rf /run/anna
    sudo rm -rf /var/lib/anna

    log_success "Files removed"
}

remove_user_files() {
    log_info "Removing user configuration..."

    # User config (already backed up)
    rm -rf "$HOME/.config/anna"

    # User data (already backed up)
    rm -rf "$HOME/.local/share/anna"

    log_success "User files removed"
}

print_completion() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}║   UNINSTALLATION COMPLETE             ║${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Backup location:"
    echo "  $BACKUP_DIR"
    echo ""
    echo "To restore your configuration, see:"
    echo "  $BACKUP_DIR/README-RESTORE.md"
    echo ""
}

confirm_uninstall() {
    echo -e "${YELLOW}This will remove Anna Assistant from your system.${NC}"
    echo "Configuration will be backed up to:"
    echo "  $BACKUP_DIR"
    echo ""
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstall cancelled"
        exit 0
    fi
}

main() {
    echo -e "${BLUE}Anna Assistant Uninstaller - Sprint 1${NC}"
    echo ""

    confirm_uninstall
    backup_config
    stop_service
    remove_files
    remove_user_files
    print_completion
}

main "$@"
