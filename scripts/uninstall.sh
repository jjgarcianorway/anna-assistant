#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Uninstaller - Sprint 3 (v0.9.2a)
# Safe removal with comprehensive backup

VERSION="0.9.2"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
TMPFILES_DIR="/usr/lib/tmpfiles.d"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"
STATE_DIR="/var/lib/anna"
BACKUP_ROOT="$HOME/Documents"

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

print_banner() {
    echo -e "${BLUE}"
    cat <<'EOF'
    ╔═══════════════════════════════════════╗
    ║                                       ║
    ║        ANNA ASSISTANT v0.9.2          ║
    ║           Uninstaller                 ║
    ║                                       ║
    ╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

confirm_uninstall() {
    echo ""
    echo -e "${YELLOW}WARNING:${NC} This will remove Anna Assistant from your system"
    echo "The following will be deleted:"
    echo "  - Binaries: /usr/local/bin/anna{d,ctl}"
    echo "  - Service: /etc/systemd/system/annad.service"
    echo "  - Config: /etc/anna/"
    echo "  - State: /var/lib/anna/"
    echo ""
    echo "A backup will be created in: $BACKUP_ROOT/anna_backup_<timestamp>/"
    echo ""
    read -p "Continue with uninstall? [y/N] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstall cancelled"
        exit 0
    fi
}

stop_service() {
    log_info "Stopping annad service..."

    if systemctl is-active --quiet annad.service; then
        systemctl stop annad.service || log_warn "Failed to stop service"
        log_success "Service stopped"
    else
        log_info "Service not running"
    fi

    if systemctl is-enabled --quiet annad.service 2>/dev/null; then
        systemctl disable annad.service || log_warn "Failed to disable service"
        log_success "Service disabled"
    fi
}

create_backup() {
    log_info "Creating backup..."

    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_dir="$BACKUP_ROOT/anna_backup_$timestamp"

    mkdir -p "$backup_dir"

    # Backup configuration
    if [[ -d "$CONFIG_DIR" ]]; then
        cp -r "$CONFIG_DIR" "$backup_dir/etc_anna"
        log_success "Configuration backed up"
    fi

    # Backup state
    if [[ -d "$STATE_DIR" ]]; then
        cp -r "$STATE_DIR" "$backup_dir/var_lib_anna"
        log_success "State data backed up"
    fi

    # Backup binaries
    if [[ -f "$BIN_DIR/annad" ]]; then
        cp "$BIN_DIR/annad" "$backup_dir/"
    fi
    if [[ -f "$BIN_DIR/annactl" ]]; then
        cp "$BIN_DIR/annactl" "$backup_dir/"
    fi

    # Create restore instructions
    cat > "$backup_dir/README-RESTORE.md" <<EOF
# Anna Assistant Backup - $timestamp

This directory contains a complete backup of your Anna Assistant installation.

## What's Included

- \`etc_anna/\` - Configuration files from /etc/anna
- \`var_lib_anna/\` - State data from /var/lib/anna
- \`annad\` - Daemon binary
- \`annactl\` - CLI binary

## Restore Instructions

To restore Anna Assistant:

### Option 1: Quick Restore (if Anna is still installed)

\`\`\`bash
# Restore configuration
sudo cp -r etc_anna/* /etc/anna/

# Restore state
sudo cp -r var_lib_anna/* /var/lib/anna/

# Restart service
sudo systemctl restart annad
\`\`\`

### Option 2: Full Restore (if Anna was uninstalled)

\`\`\`bash
# 1. Restore binaries
sudo cp annad /usr/local/bin/
sudo cp annactl /usr/local/bin/
sudo chmod +x /usr/local/bin/anna{d,ctl}

# 2. Restore configuration
sudo mkdir -p /etc/anna
sudo cp -r etc_anna/* /etc/anna/

# 3. Restore state
sudo mkdir -p /var/lib/anna
sudo cp -r var_lib_anna/* /var/lib/anna/

# 4. Reinstall service (requires original source)
# Navigate to anna-assistant source directory and run:
sudo ./scripts/install.sh
\`\`\`

### Option 3: Clean Reinstall

For a clean installation from source:

\`\`\`bash
# Clone repository
git clone https://github.com/anna-assistant/anna
cd anna

# Install
sudo ./scripts/install.sh

# Then restore just your configuration
sudo cp -r /path/to/backup/etc_anna/config.toml /etc/anna/
sudo cp -r /path/to/backup/etc_anna/policies.d/* /etc/anna/policies.d/
\`\`\`

## Backup Contents

- Created: $timestamp
- Version: $VERSION
- User: ${SUDO_USER:-root}
- System: $(uname -n)

## Notes

- Configuration includes policies from /etc/anna/policies.d/
- State includes:
  - Telemetry events
  - Persistent state snapshots
  - Learning cache data
- User-specific config (~/.config/anna/) is not backed up
  (it's preserved in your home directory)

---

For support: https://github.com/anna-assistant/anna/issues
EOF

    log_success "Backup created in: $backup_dir"
    echo ""
    echo -e "${GREEN}Backup location:${NC} $backup_dir"
    echo "Restore instructions: $backup_dir/README-RESTORE.md"
    echo ""
}

remove_binaries() {
    log_info "Removing binaries..."

    rm -f "$BIN_DIR/annad" || log_warn "Failed to remove annad"
    rm -f "$BIN_DIR/annactl" || log_warn "Failed to remove annactl"

    log_success "Binaries removed"
}

remove_systemd() {
    log_info "Removing systemd integration..."

    # Remove service file
    if [[ -f "$SYSTEMD_DIR/annad.service" ]]; then
        rm -f "$SYSTEMD_DIR/annad.service"
        log_success "Service file removed"
    fi

    # Remove tmpfiles configuration
    if [[ -f "$TMPFILES_DIR/annad.conf" ]]; then
        rm -f "$TMPFILES_DIR/annad.conf"
        log_success "Tmpfiles configuration removed"
    fi

    systemctl daemon-reload
    log_success "Systemd reloaded"
}

remove_polkit() {
    log_info "Removing polkit policy..."

    if [[ -f "$POLKIT_DIR/com.anna.policy" ]]; then
        rm -f "$POLKIT_DIR/com.anna.policy"
        log_success "Polkit policy removed"
    else
        log_info "Polkit policy not found"
    fi
}

remove_bash_completion() {
    log_info "Removing bash completion..."

    if [[ -f "$COMPLETION_DIR/annactl" ]]; then
        rm -f "$COMPLETION_DIR/annactl"
        log_success "Bash completion removed"
    else
        log_info "Bash completion not found"
    fi
}

remove_config() {
    log_info "Removing configuration..."

    if [[ -d "$CONFIG_DIR" ]]; then
        rm -rf "$CONFIG_DIR"
        log_success "Configuration removed"
    else
        log_info "Configuration directory not found"
    fi
}

remove_state() {
    log_info "Removing state data..."

    if [[ -d "$STATE_DIR" ]]; then
        rm -rf "$STATE_DIR"
        log_success "State data removed"
    else
        log_info "State directory not found"
    fi
}

remove_runtime() {
    log_info "Removing runtime directory..."

    if [[ -d "/run/anna" ]]; then
        rm -rf "/run/anna"
        log_success "Runtime directory removed"
    fi
}

remove_group() {
    log_info "Checking anna group..."

    if getent group anna > /dev/null 2>&1; then
        # Check if any users are in the group
        local group_users=$(getent group anna | cut -d: -f4)

        if [[ -n "$group_users" ]]; then
            log_info "Group 'anna' has members: $group_users"
            log_info "Keeping group (users still need access)"
        else
            groupdel anna 2>/dev/null || log_warn "Failed to remove group"
            log_success "Group 'anna' removed (no members)"
        fi
    else
        log_info "Group 'anna' not found"
    fi
}

print_completion() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}║   UNINSTALL COMPLETE!                 ║${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Anna Assistant has been removed from your system."
    echo ""
    echo "User-specific data preserved:"
    echo "  ~/.config/anna/           (user configuration)"
    echo "  ~/.local/share/anna/      (user telemetry)"
    echo ""
    echo "To completely remove user data:"
    echo "  rm -rf ~/.config/anna"
    echo "  rm -rf ~/.local/share/anna"
    echo ""
    echo "To reinstall:"
    echo "  git clone https://github.com/anna-assistant/anna"
    echo "  cd anna && sudo ./scripts/install.sh"
    echo ""
}

main() {
    print_banner

    check_root
    confirm_uninstall

    stop_service
    create_backup
    remove_systemd
    remove_binaries
    remove_polkit
    remove_bash_completion
    remove_config
    remove_state
    remove_runtime
    remove_group

    print_completion
}

main "$@"
