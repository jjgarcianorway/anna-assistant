#!/bin/bash
# Anna Installer
# Usage: curl -sSL <url>/install.sh | sudo bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="jjgarcianorway/anna-assistant"
VERSION="0.0.1"
INSTALL_DIR="/usr/local/bin"
STATE_DIR="/var/lib/anna"
RUN_DIR="/run/anna"
SYSTEMD_DIR="/etc/systemd/system"

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "Please run as root: sudo bash install.sh"
        exit 1
    fi
}

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."

    # Check for systemd
    if ! command -v systemctl &> /dev/null; then
        log_error "systemd is required but not found"
        exit 1
    fi

    # Check for curl
    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not found"
        exit 1
    fi

    # Check RAM (warn if < 8GB)
    local ram_kb=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    local ram_gb=$((ram_kb / 1024 / 1024))
    if [ "$ram_gb" -lt 8 ]; then
        log_warn "System has ${ram_gb}GB RAM. 8GB+ recommended for best performance."
    fi

    log_info "System requirements OK"
}

# Create anna group if it doesn't exist
setup_group() {
    log_info "Setting up anna group..."
    if ! getent group anna > /dev/null 2>&1; then
        groupadd anna
        log_info "Created 'anna' group"
    fi

    # Add current sudo user to anna group
    if [ -n "$SUDO_USER" ]; then
        usermod -aG anna "$SUDO_USER"
        log_info "Added $SUDO_USER to anna group"
    fi
}

# Create required directories
setup_directories() {
    log_info "Creating directories..."

    mkdir -p "$STATE_DIR"
    chmod 750 "$STATE_DIR"
    chown root:anna "$STATE_DIR"

    mkdir -p "$RUN_DIR"
    chmod 750 "$RUN_DIR"
    chown root:anna "$RUN_DIR"

    log_info "Directories created"
}

# Download and install binaries
install_binaries() {
    log_info "Installing binaries..."

    local arch=$(uname -m)
    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64) arch="aarch64" ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    local base_url="https://github.com/${REPO}/releases/download/v${VERSION}"

    # Download annad
    log_info "Downloading annad..."
    curl -sSL "${base_url}/annad-linux-${arch}" -o "${INSTALL_DIR}/annad"
    chmod 755 "${INSTALL_DIR}/annad"

    # Download annactl
    log_info "Downloading annactl..."
    curl -sSL "${base_url}/annactl-linux-${arch}" -o "${INSTALL_DIR}/annactl"
    chmod 755 "${INSTALL_DIR}/annactl"

    # Verify checksums
    log_info "Verifying checksums..."
    local checksums=$(curl -sSL "${base_url}/SHA256SUMS")

    local annad_expected=$(echo "$checksums" | grep "annad-linux-${arch}" | awk '{print $1}')
    local annactl_expected=$(echo "$checksums" | grep "annactl-linux-${arch}" | awk '{print $1}')

    local annad_actual=$(sha256sum "${INSTALL_DIR}/annad" | awk '{print $1}')
    local annactl_actual=$(sha256sum "${INSTALL_DIR}/annactl" | awk '{print $1}')

    if [ "$annad_expected" != "$annad_actual" ]; then
        log_error "annad checksum mismatch!"
        rm -f "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annactl"
        exit 1
    fi

    if [ "$annactl_expected" != "$annactl_actual" ]; then
        log_error "annactl checksum mismatch!"
        rm -f "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annactl"
        exit 1
    fi

    log_info "Binaries installed and verified"
}

# Install systemd service
install_service() {
    log_info "Installing systemd service..."

    cat > "${SYSTEMD_DIR}/annad.service" << 'EOF'
[Unit]
Description=Anna AI Assistant Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=false
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/anna /run/anna

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    log_info "Systemd service installed"
}

# Enable and start service
start_service() {
    log_info "Starting Anna daemon..."

    systemctl enable annad
    systemctl start annad

    # Wait for daemon to be ready
    log_info "Waiting for daemon to initialize..."
    local attempts=0
    local max_attempts=60

    while [ $attempts -lt $max_attempts ]; do
        if [ -S "${RUN_DIR}/anna.sock" ]; then
            # Socket exists, check if daemon is ready
            sleep 2
            log_info "Anna daemon is running"
            return 0
        fi
        sleep 1
        attempts=$((attempts + 1))
    done

    log_warn "Daemon started but may still be initializing (pulling models)"
    log_warn "Check status with: annactl status"
}

# Print success message
print_success() {
    echo
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}  Anna installed successfully!${NC}"
    echo -e "${GREEN}======================================${NC}"
    echo
    echo "Usage:"
    echo "  annactl \"your question here\"  - Ask Anna"
    echo "  annactl                        - Interactive mode"
    echo "  annactl status                 - Check status"
    echo
    if [ -n "$SUDO_USER" ]; then
        echo "NOTE: Log out and back in for group membership to take effect,"
        echo "or run: newgrp anna"
    fi
    echo
}

# Main installation flow
main() {
    echo
    echo "================================"
    echo "   Anna Installer v${VERSION}"
    echo "================================"
    echo

    check_root
    check_requirements
    setup_group
    setup_directories
    install_binaries
    install_service
    start_service
    print_success
}

main "$@"
