#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Installer - Sprint 1
# Idempotent, elegant, contract-driven installation

VERSION="0.9.0"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"

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
    ║        ANNA ASSISTANT v0.9.0          ║
    ║     Next-Gen Linux System Helper      ║
    ║             Sprint 1                  ║
    ║                                       ║
    ╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

check_requirements() {
    log_info "Checking system requirements..."

    # Check OS
    if [[ ! -f /etc/arch-release ]]; then
        log_warn "Not running on Arch Linux. Proceeding anyway..."
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
        log_info "Install with: sudo pacman -S rust cargo systemd"
        exit 1
    fi

    log_success "All requirements satisfied"
}

compile_binaries() {
    log_info "Compiling Anna (this may take a few minutes)..."

    if cargo build --release --quiet 2>/dev/null; then
        log_success "Compilation complete"
    else
        log_error "Compilation failed"
        exit 1
    fi
}

install_binaries() {
    log_info "Installing binaries to $BIN_DIR..."

    sudo install -m 755 target/release/annad "$BIN_DIR/annad"
    sudo install -m 755 target/release/annactl "$BIN_DIR/annactl"

    log_success "Binaries installed"
}

install_systemd_service() {
    log_info "Installing systemd service..."

    sudo cp etc/systemd/annad.service "$SYSTEMD_DIR/annad.service"
    sudo systemctl daemon-reload

    log_success "Systemd service installed"
}

install_polkit_policy() {
    log_info "Installing polkit policy..."

    if [[ ! -d "$POLKIT_DIR" ]]; then
        log_warn "Polkit directory not found, creating..."
        sudo mkdir -p "$POLKIT_DIR"
    fi

    sudo cp polkit/com.anna.policy "$POLKIT_DIR/com.anna.policy"
    log_success "Polkit policy installed"
}

install_bash_completion() {
    log_info "Installing bash completion..."

    if [[ ! -d "$COMPLETION_DIR" ]]; then
        log_warn "Bash completion directory not found, skipping..."
        return
    fi

    sudo cp completion/annactl.bash "$COMPLETION_DIR/annactl"
    log_success "Bash completion installed"
}

setup_config() {
    log_info "Setting up configuration..."

    if [[ ! -d "$CONFIG_DIR" ]]; then
        sudo mkdir -p "$CONFIG_DIR"
    fi

    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        sudo cp config/default.toml "$CONFIG_DIR/config.toml"
        log_success "Default configuration created"
    else
        log_info "Configuration already exists, preserving"
    fi
}

create_required_paths() {
    log_info "Creating required paths..."

    # System paths (created by daemon)
    sudo mkdir -p /run/anna
    sudo mkdir -p /var/lib/anna/events

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
    if sudo systemctl enable annad.service 2>/dev/null; then
        log_success "Service enabled"
    else
        log_warn "Service enable failed or already enabled"
    fi

    # Start or restart the service
    if sudo systemctl is-active --quiet annad.service; then
        log_info "Service already running, restarting..."
        sudo systemctl restart annad.service
    else
        sudo systemctl start annad.service
    fi

    # Wait for socket to appear
    for i in {1..10}; do
        if [[ -S /run/anna/annad.sock ]]; then
            log_success "Service started successfully"
            return
        fi
        sleep 0.5
    done

    log_warn "Service started but socket not yet available"
}

run_doctor() {
    log_info "Running system diagnostics..."

    if annactl doctor 2>/dev/null; then
        log_success "All diagnostics passed"
    else
        log_warn "Some diagnostics failed - review output above"
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
    echo "  annactl config get <key>    - Get config value"
    echo "  annactl config set <scope> <key> <value>"
    echo ""
    echo "Service management:"
    echo "  sudo systemctl status annad"
    echo "  sudo systemctl restart annad"
    echo "  sudo journalctl -u annad -f"
    echo ""
}

main() {
    print_banner

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        log_error "Must run from anna-assistant project root"
        exit 1
    fi

    # All steps are idempotent
    check_requirements
    compile_binaries
    install_binaries
    install_systemd_service
    install_polkit_policy
    install_bash_completion
    setup_config
    create_required_paths
    enable_service
    run_doctor
    print_completion
}

main "$@"
