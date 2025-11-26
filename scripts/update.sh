#!/usr/bin/env bash
# Anna v0.0.1 - Update Script
# Checks for updates, downloads, and installs atomically

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
ICON_UPDATE="⬆"

GITHUB_REPO="jjgarcianorway/anna-assistant"
CURRENT_VERSION="0.0.1"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR="/tmp/anna-update"

print_banner() {
    echo -e "\n${BLUE}${ICON_UPDATE}${NC}  ${BOLD}Anna Update${NC}"
    echo -e "   Current version: ${CYAN}${CURRENT_VERSION}${NC}\n"
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

# Get latest version from GitHub
get_latest_version() {
    log_info "Checking for updates..."

    LATEST_VERSION=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" \
        | grep '"tag_name":' \
        | sed -E 's/.*"v([^"]+)".*/\1/')

    if [[ -z "$LATEST_VERSION" ]]; then
        log_warn "Could not fetch latest version"
        LATEST_VERSION="$CURRENT_VERSION"
    fi

    echo "   Latest version: ${CYAN}${LATEST_VERSION}${NC}"
}

# Compare versions
compare_versions() {
    if [[ "$CURRENT_VERSION" == "$LATEST_VERSION" ]]; then
        log_success "Already up to date!"
        exit 0
    fi

    log_info "Update available: ${CURRENT_VERSION} → ${LATEST_VERSION}"
}

# Download new binaries
download_update() {
    log_info "Downloading update..."

    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"

    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64) ARCH_NAME="x86_64-unknown-linux-gnu" ;;
        aarch64) ARCH_NAME="aarch64-unknown-linux-gnu" ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${LATEST_VERSION}/anna-${LATEST_VERSION}-${ARCH_NAME}.tar.gz"

    if curl -sL "$DOWNLOAD_URL" -o anna.tar.gz; then
        log_success "Downloaded update"
    else
        log_error "Failed to download update"
        exit 1
    fi
}

# Verify signature (placeholder for future implementation)
verify_signature() {
    log_info "Verifying signature..."
    # TODO: Implement signature verification
    log_warn "Signature verification not yet implemented"
}

# Install update atomically
install_update() {
    log_info "Installing update..."

    cd "$TEMP_DIR"
    tar -xzf anna.tar.gz

    # Stop service
    if systemctl is-active --quiet annad; then
        systemctl stop annad
        RESTART_SERVICE=true
    else
        RESTART_SERVICE=false
    fi

    # Atomic replacement
    if [[ -f "annad" ]]; then
        mv annad "${INSTALL_DIR}/annad.new"
        mv "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annad.old" 2>/dev/null || true
        mv "${INSTALL_DIR}/annad.new" "${INSTALL_DIR}/annad"
        rm -f "${INSTALL_DIR}/annad.old"
    fi

    if [[ -f "annactl" ]]; then
        mv annactl "${INSTALL_DIR}/annactl.new"
        mv "${INSTALL_DIR}/annactl" "${INSTALL_DIR}/annactl.old" 2>/dev/null || true
        mv "${INSTALL_DIR}/annactl.new" "${INSTALL_DIR}/annactl"
        rm -f "${INSTALL_DIR}/annactl.old"
    fi

    # Set permissions
    chmod 755 "${INSTALL_DIR}/annad"
    chmod 755 "${INSTALL_DIR}/annactl"

    # Restart service if it was running
    if [[ "$RESTART_SERVICE" == "true" ]]; then
        systemctl start annad
        log_success "Restarted annad service"
    fi

    log_success "Update installed"
}

# Cleanup
cleanup() {
    rm -rf "$TEMP_DIR"
}

# Main
main() {
    print_banner
    check_root
    get_latest_version
    compare_versions
    download_update
    verify_signature
    install_update
    cleanup

    echo ""
    log_success "${BOLD}Update complete!${NC}"
    echo "   New version: ${CYAN}${LATEST_VERSION}${NC}"
    echo ""
}

main "$@"
