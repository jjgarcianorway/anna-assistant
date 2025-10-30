#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Installer
# Minimal, elegant, contract-driven installation

VERSION="0.9.0"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/anna"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    cat <<'EOF'
    ╔═══════════════════════════════════════╗
    ║                                       ║
    ║        ANNA ASSISTANT v0.9.0          ║
    ║     Next-Gen Linux System Helper      ║
    ║                                       ║
    ╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

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

    if ! cargo build --release --quiet; then
        log_error "Compilation failed"
        exit 1
    fi

    log_success "Compilation complete"
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

setup_config() {
    log_info "Setting up configuration..."

    if [[ ! -d "$CONFIG_DIR" ]]; then
        sudo mkdir -p "$CONFIG_DIR"
    fi

    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        sudo cp config/default.toml "$CONFIG_DIR/config.toml"
        log_success "Default configuration created"
    else
        log_info "Configuration already exists, skipping"
    fi
}

enable_service() {
    log_info "Enabling and starting annad service..."

    sudo systemctl enable annad.service
    sudo systemctl start annad.service

    # Wait a moment for socket to appear
    sleep 1

    log_success "Service started"
}

run_doctor() {
    log_info "Running system diagnostics..."

    if annactl doctor; then
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
    echo "  annactl status    - Check daemon status"
    echo "  annactl doctor    - Run diagnostics"
    echo "  annactl --help    - Show all commands"
    echo ""
    echo "Service management:"
    echo "  sudo systemctl status annad"
    echo "  sudo systemctl restart annad"
    echo ""
}

main() {
    print_banner

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        log_error "Must run from anna-assistant project root"
        exit 1
    fi

    check_requirements
    compile_binaries
    install_binaries
    install_systemd_service
    setup_config
    enable_service
    run_doctor
    print_completion
}

main "$@"
