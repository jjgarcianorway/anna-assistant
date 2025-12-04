#!/bin/bash
# Anna Installer
# Usage: curl -sSL <url>/install.sh | bash
# Note: sudo is NOT required for curl - installer will request it when needed

set -e

VERSION="0.0.12"
REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/anna"
STATE_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
RUN_DIR="/run/anna"
SYSTEMD_DIR="/etc/systemd/system"
ANNA_GROUP="anna"

# Colors (24-bit true color)
C_HEADER=$'\033[38;2;255;210;120m'
C_OK=$'\033[38;2;120;255;120m'
C_ERR=$'\033[38;2;255;100;100m'
C_DIM=$'\033[38;2;140;140;140m'
C_CYAN=$'\033[38;2;120;200;255m'
C_BOLD=$'\033[1m'
C_RESET=$'\033[0m'

# Symbols
SYM_OK="✓"
SYM_ERR="✗"
SYM_ARROW="›"

HR="${C_DIM}──────────────────────────────────────────────────────────────────────────────${C_RESET}"

# Get current username
USERNAME=$(whoami)

print_header() {
    echo ""
    echo "${C_HEADER}anna-install v${VERSION}${C_RESET}"
    echo "$HR"
    echo "No hidden steps. Every action is shown. Checksums are mandatory."
    echo "$HR"
    echo ""
}

print_greeting() {
    echo ""
    echo "${C_CYAN}Hello ${USERNAME}${C_RESET}, thanks a lot for giving me the opportunity to live"
    echo "in your computer! I promise to take good care of it... and you! ;)"
    echo ""
}

print_section() {
    echo "${C_DIM}[${C_RESET}$1${C_DIM}]${C_RESET} $2"
}

print_ok() {
    echo "  ${C_OK}${SYM_OK}${C_RESET} $1"
}

print_item_ok() {
    printf "  %-20s ${C_OK}${SYM_OK}${C_RESET}\n" "$1"
}

print_err() {
    echo "  ${C_ERR}${SYM_ERR}${C_RESET} $1"
}

print_footer() {
    echo ""
    echo "$HR"
    echo "Run: ${C_BOLD}annactl status${C_RESET}"
    echo "$HR"
    echo ""
}

fail() {
    print_err "$1"
    exit 1
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) fail "Unsupported architecture: $arch" ;;
    esac
}

# Check preflight requirements
preflight() {
    print_section "preflight" "linux + systemd + tools"

    # Check architecture
    ARCH=$(detect_arch)
    print_ok "arch: ${ARCH}"

    # Check systemd
    if ! command -v systemctl &> /dev/null; then
        fail "systemd not found"
    fi
    print_ok "systemd: ok"

    # Check required tools
    local tools="curl sha256sum"
    local missing=""
    for tool in $tools; do
        if ! command -v "$tool" &> /dev/null; then
            missing="$missing $tool"
        fi
    done
    if [ -n "$missing" ]; then
        fail "missing tools:$missing"
    fi
    print_ok "curl sha256sum: ok"
    echo ""
}

# Fetch release artifacts
fetch_artifacts() {
    print_section "fetch" "release artifacts"

    local base_url="https://github.com/${REPO}/releases/download/v${VERSION}"
    TMPDIR=$(mktemp -d)

    # Download annactl
    if curl -sSL "${base_url}/annactl-linux-${ARCH}" -o "${TMPDIR}/annactl" 2>/dev/null; then
        print_item_ok "annactl-${ARCH}"
    else
        fail "failed to download annactl"
    fi

    # Download annad
    if curl -sSL "${base_url}/annad-linux-${ARCH}" -o "${TMPDIR}/annad" 2>/dev/null; then
        print_item_ok "annad-${ARCH}"
    else
        fail "failed to download annad"
    fi

    # Download checksums
    if curl -sSL "${base_url}/SHA256SUMS" -o "${TMPDIR}/SHA256SUMS" 2>/dev/null; then
        print_item_ok "SHA256SUMS"
    else
        fail "failed to download SHA256SUMS"
    fi
    echo ""
}

# Verify checksums
verify_checksums() {
    print_section "verify" "checksums"

    cd "$TMPDIR"

    # Extract expected checksums
    local annactl_expected=$(grep "annactl-linux-${ARCH}" SHA256SUMS | awk '{print $1}')
    local annad_expected=$(grep "annad-linux-${ARCH}" SHA256SUMS | awk '{print $1}')

    # Compute actual checksums
    local annactl_actual=$(sha256sum annactl | awk '{print $1}')
    local annad_actual=$(sha256sum annad | awk '{print $1}')

    # Verify
    if [ "$annactl_expected" = "$annactl_actual" ]; then
        printf "  annactl  ${C_OK}OK${C_RESET}\n"
    else
        fail "annactl checksum mismatch"
    fi

    if [ "$annad_expected" = "$annad_actual" ]; then
        printf "  annad    ${C_OK}OK${C_RESET}\n"
    else
        fail "annad checksum mismatch"
    fi
    echo ""
}

# Request sudo with explanation
request_sudo() {
    print_section "sudo" "needed to write to /usr/local/bin, /etc, systemd, /var/lib"
    echo ""
    echo "  Anna needs root access to:"
    echo "    ${SYM_ARROW} Install binaries to /usr/local/bin"
    echo "    ${SYM_ARROW} Create config in /etc/anna"
    echo "    ${SYM_ARROW} Create data directory in /var/lib/anna"
    echo "    ${SYM_ARROW} Install systemd service"
    echo ""

    if [ "$EUID" -eq 0 ]; then
        SUDO=""
        print_ok "already running as root"
    else
        echo "  ${SYM_ARROW} Requesting sudo access..."
        if sudo -v; then
            SUDO="sudo"
            print_ok "sudo access granted"
        else
            fail "sudo access required but denied"
        fi
    fi
    echo ""
}

# Stop existing service if running (for upgrades)
stop_existing_service() {
    if systemctl is-active --quiet annad 2>/dev/null; then
        print_section "upgrade" "stopping existing annad service"
        $SUDO systemctl stop annad
        print_ok "annad stopped"
        echo ""
        UPGRADE_MODE=true
    else
        UPGRADE_MODE=false
    fi
}

# Install binaries
install_binaries() {
    print_section "install" "binaries"

    chmod +x "${TMPDIR}/annactl" "${TMPDIR}/annad"

    $SUDO cp "${TMPDIR}/annactl" "${INSTALL_DIR}/annactl"
    print_item_ok "/usr/local/bin/annactl"

    $SUDO cp "${TMPDIR}/annad" "${INSTALL_DIR}/annad"
    print_item_ok "/usr/local/bin/annad"
    echo ""
}

# Setup anna group and add current user
setup_group() {
    print_section "security" "group setup"

    # Create anna group if it doesn't exist
    if ! getent group "$ANNA_GROUP" > /dev/null 2>&1; then
        $SUDO groupadd "$ANNA_GROUP"
        print_ok "created group: ${ANNA_GROUP}"
    else
        print_ok "group exists: ${ANNA_GROUP}"
    fi

    # Add current user to anna group if not already member
    if ! groups "$USERNAME" 2>/dev/null | grep -q "\b${ANNA_GROUP}\b"; then
        $SUDO usermod -aG "$ANNA_GROUP" "$USERNAME"
        print_ok "added ${USERNAME} to ${ANNA_GROUP} group"
    else
        print_ok "${USERNAME} already in ${ANNA_GROUP} group"
    fi
    echo ""
}

# Install directories
install_directories() {
    print_section "install" "directories"

    $SUDO mkdir -p "$CONFIG_DIR"
    print_item_ok "/etc/anna"

    $SUDO mkdir -p "$STATE_DIR"
    $SUDO chgrp "$ANNA_GROUP" "$STATE_DIR"
    $SUDO chmod 775 "$STATE_DIR"
    print_item_ok "/var/lib/anna"

    $SUDO mkdir -p "$LOG_DIR"
    $SUDO chgrp "$ANNA_GROUP" "$LOG_DIR"
    $SUDO chmod 775 "$LOG_DIR"
    print_item_ok "/var/log/anna"

    $SUDO mkdir -p "$RUN_DIR"
    $SUDO chgrp "$ANNA_GROUP" "$RUN_DIR"
    $SUDO chmod 775 "$RUN_DIR"
    print_item_ok "/run/anna"

    $SUDO mkdir -p "${STATE_DIR}/models"
    print_item_ok "/var/lib/anna/models"
    echo ""
}

# Install config
install_config() {
    print_section "install" "config (create if missing)"

    if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
        $SUDO tee "${CONFIG_DIR}/config.toml" > /dev/null << 'EOF'
# Anna configuration
[daemon]
debug_mode = true
auto_update = true
update_interval = 600

[llm]
provider = "ollama"
EOF
    fi
    print_item_ok "/etc/anna/config.toml"
    echo ""
}

# Install systemd service
install_service() {
    print_section "service" "systemd"

    $SUDO tee "${SYSTEMD_DIR}/annad.service" > /dev/null << 'EOF'
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
Environment="HOME=/root"
Environment="OLLAMA_MODELS=/var/lib/anna/models"

[Install]
WantedBy=multi-user.target
EOF
    print_item_ok "annad.service installed"

    $SUDO systemctl daemon-reload
    $SUDO systemctl enable annad --quiet
    print_item_ok "enable"

    $SUDO systemctl start annad
    print_item_ok "start"
    echo ""
}

# Handoff message
print_handoff() {
    print_section "handoff" "annad will bootstrap the required local LLM (ollama + models)"
}

# Cleanup
cleanup() {
    rm -rf "$TMPDIR" 2>/dev/null || true
}

trap cleanup EXIT

# Main
main() {
    print_header
    print_greeting
    preflight
    fetch_artifacts
    verify_checksums
    request_sudo
    stop_existing_service
    install_binaries
    setup_group
    install_directories
    install_config
    install_service
    print_handoff
    print_footer
}

main "$@"
