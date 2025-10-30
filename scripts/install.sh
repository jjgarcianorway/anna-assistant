#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Installer - Sprint 3 (v0.9.2a)
# Idempotent, production-ready installation with systemd integration

VERSION="0.9.2"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
TMPFILES_DIR="/usr/lib/tmpfiles.d"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"
STATE_DIR="/var/lib/anna"
ANNA_GROUP="anna"

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
    ║     Next-Gen Linux System Helper      ║
    ║   Sprint 3: Intelligence & Policies   ║
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

check_requirements() {
    log_info "Checking system requirements..."

    # Check OS
    if [[ ! -f /etc/arch-release ]] && [[ ! -f /etc/os-release ]]; then
        log_warn "Could not detect Linux distribution. Proceeding anyway..."
    fi

    # Check for required tools
    local missing=()
    for tool in cargo rustc systemctl; do
        if ! command -v "$tool" &>/dev/null; then
            missing+=("$tool")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Install with: sudo pacman -S rust cargo systemd (Arch)"
        log_info "           or: sudo apt install cargo rustc systemd (Debian/Ubuntu)"
        exit 1
    fi

    log_success "All requirements satisfied"
}

compile_binaries() {
    log_info "Compiling Anna (this may take a few minutes)..."

    # Build in release mode
    if ! cargo build --release; then
        log_error "Compilation failed"
        exit 1
    fi

    if [[ ! -f target/release/annad ]] || [[ ! -f target/release/annactl ]]; then
        log_error "Binaries not found after compilation"
        exit 1
    fi

    log_success "Compilation complete"
}

create_anna_group() {
    log_info "Setting up anna group..."

    if getent group "$ANNA_GROUP" > /dev/null 2>&1; then
        log_info "Group '$ANNA_GROUP' already exists"
    else
        groupadd "$ANNA_GROUP"
        log_success "Group '$ANNA_GROUP' created"
    fi
}

add_user_to_group() {
    log_info "Adding user to anna group..."

    # Determine the actual user (not root)
    local target_user="${SUDO_USER:-}"

    if [[ -z "$target_user" ]]; then
        log_warn "Could not determine user to add to group (not run via sudo)"
        log_warn "Manual step: Run 'sudo usermod -aG anna YOUR_USERNAME'"
        return
    fi

    if id -nG "$target_user" | grep -qw "$ANNA_GROUP"; then
        log_info "User '$target_user' already in group '$ANNA_GROUP'"
    else
        usermod -aG "$ANNA_GROUP" "$target_user"
        log_success "User '$target_user' added to group '$ANNA_GROUP'"
        log_warn "NOTE: Group membership requires logout/login or 'newgrp anna' to take effect"
    fi
}

install_binaries() {
    log_info "Installing binaries to $BIN_DIR..."

    install -m 755 target/release/annad "$BIN_DIR/annad"
    install -m 755 target/release/annactl "$BIN_DIR/annactl"

    log_success "Binaries installed"
}

install_systemd_service() {
    log_info "Installing systemd service..."

    # Install service unit
    if [[ -f packaging/arch/annad.service ]]; then
        cp packaging/arch/annad.service "$SYSTEMD_DIR/annad.service"
    elif [[ -f etc/systemd/annad.service ]]; then
        cp etc/systemd/annad.service "$SYSTEMD_DIR/annad.service"
    else
        log_error "Service file not found"
        exit 1
    fi

    # Install tmpfiles configuration
    if [[ -f packaging/arch/annad.tmpfiles.conf ]]; then
        mkdir -p "$TMPFILES_DIR"
        cp packaging/arch/annad.tmpfiles.conf "$TMPFILES_DIR/annad.conf"
        systemd-tmpfiles --create "$TMPFILES_DIR/annad.conf" 2>/dev/null || true
        log_success "Tmpfiles configuration installed"
    fi

    systemctl daemon-reload
    log_success "Systemd service installed"
}

install_polkit_policy() {
    log_info "Installing polkit policy..."

    if [[ ! -d "$POLKIT_DIR" ]]; then
        mkdir -p "$POLKIT_DIR"
    fi

    if [[ -f polkit/com.anna.policy ]]; then
        cp polkit/com.anna.policy "$POLKIT_DIR/com.anna.policy"
        log_success "Polkit policy installed"
    else
        log_warn "Polkit policy file not found, skipping"
    fi
}

install_bash_completion() {
    log_info "Installing bash completion..."

    if [[ ! -d "$COMPLETION_DIR" ]]; then
        log_warn "Bash completion directory not found, skipping..."
        return
    fi

    if [[ -f completion/annactl.bash ]]; then
        cp completion/annactl.bash "$COMPLETION_DIR/annactl"
        log_success "Bash completion installed"
    else
        log_warn "Bash completion file not found, skipping"
    fi
}

setup_directories() {
    log_info "Setting up directories with correct permissions..."

    # Config directory
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR/policies.d"
    chown -R root:"$ANNA_GROUP" "$CONFIG_DIR"
    chmod 0750 "$CONFIG_DIR"
    chmod 0750 "$CONFIG_DIR/policies.d"

    # State directory
    mkdir -p "$STATE_DIR/state"
    mkdir -p "$STATE_DIR/events"
    chown -R root:"$ANNA_GROUP" "$STATE_DIR"
    chmod -R 0750 "$STATE_DIR"

    # Runtime directory (handled by systemd RuntimeDirectory)
    # But create it manually in case systemd hasn't started yet
    mkdir -p /run/anna
    chown root:"$ANNA_GROUP" /run/anna
    chmod 0770 /run/anna

    log_success "Directories created with correct permissions"
}

setup_config() {
    log_info "Setting up configuration..."

    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        if [[ -f config/default.toml ]]; then
            cp config/default.toml "$CONFIG_DIR/config.toml"
            chown root:"$ANNA_GROUP" "$CONFIG_DIR/config.toml"
            chmod 0640 "$CONFIG_DIR/config.toml"
            log_success "Default configuration created"
        else
            log_warn "Default config not found, daemon will create it"
        fi
    else
        log_info "Configuration already exists, preserving"
    fi

    # Install example policies
    if [[ -d docs/policies.d ]] && ls docs/policies.d/*.yaml >/dev/null 2>&1; then
        cp docs/policies.d/*.yaml "$CONFIG_DIR/policies.d/" 2>/dev/null || true
        chown -R root:"$ANNA_GROUP" "$CONFIG_DIR/policies.d"
        chmod 0640 "$CONFIG_DIR/policies.d"/*.yaml 2>/dev/null || true
        log_success "Example policies installed"
    fi
}

create_user_paths() {
    log_info "Creating user paths..."

    # User paths (if we know the user)
    if [[ -n "${SUDO_USER:-}" ]]; then
        local user_home=$(eval echo "~$SUDO_USER")
        sudo -u "$SUDO_USER" mkdir -p "$user_home/.config/anna"
        sudo -u "$SUDO_USER" mkdir -p "$user_home/.local/share/anna/events"
        log_success "User paths created for $SUDO_USER"
    else
        log_info "User paths will be created on first use"
    fi
}

enable_service() {
    log_info "Enabling and starting annad service..."

    # Enable the service
    systemctl enable annad.service 2>/dev/null || log_warn "Service enable failed or already enabled"

    # Start or restart the service
    if systemctl is-active --quiet annad.service; then
        log_info "Service already running, restarting..."
        systemctl restart annad.service
    else
        systemctl start annad.service
    fi

    # Wait for socket to appear
    log_info "Waiting for socket creation..."
    for i in {1..10}; do
        if [[ -S /run/anna/annad.sock ]]; then
            log_success "Service started successfully"
            return 0
        fi
        sleep 0.5
    done

    log_error "Service started but socket not available"
    log_info "Check status with: sudo systemctl status annad"
    log_info "View logs with: sudo journalctl -u annad --since -5m"
    return 1
}

post_install_validation() {
    log_info "Running post-install validation..."

    # Wait a moment for socket to be ready
    sleep 1

    local validation_failed=false
    local target_user="${SUDO_USER:-root}"

    # Check if socket exists
    if [[ ! -S /run/anna/annad.sock ]]; then
        log_error "Socket not found at /run/anna/annad.sock"
        validation_failed=true
    else
        log_success "Socket exists"
    fi

    # Check socket permissions
    if [[ -S /run/anna/annad.sock ]]; then
        local sock_perms=$(stat -c "%a" /run/anna/annad.sock 2>/dev/null)
        if [[ "$sock_perms" != "660" ]] && [[ "$sock_perms" != "666" ]]; then
            log_warn "Socket permissions are $sock_perms (expected 660 or 666)"
        else
            log_success "Socket permissions correct ($sock_perms)"
        fi
    fi

    # Test commands
    log_info "Testing annactl commands..."

    # Determine how to run annactl (with correct group context)
    local annactl_cmd
    if [[ "$target_user" != "root" ]] && ! id -nG "$target_user" | grep -qw "$ANNA_GROUP"; then
        log_warn "User '$target_user' not yet in anna group (requires logout/login)"
        log_info "Using 'sg anna' for validation..."
        annactl_cmd="sg anna -c"
    else
        annactl_cmd=""
    fi

    # Test ping
    if [[ -n "$annactl_cmd" ]]; then
        if sudo -u "$target_user" $annactl_cmd "annactl ping" &>/dev/null; then
            log_success "annactl ping: OK"
        else
            log_error "annactl ping: FAILED"
            validation_failed=true
        fi
    else
        if annactl ping &>/dev/null; then
            log_success "annactl ping: OK"
        else
            log_error "annactl ping: FAILED"
            validation_failed=true
        fi
    fi

    # Test status
    if [[ -n "$annactl_cmd" ]]; then
        if sudo -u "$target_user" $annactl_cmd "annactl status" &>/dev/null; then
            log_success "annactl status: OK"
        else
            log_error "annactl status: FAILED"
            validation_failed=true
        fi
    else
        if annactl status &>/dev/null; then
            log_success "annactl status: OK"
        else
            log_error "annactl status: FAILED"
            validation_failed=true
        fi
    fi

    if $validation_failed; then
        log_error "Post-install validation failed"
        log_info "Troubleshooting:"
        log_info "  1. Check service: sudo systemctl status annad"
        log_info "  2. View logs: sudo journalctl -u annad --since -5m"
        log_info "  3. Check socket: ls -lh /run/anna/annad.sock"
        return 1
    else
        log_success "All validation checks passed"
        return 0
    fi
}

print_completion() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}║   INSTALLATION COMPLETE!              ║${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Quick start:"
    echo "  annactl status              - Check daemon status"
    echo "  annactl doctor              - Run diagnostics"
    echo "  annactl config list         - List configuration"
    echo "  annactl policy list         - List policies"
    echo "  annactl events show         - Show recent events"
    echo "  annactl learning stats      - Learning statistics"
    echo ""
    echo "Service management:"
    echo "  sudo systemctl status annad"
    echo "  sudo systemctl restart annad"
    echo "  sudo journalctl -u annad -f"
    echo ""

    if [[ -n "${SUDO_USER:-}" ]]; then
        echo -e "${YELLOW}IMPORTANT:${NC} Group membership requires logout/login to take effect"
        echo "Temporary workaround: Run 'newgrp anna' in your shell"
        echo ""
    fi
}

main() {
    print_banner

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        log_error "Must run from anna-assistant project root"
        exit 1
    fi

    check_root
    check_requirements
    compile_binaries
    create_anna_group
    add_user_to_group
    install_binaries
    install_systemd_service
    install_polkit_policy
    install_bash_completion
    setup_directories
    setup_config
    create_user_paths

    if ! enable_service; then
        log_error "Service startup failed"
        exit 1
    fi

    if ! post_install_validation; then
        log_warn "Validation had issues, but installation completed"
        log_info "Review the errors above and check service logs"
    fi

    print_completion
}

main "$@"
