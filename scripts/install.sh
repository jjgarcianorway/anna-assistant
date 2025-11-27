#!/bin/bash
# Anna Installer v2.0.0
#
# This installer is versioned INDEPENDENTLY from Anna itself.
# Installer version: 2.x.x
# Anna version: fetched from GitHub releases
#
# Behavior:
#   - Detects installed version (if any)
#   - Fetches latest Anna version from GitHub
#   - Compares and shows planned action
#   - Non-interactive default: update if newer, skip if same/older
#   - Interactive mode (-i): prompt for confirmation
#   - Never clobbers config/data unless --reset is passed
#   - Logs all actions to /var/log/anna/install.log
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

# Note: Using set -u (undefined vars) but NOT set -e (exit on error)
# because we handle errors explicitly and set -e can cause unexpected exits
set -uo pipefail

# ============================================================
# CONFIGURATION
# ============================================================

# Installer version (independent from Anna version)
INSTALLER_VERSION="2.0.0"
GITHUB_REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/anna"
DATA_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
RUN_DIR="/run/anna"
PROBES_DIR="/usr/share/anna/probes"

# ============================================================
# COLORS AND FORMATTING (ASCII-only, sysadmin style)
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

# ============================================================
# LOGGING
# ============================================================

log_to_file() {
    local msg="$1"
    local log_file="/var/log/anna/install.log"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    # Create log dir if missing (best effort)
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

# Compare semantic versions: returns 0 if $1 < $2, 1 if $1 == $2, 2 if $1 > $2
version_compare() {
    local v1="$1"
    local v2="$2"

    # Strip leading 'v' if present
    v1="${v1#v}"
    v2="${v2#v}"

    if [[ "$v1" == "$v2" ]]; then
        echo 1
        return
    fi

    # Compare using sort -V
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

    # Try annactl -V first (primary method)
    # Use timeout and redirect stdin to /dev/null to prevent any stdin reading
    if command -v annactl &>/dev/null; then
        local output
        output=$(timeout 5 annactl -V </dev/null 2>&1 | head -20) || true
        # Look for "Anna Assistant vX.Y.Z" or "v0.X.Y" pattern specifically
        if [[ "$output" =~ Anna[[:space:]]+(Assistant[[:space:]]+)?v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            INSTALLED_VERSION="${BASH_REMATCH[2]}"
        fi
    fi

    # Fallback: check binary for embedded CARGO_PKG_VERSION
    # Note: Don't use generic strings search as it picks up library versions
    if [[ -z "$INSTALLED_VERSION" ]] && [[ -x "${INSTALL_DIR}/annactl" ]]; then
        local output
        # Look specifically for Anna version patterns in the binary
        output=$(strings "${INSTALL_DIR}/annactl" 2>/dev/null | grep -E 'Anna.*[0-9]+\.[0-9]+\.[0-9]+' | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || true
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
        # Extract tag_name from JSON
        LATEST_VERSION=$(echo "$response" | grep -oP '"tag_name":\s*"v?\K[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
    fi

    # Fail if we can't fetch the version - don't use stale hardcoded version
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
    printf "\n${BOLD}Anna Assistant Installer${NC}\n"
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
            0)  # Installed < Latest
                printf "  ${GREEN}Update available${NC} - v%s -> v%s\n" "$INSTALLED_VERSION" "$LATEST_VERSION"
                PLANNED_ACTION="update"
                ;;
            1)  # Installed == Latest
                printf "  ${CYAN}Same version${NC} - v%s already installed\n" "$INSTALLED_VERSION"
                PLANNED_ACTION="reinstall"
                ;;
            2)  # Installed > Latest
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

    if [[ "$INTERACTIVE" == "false" ]]; then
        # Non-interactive defaults
        case "$action" in
            install|update|reinstall)
                return 0  # Proceed - always allow install/update/reinstall
                ;;
            downgrade)
                log_warn "Downgrade not allowed in non-interactive mode"
                return 1  # Skip
                ;;
        esac
    fi

    # Interactive mode
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
    read -r answer
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

    # v0.11.0: Binaries are named simply annad/annactl (no arch suffix)
    local annad_file="annad"
    local annactl_file="annactl"

    # Download files
    if command -v curl &>/dev/null; then
        curl -fsSL "${base_url}/${annad_file}" -o "${TMP_DIR}/annad" || {
            log_error "Failed to download annad"
            return 1
        }
        curl -fsSL "${base_url}/${annactl_file}" -o "${TMP_DIR}/annactl" || {
            log_error "Failed to download annactl"
            return 1
        }
        curl -fsSL "${base_url}/SHA256SUMS" -o "${TMP_DIR}/SHA256SUMS" || {
            log_error "Failed to download checksums"
            return 1
        }
    else
        wget -q "${base_url}/${annad_file}" -O "${TMP_DIR}/annad" || return 1
        wget -q "${base_url}/${annactl_file}" -O "${TMP_DIR}/annactl" || return 1
        wget -q "${base_url}/SHA256SUMS" -O "${TMP_DIR}/SHA256SUMS" || return 1
    fi

    log_ok "Downloaded binaries"

    # Verify checksums
    log_info "Verifying checksums..."
    cd "$TMP_DIR"

    local annad_sum annactl_sum
    annad_sum=$(sha256sum annad | awk '{print $1}')
    annactl_sum=$(sha256sum annactl | awk '{print $1}')

    if grep -q "$annad_sum" SHA256SUMS && grep -q "$annactl_sum" SHA256SUMS; then
        log_ok "Checksums verified"
    else
        log_error "Checksum verification failed!"
        return 1
    fi
}

install_binaries() {
    log_info "Installing binaries..."

    # Backup existing if updating
    if [[ -f "${INSTALL_DIR}/annad" ]]; then
        $SUDO cp "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annad.bak" 2>/dev/null || true
    fi
    if [[ -f "${INSTALL_DIR}/annactl" ]]; then
        $SUDO cp "${INSTALL_DIR}/annactl" "${INSTALL_DIR}/annactl.bak" 2>/dev/null || true
    fi

    # Install (atomic move)
    $SUDO mv "${TMP_DIR}/annad" "${INSTALL_DIR}/annad"
    $SUDO mv "${TMP_DIR}/annactl" "${INSTALL_DIR}/annactl"
    $SUDO chmod 755 "${INSTALL_DIR}/annad"
    $SUDO chmod 755 "${INSTALL_DIR}/annactl"

    log_ok "Installed binaries to ${INSTALL_DIR}"
}

create_user_and_dirs() {
    log_info "Creating user and directories..."

    # Create anna user if not exists
    if ! id "anna" &>/dev/null; then
        $SUDO useradd -r -s /bin/false -d "$DATA_DIR" anna 2>/dev/null || true
        log_ok "Created user 'anna'"
    else
        log_ok "User 'anna' exists"
    fi

    # Create directories (never wipe existing)
    $SUDO mkdir -p "$DATA_DIR" "$LOG_DIR" "$RUN_DIR" "$CONFIG_DIR" "$PROBES_DIR"

    # v0.11.0: Knowledge store directory
    $SUDO mkdir -p "${DATA_DIR}/knowledge"

    # Set ownership
    $SUDO chown -R anna:anna "$DATA_DIR" "$LOG_DIR" "$RUN_DIR"

    log_ok "Created directories"
    log_ok "Knowledge store: ${DATA_DIR}/knowledge"
}

install_systemd_service() {
    log_info "Installing systemd service..."

    local service_file="/etc/systemd/system/annad.service"

    # Check if service exists and is different
    if [[ -f "$service_file" ]] && [[ "$RESET_MODE" == "false" ]]; then
        log_ok "Systemd service exists (preserving)"
    else
        $SUDO tee "$service_file" > /dev/null << 'EOF'
[Unit]
Description=Anna Daemon - Evidence Oracle
Documentation=https://github.com/jjgarcianorway/anna-assistant
After=network.target

[Service]
Type=simple
User=anna
Group=anna
ExecStart=/usr/local/bin/annad
WorkingDirectory=/usr/share/anna
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX AF_NETLINK

# Allow reading system info
ReadOnlyPaths=/proc /sys
ReadWritePaths=/var/lib/anna /var/log/anna /run/anna

[Install]
WantedBy=multi-user.target
EOF
        $SUDO systemctl daemon-reload
        log_ok "Installed systemd service"
    fi
}

write_config() {
    local config_file="${CONFIG_DIR}/config.toml"

    # Never overwrite existing config unless --reset
    if [[ -f "$config_file" ]] && [[ "$RESET_MODE" == "false" ]]; then
        log_ok "Config exists (preserving)"
        return
    fi

    log_info "Writing default configuration..."

    $SUDO tee "$config_file" > /dev/null << EOF
# Anna v${LATEST_VERSION} Configuration
# This file was auto-generated. Feel free to customize.

[core]
mode = "normal"

[llm]
preferred_model = "llama3.2:3b"
fallback_model = "llama3.2:3b"
selection_mode = "auto"

[update]
enabled = false
interval_seconds = 86400
channel = "main"

[log]
level = "debug"
daemon_enabled = true
requests_enabled = true
llm_enabled = true
EOF

    log_ok "Configuration written to ${config_file}"
}

verify_installation() {
    log_info "Verifying installation..."

    local errors=0

    # Check binaries
    if [[ -x "${INSTALL_DIR}/annad" ]]; then
        log_ok "annad binary OK"
    else
        log_error "annad binary missing or not executable"
        ((errors++))
    fi

    if [[ -x "${INSTALL_DIR}/annactl" ]]; then
        # Quick version check - use timeout and stdin redirect
        local version
        version=$(timeout 5 "${INSTALL_DIR}/annactl" -V </dev/null 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || version="unknown"
        if [[ "$version" == "$LATEST_VERSION" ]]; then
            log_ok "annactl v${version} OK"
        else
            log_warn "annactl version mismatch: expected ${LATEST_VERSION}, got ${version}"
        fi
    else
        log_error "annactl binary missing or not executable"
        ((errors++)) || true
    fi

    # Check config
    if [[ -f "${CONFIG_DIR}/config.toml" ]]; then
        log_ok "Configuration OK"
    else
        log_warn "Configuration file missing"
    fi

    return $errors
}

# ============================================================
# MAIN
# ============================================================

main() {
    trap cleanup EXIT

    # Parse arguments
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

    # Detect architecture
    detect_arch

    # Print banner
    print_banner

    # Detect versions
    detect_installed_version
    fetch_latest_version

    # Show version summary
    print_version_summary

    # Determine and show planned action
    print_planned_action

    # Confirm action
    if ! confirm_action "$PLANNED_ACTION"; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=declined installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"
        exit 0
    fi

    # Request sudo
    check_sudo
    request_sudo

    # Log start
    log_to_file "Installer: action=${PLANNED_ACTION} starting installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"

    print_header "INSTALLING"

    # Download
    if ! download_binaries; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=download_failed"
        exit 1
    fi

    # Install
    create_user_and_dirs
    install_binaries
    install_systemd_service
    write_config

    # Verify
    print_header "VERIFICATION"
    if verify_installation; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=success version=${LATEST_VERSION}"
    else
        log_to_file "Installer: action=${PLANNED_ACTION} result=partial_success version=${LATEST_VERSION}"
    fi

    # Start daemon
    print_header "STARTING DAEMON"
    log_info "Starting annad service..."
    if $SUDO systemctl restart annad 2>/dev/null; then
        sleep 1
        if $SUDO systemctl is-active --quiet annad; then
            log_ok "annad is running"
        else
            log_warn "annad failed to start, check: journalctl -u annad"
        fi
    else
        log_warn "Could not start annad service"
    fi

    # Enable at boot
    if ! $SUDO systemctl is-enabled --quiet annad 2>/dev/null; then
        $SUDO systemctl enable annad 2>/dev/null && log_ok "Enabled annad at boot" || true
    fi

    # Final message
    print_header "COMPLETE"
    printf "  Anna v%s installed and running.\n" "$LATEST_VERSION"
    printf "\n"
    printf "  Check status:    ${CYAN}annactl status${NC}\n"
    printf "  Ask a question:  ${CYAN}annactl \"How many CPU cores?\"${NC}\n"
    printf "\n"
    print_line
}

main "$@"
