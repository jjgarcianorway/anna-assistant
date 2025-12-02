#!/bin/bash
# Anna Installer v6.0.0 - Root Daemon Model
#
# This installer is versioned INDEPENDENTLY from Anna itself.
# Installer version: 6.x.x
# Anna version: fetched from GitHub releases
#
# v6.0.0 (Root Daemon):
#   - annad runs as ROOT for full system access
#   - All data directories owned by root:root
#   - annactl is a read-only remote control (no special permissions)
#   - All state mutations via RPC to daemon
#   - No more permission errors ever
#
# Behavior:
#   - Detects installed version (if any) via annactl version
#   - Fetches latest Anna version from GitHub
#   - Compares and shows planned action
#   - Non-interactive default: update if newer, skip if same/older
#   - Interactive mode (-i): prompt for confirmation
#   - Never clobbers config/data unless --reset is passed
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/.../install.sh | bash
#   curl -sSL ... | bash -s -- -i          # Interactive mode
#   curl -sSL ... | bash -s -- --reset     # Full reinstall (wipes config)
#
# Exit codes:
#   0 = Success or no action needed
#   1 = Error
#   2 = User declined

set -uo pipefail

# ============================================================
# CONFIGURATION
# ============================================================

INSTALLER_VERSION="7.29.0"
GITHUB_REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/anna"
DATA_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
RUN_DIR="/run/anna"

# ============================================================
# COLORS AND FORMATTING
# ============================================================

BOLD=$'\033[1m'
DIM=$'\033[2m'
RED=$'\033[31m'
GREEN=$'\033[32m'
YELLOW=$'\033[33m'
BLUE=$'\033[34m'
CYAN=$'\033[36m'
NC=$'\033[0m'

# ============================================================
# VARIABLES
# ============================================================

TMP_DIR=""
INTERACTIVE=false
RESET_MODE=false
INSTALLED_VERSION=""
LATEST_VERSION=""
SUDO=""
PLANNED_ACTION=""

# ============================================================
# LOGGING
# ============================================================

log_to_file() {
    local msg="$1"
    local log_file="/var/log/anna/install.log"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    if [[ -w "$(dirname "$log_file")" ]] || sudo mkdir -p "$(dirname "$log_file")" 2>/dev/null; then
        echo "[$timestamp] $msg" | sudo tee -a "$log_file" >/dev/null 2>/dev/null || true
    fi
}

print_line() {
    printf "%s\n" "------------------------------------------------------------"
}

print_header() {
    printf "\n${BOLD}%s${NC}\n" "$1"
    print_line
}

log_info() {
    printf "  ${BLUE}*${NC}  %s\n" "$1"
}

log_ok() {
    printf "  ${GREEN}+${NC}  %s\n" "$1"
}

log_warn() {
    printf "  ${YELLOW}~${NC}  %s\n" "$1"
}

log_error() {
    printf "  ${RED}!${NC}  %s\n" "$1"
}

# ============================================================
# UTILITY FUNCTIONS
# ============================================================

cleanup() {
    if [[ -n "$TMP_DIR" ]] && [[ -d "$TMP_DIR" ]]; then
        rm -rf "$TMP_DIR"
    fi
}

check_sudo() {
    if command -v sudo &>/dev/null; then
        SUDO="sudo"
    elif [[ $EUID -eq 0 ]]; then
        SUDO=""
    else
        log_error "sudo not found and not running as root"
        exit 1
    fi
}

request_sudo() {
    if [[ -n "$SUDO" ]]; then
        printf "\n  Sudo required for installation to ${INSTALL_DIR}\n"
        $SUDO -v || {
            log_error "Failed to obtain sudo"
            exit 1
        }
    fi
}

version_compare() {
    local v1="$1"
    local v2="$2"

    v1="${v1#v}"
    v2="${v2#v}"

    if [[ "$v1" == "$v2" ]]; then
        echo 1
        return
    fi

    local sorted
    sorted=$(printf '%s\n%s' "$v1" "$v2" | sort -V | head -1)
    if [[ "$sorted" == "$v1" ]]; then
        echo 0  # v1 < v2
    else
        echo 2  # v1 > v2
    fi
}

# ============================================================
# VERSION DETECTION
# ============================================================

detect_installed_version() {
    INSTALLED_VERSION=""

    # v5.5.0: Primary method - run annactl version and parse output
    if command -v annactl &>/dev/null; then
        local output
        # Run annactl version (or -V) and look for version number
        output=$(timeout 5 annactl version </dev/null 2>&1 | head -30) || true

        # Look for "annactl:  vX.Y.Z" or "annactl  vX.Y.Z" pattern
        if [[ "$output" =~ annactl[[:space:]]*:?[[:space:]]*v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            INSTALLED_VERSION="${BASH_REMATCH[1]}"
            return
        fi

        # Fallback: Try -V flag
        output=$(timeout 5 annactl -V </dev/null 2>&1) || true
        if [[ "$output" =~ v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            INSTALLED_VERSION="${BASH_REMATCH[1]}"
            return
        fi
    fi

    # Fallback: check if binary exists but doesn't run
    if [[ -x "${INSTALL_DIR}/annactl" ]]; then
        # Try to extract version from strings
        local output
        output=$(strings "${INSTALL_DIR}/annactl" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || true
        if [[ -n "$output" ]]; then
            INSTALLED_VERSION="$output"
        fi
    fi
}

fetch_latest_version() {
    LATEST_VERSION=""

    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local response

    if command -v curl &>/dev/null; then
        response=$(curl -fsSL "$api_url" 2>/dev/null || true)
    elif command -v wget &>/dev/null; then
        response=$(wget -qO- "$api_url" 2>/dev/null || true)
    fi

    if [[ -n "$response" ]]; then
        LATEST_VERSION=$(echo "$response" | grep -oP '"tag_name":\s*"v?\K[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
    fi

    if [[ -z "$LATEST_VERSION" ]]; then
        log_error "Could not fetch latest version from GitHub"
        log_error "Check your internet connection or try again later"
        exit 1
    fi
}

detect_arch() {
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64)
            ARCH_NAME="x86_64-unknown-linux-gnu"
            ;;
        aarch64)
            ARCH_NAME="aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
}

# ============================================================
# DISPLAY FUNCTIONS
# ============================================================

print_banner() {
    printf "\n${BOLD}Anna Telemetry Daemon Installer${NC}\n"
    print_line
    printf "  Installer version: %s\n" "$INSTALLER_VERSION"
    printf "  Architecture: %s\n" "$ARCH_NAME"
    print_line
}

print_version_summary() {
    print_header "VERSION INFORMATION"

    if [[ -z "$INSTALLED_VERSION" ]]; then
        printf "  Installed version : ${DIM}(none)${NC}\n"
    else
        printf "  Installed version : v%s\n" "$INSTALLED_VERSION"
    fi

    printf "  Available version : v%s\n" "$LATEST_VERSION"
    printf "\n"
}

print_planned_action() {
    print_header "PLANNED ACTION"

    if [[ -z "$INSTALLED_VERSION" ]]; then
        printf "  ${GREEN}Fresh install${NC} - Anna will be installed\n"
        PLANNED_ACTION="install"
    else
        local cmp
        cmp=$(version_compare "$INSTALLED_VERSION" "$LATEST_VERSION")

        case "$cmp" in
            0)
                printf "  ${GREEN}Update available${NC} - v%s -> v%s\n" "$INSTALLED_VERSION" "$LATEST_VERSION"
                PLANNED_ACTION="update"
                ;;
            1)
                printf "  ${CYAN}Same version${NC} - v%s already installed\n" "$INSTALLED_VERSION"
                PLANNED_ACTION="reinstall"
                ;;
            2)
                printf "  ${YELLOW}Downgrade${NC} - v%s -> v%s (not recommended)\n" "$INSTALLED_VERSION" "$LATEST_VERSION"
                PLANNED_ACTION="downgrade"
                ;;
        esac
    fi

    printf "\n  Target paths:\n"
    printf "    Binaries: %s\n" "$INSTALL_DIR"
    printf "    Config:   %s\n" "$CONFIG_DIR"
    printf "    Data:     %s\n" "$DATA_DIR"
    printf "    Logs:     %s\n" "$LOG_DIR"
    print_line
}

# ============================================================
# USER INTERACTION
# ============================================================

confirm_action() {
    local action="$1"

    # In non-interactive mode (pipe), auto-approve safe actions
    if [[ "$INTERACTIVE" == "false" ]]; then
        case "$action" in
            install|update)
                log_info "Auto-approving $action (non-interactive mode)"
                return 0
                ;;
            reinstall)
                # v5.5.1: Auto-approve reinstall in non-interactive mode
                # This handles curl | bash for same version
                log_info "Auto-approving reinstall (non-interactive mode)"
                return 0
                ;;
            downgrade)
                log_warn "Downgrade not allowed in non-interactive mode"
                log_warn "Use -i flag for interactive mode"
                return 1
                ;;
        esac
    fi

    # Interactive mode - prompt user
    local prompt=""
    local default=""

    case "$action" in
        install)
            prompt="Proceed with installation? (Y/n)"
            default="y"
            ;;
        update)
            prompt="Proceed with update? (Y/n)"
            default="y"
            ;;
        reinstall)
            prompt="Reinstall same version? (y/N)"
            default="n"
            ;;
        downgrade)
            prompt="Downgrade to older version? (y/N)"
            default="n"
            ;;
    esac

    printf "\n  %s " "$prompt"
    read -r answer </dev/tty || answer=""
    answer="${answer:-$default}"

    case "${answer,,}" in
        y|yes)
            return 0
            ;;
        *)
            log_info "Installation cancelled by user"
            return 1
            ;;
    esac
}

# ============================================================
# INSTALLATION FUNCTIONS
# ============================================================

download_binaries() {
    log_info "Downloading binaries..."

    local base_url="https://github.com/${GITHUB_REPO}/releases/download/v${LATEST_VERSION}"
    TMP_DIR=$(mktemp -d)

    local annad_name annactl_name
    case "$ARCH" in
        x86_64|x86_64-unknown-linux-gnu)
            annad_name="annad-${LATEST_VERSION}-x86_64-unknown-linux-gnu"
            annactl_name="annactl-${LATEST_VERSION}-x86_64-unknown-linux-gnu"
            ;;
        aarch64|aarch64-unknown-linux-gnu)
            annad_name="annad-${LATEST_VERSION}-aarch64-unknown-linux-gnu"
            annactl_name="annactl-${LATEST_VERSION}-aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            return 1
            ;;
    esac

    log_info "Downloading annad..."
    if command -v curl &>/dev/null; then
        curl -fsSL "${base_url}/${annad_name}" -o "${TMP_DIR}/annad" || {
            log_error "Failed to download ${annad_name}"
            return 1
        }
        log_info "Downloading annactl..."
        curl -fsSL "${base_url}/${annactl_name}" -o "${TMP_DIR}/annactl" || {
            log_error "Failed to download ${annactl_name}"
            return 1
        }
        log_info "Downloading checksums..."
        curl -fsSL "${base_url}/SHA256SUMS" -o "${TMP_DIR}/SHA256SUMS" || {
            log_error "Failed to download checksums"
            return 1
        }
    else
        wget -q "${base_url}/${annad_name}" -O "${TMP_DIR}/annad" || return 1
        wget -q "${base_url}/${annactl_name}" -O "${TMP_DIR}/annactl" || return 1
        wget -q "${base_url}/SHA256SUMS" -O "${TMP_DIR}/SHA256SUMS" || return 1
    fi

    log_ok "Downloaded binaries"

    log_info "Verifying checksums..."
    cd "$TMP_DIR"

    local annad_sum annactl_sum
    annad_sum=$(sha256sum "annad" | awk '{print $1}')
    annactl_sum=$(sha256sum "annactl" | awk '{print $1}')

    if grep -q "$annad_sum" SHA256SUMS; then
        log_ok "annad checksum verified"
    else
        log_error "annad checksum verification failed!"
        return 1
    fi

    if grep -q "$annactl_sum" SHA256SUMS; then
        log_ok "annactl checksum verified"
    else
        log_error "annactl checksum verification failed!"
        return 1
    fi

    chmod 755 "${TMP_DIR}/annad" "${TMP_DIR}/annactl"
    log_ok "Binaries ready for installation"
}

install_binaries() {
    log_info "Installing binaries..."

    if [[ -f "${INSTALL_DIR}/annad" ]]; then
        $SUDO cp "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annad.bak" 2>/dev/null || true
    fi
    if [[ -f "${INSTALL_DIR}/annactl" ]]; then
        $SUDO cp "${INSTALL_DIR}/annactl" "${INSTALL_DIR}/annactl.bak" 2>/dev/null || true
    fi

    $SUDO mv "${TMP_DIR}/annad" "${INSTALL_DIR}/annad"
    $SUDO mv "${TMP_DIR}/annactl" "${INSTALL_DIR}/annactl"
    $SUDO chmod 755 "${INSTALL_DIR}/annad"
    $SUDO chmod 755 "${INSTALL_DIR}/annactl"

    log_ok "Installed binaries to ${INSTALL_DIR}"
}

create_directories() {
    log_info "Creating directories..."

    # v6.0.0: Root ownership model - daemon runs as root
    # No anna user needed - simpler and eliminates all permission issues

    # Create directories
    $SUDO mkdir -p "$DATA_DIR" "$LOG_DIR" "$RUN_DIR" "$CONFIG_DIR"
    $SUDO mkdir -p "${DATA_DIR}/knowledge"
    $SUDO mkdir -p "${DATA_DIR}/telemetry"

    # All directories owned by root:root with standard permissions
    # Daemon (root) has full access, annactl reads via HTTP API
    $SUDO chown -R root:root "$DATA_DIR"
    $SUDO chmod 0755 "$DATA_DIR"
    $SUDO chmod 0755 "${DATA_DIR}/knowledge"
    $SUDO chmod 0755 "${DATA_DIR}/telemetry"

    # Config directory
    $SUDO chown root:root "$CONFIG_DIR"
    $SUDO chmod 0755 "$CONFIG_DIR"

    # Log directory
    $SUDO chown -R root:root "$LOG_DIR"
    $SUDO chmod 0755 "$LOG_DIR"

    # Run directory
    $SUDO chown root:root "$RUN_DIR"
    $SUDO chmod 0755 "$RUN_DIR"

    # Fix existing file permissions
    $SUDO find "$DATA_DIR" -type f -exec chmod 0644 {} \; 2>/dev/null || true

    log_ok "Created directories (root:root ownership)"
    log_ok "  Data: ${DATA_DIR}"
    log_ok "  Logs: ${LOG_DIR}"
    log_ok "  Config: ${CONFIG_DIR}"
}

install_systemd_service() {
    log_info "Installing systemd service..."

    local service_file="/etc/systemd/system/annad.service"

    # v6.0.0: Daemon runs as root for full system access
    # This eliminates ALL permission errors - daemon can read/write anything
    $SUDO tee "$service_file" > /dev/null << 'EOF'
[Unit]
Description=Anna Telemetry Daemon
Documentation=https://github.com/jjgarcianorway/anna-assistant
After=network.target

[Service]
Type=simple
# v6.0.0: Run as root - full system access, no permission errors
ExecStart=/usr/local/bin/annad
WorkingDirectory=/var/lib/anna
Restart=always
RestartSec=5

# Minimal hardening - root needs full access for telemetry
ProtectSystem=false
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

    $SUDO systemctl daemon-reload
    log_ok "Installed systemd service (runs as root)"
}

write_config() {
    local config_file="${CONFIG_DIR}/config.toml"

    if [[ -f "$config_file" ]] && [[ "$RESET_MODE" == "false" ]]; then
        log_ok "Config exists (preserving)"
        return
    fi

    log_info "Writing configuration..."

    $SUDO tee "$config_file" > /dev/null << EOF
# Anna v${LATEST_VERSION} Configuration
# Telemetry daemon - no LLM, no Q&A

[core]
mode = "normal"

[telemetry]
sample_interval_secs = 15      # Process sampling interval
log_scan_interval_secs = 60    # Log scanning interval
max_events_per_log = 100000    # Max events per log file
retention_days = 30            # Days to keep event logs

[log]
level = "info"  # trace, debug, info, warn, error
EOF

    $SUDO chown root:root "$config_file"
    $SUDO chmod 644 "$config_file"
    log_ok "Configuration written to ${config_file}"
}

verify_installation() {
    log_info "Verifying installation..."

    local errors=0

    if [[ -x "${INSTALL_DIR}/annad" ]]; then
        log_ok "annad binary OK"
    else
        log_error "annad binary missing or not executable"
        ((errors++))
    fi

    if [[ -x "${INSTALL_DIR}/annactl" ]]; then
        local version
        version=$("${INSTALL_DIR}/annactl" version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || version=""
        if [[ "$version" == "$LATEST_VERSION" ]]; then
            log_ok "annactl v${version} OK"
        elif [[ -n "$version" ]]; then
            log_ok "annactl v${version} installed"
        else
            log_ok "annactl binary OK"
        fi
    else
        log_error "annactl binary missing or not executable"
        ((errors++)) || true
    fi

    if [[ -f "${CONFIG_DIR}/config.toml" ]]; then
        log_ok "Configuration OK"
    else
        log_warn "Configuration file missing"
    fi

    # v6.0.0: Data directory should be owned by root
    local data_owner
    data_owner=$(stat -c '%U' "$DATA_DIR" 2>/dev/null || echo "unknown")
    if [[ "$data_owner" == "root" ]]; then
        log_ok "Data directory ownership OK (root)"
    else
        log_warn "Data directory owned by $data_owner (expected: root)"
    fi

    return $errors
}

# ============================================================
# MAIN
# ============================================================

main() {
    trap cleanup EXIT

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -i|--interactive)
                INTERACTIVE=true
                shift
                ;;
            --reset)
                RESET_MODE=true
                shift
                ;;
            -h|--help)
                printf "Usage: install.sh [-i|--interactive] [--reset]\n"
                printf "  -i, --interactive  Prompt before actions\n"
                printf "  --reset            Full reinstall (wipes config)\n"
                exit 0
                ;;
            *)
                shift
                ;;
        esac
    done

    detect_arch
    print_banner
    detect_installed_version
    fetch_latest_version
    print_version_summary
    print_planned_action

    if ! confirm_action "$PLANNED_ACTION"; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=declined installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"
        exit 0
    fi

    check_sudo
    request_sudo

    log_to_file "Installer: action=${PLANNED_ACTION} starting installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"

    print_header "INSTALLING"

    if ! download_binaries; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=download_failed"
        exit 1
    fi

    create_directories
    install_binaries
    install_systemd_service
    write_config

    print_header "VERIFICATION"
    if verify_installation; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=success version=${LATEST_VERSION}"
    else
        log_to_file "Installer: action=${PLANNED_ACTION} result=partial_success version=${LATEST_VERSION}"
    fi

    print_header "STARTING DAEMON"

    # v7.29.0: Strict daemon validation
    # The installer MUST guarantee annad is running after install/upgrade

    log_info "Reloading systemd configuration..."
    $SUDO systemctl daemon-reload

    log_info "Enabling and starting annad service..."
    if ! $SUDO systemctl enable --now annad 2>/dev/null; then
        log_error "Failed to enable and start annad service"
        log_error "Showing recent logs:"
        print_line
        $SUDO journalctl -u annad -b -n 80 --no-pager 2>/dev/null || true
        print_line
        log_to_file "Installer: action=${PLANNED_ACTION} result=daemon_start_failed"
        exit 1
    fi

    # Wait for daemon to stabilize
    sleep 2

    # Validate daemon is actually running
    if ! $SUDO systemctl is-active --quiet annad; then
        log_error "annad service failed to start"
        log_error "Showing recent logs:"
        print_line
        $SUDO journalctl -u annad -b -n 80 --no-pager 2>/dev/null || true
        print_line
        log_to_file "Installer: action=${PLANNED_ACTION} result=daemon_not_active"
        exit 1
    fi

    log_ok "annad is running"

    print_header "COMPLETE"
    printf "  Anna v%s installed and running.\n" "$LATEST_VERSION"
    printf "\n"
    printf "  Anna is a ${CYAN}system telemetry daemon${NC}.\n"
    printf "  It observes hardware, services, and processes.\n"
    printf "\n"
    printf "  Check status:        ${CYAN}annactl status${NC}\n"
    printf "  Software overview:   ${CYAN}annactl sw${NC}\n"
    printf "  Software details:    ${CYAN}annactl sw <name>${NC}\n"
    printf "  Hardware overview:   ${CYAN}annactl hw${NC}\n"
    printf "  Hardware details:    ${CYAN}annactl hw <name>${NC}\n"
    printf "\n"
    print_line
}

main "$@"
