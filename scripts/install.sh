#!/bin/bash
# Anna Installer - curl -sSL <url>/install.sh | bash
# v0.3.106: Improved version fetching with better error handling
set -e

REPO="jjgarcianorway/anna-assistant"

# Fetch latest version from GitHub releases
fetch_version() {
    local response version

    # Check if curl is available
    if ! command -v curl &>/dev/null; then
        echo "Error: curl is required but not installed" >&2
        echo "Install with: sudo pacman -S curl (Arch) or sudo apt install curl (Debian/Ubuntu)" >&2
        exit 1
    fi

    # Fetch from GitHub API
    response=$(curl -sSL --connect-timeout 10 "https://api.github.com/repos/${REPO}/releases/latest" 2>&1)

    # Check for rate limiting
    if echo "$response" | grep -q "API rate limit"; then
        echo "Error: GitHub API rate limit exceeded" >&2
        echo "Try again in a few minutes, or install manually from:" >&2
        echo "  https://github.com/${REPO}/releases/latest" >&2
        exit 1
    fi

    # Check for network errors
    if echo "$response" | grep -qi "could not resolve\|connection refused\|timed out"; then
        echo "Error: Cannot connect to GitHub" >&2
        echo "Check your internet connection and try again" >&2
        exit 1
    fi

    # Extract version from response
    version=$(echo "$response" | grep '"tag_name"' | head -1 | sed -E 's/.*"v([^"]+)".*/\1/')

    # Validate version format (should be like 0.3.106)
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Could not fetch version from GitHub" >&2
        echo "Response was: ${response:0:200}" >&2
        exit 1
    fi

    echo "$version"
}

VERSION=$(fetch_version)
INSTALL_DIR="/usr/local/bin" CONFIG_DIR="/etc/anna" STATE_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna" RUN_DIR="/run/anna" SYSTEMD_DIR="/etc/systemd/system" ANNA_GROUP="anna"

# Colors (24-bit true color)
C_HEADER=$'\033[38;2;255;210;120m' C_OK=$'\033[38;2;120;255;120m' C_ERR=$'\033[38;2;255;100;100m'
C_DIM=$'\033[38;2;140;140;140m' C_CYAN=$'\033[38;2;120;200;255m' C_BOLD=$'\033[1m' C_RESET=$'\033[0m'
SYM_OK="+" SYM_ERR="x" SYM_ARROW=">"
HR="${C_DIM}──────────────────────────────────────────────────────────────────────────────${C_RESET}"
USERNAME=$(whoami)

print_header() { echo ""; echo "${C_HEADER}anna-install v${VERSION}${C_RESET}"; echo "$HR"; echo "No hidden steps. Every action shown. Checksums mandatory."; echo "$HR"; echo ""; }
print_greeting() { echo ""; echo "${C_CYAN}Hello ${USERNAME}${C_RESET}, thanks for giving me the opportunity to live"; echo "in your computer! I promise to take good care of it... and you! ;)"; echo ""; }
print_section() { echo "${C_DIM}[${C_RESET}$1${C_DIM}]${C_RESET} $2"; }
print_ok() { echo "  ${C_OK}${SYM_OK}${C_RESET} $1"; }
print_item_ok() { printf "  %-20s ${C_OK}${SYM_OK}${C_RESET}\n" "$1"; }
print_err() { echo "  ${C_ERR}${SYM_ERR}${C_RESET} $1"; }
print_footer() {
    echo ""; echo "$HR"
    if [[ "${GROUP_ADDED:-false}" == "true" ]]; then
        echo "  ${C_ERR}IMPORTANT: Log out and back in before running annactl.${C_RESET}"
        echo "  Your user was just added to the 'anna' group."
        echo "  Group membership activates on next login — annactl will fail until then."
        echo ""
    fi
    if [[ "${DAEMON_START_FAILED:-false}" == "true" ]]; then
        echo "  ${C_ERR}WARNING: annad failed to start. Check logs: journalctl -u annad -n 30${C_RESET}"
        echo ""
    else
        echo "  annad is running. Try: ${C_BOLD}annactl status${C_RESET}"
    fi
    if [[ -f "${CONFIG_DIR}/telegram.env" ]]; then
        echo "  Telegram: ${C_OK}Configured${C_RESET} - message your bot to test!"
    else
        echo "  Telegram: annactl telegram setup"
    fi
    echo "$HR"; echo ""
}
fail() { print_err "$1"; exit 1; }

detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64) echo "x86_64" ;;
        aarch64|arm64)
            fail "ARM (aarch64) not yet supported. Please use x86_64 or build from source: git clone https://github.com/${REPO} && cd anna-assistant && cargo build --release"
            ;;
        *) fail "Unsupported architecture: $arch. Supported: x86_64" ;;
    esac
}

preflight() {
    print_section "preflight" "linux + systemd + tools"
    ARCH=$(detect_arch); print_ok "arch: ${ARCH}"
    command -v systemctl &>/dev/null || fail "systemd not found"; print_ok "systemd: ok"
    local missing=""
    for tool in curl sha256sum; do command -v "$tool" &>/dev/null || missing="$missing $tool"; done
    [[ -n "$missing" ]] && fail "missing tools:$missing"
    print_ok "curl sha256sum: ok"; echo ""
}

fetch_artifacts() {
    print_section "fetch" "release artifacts"
    local base_url="https://github.com/${REPO}/releases/download/v${VERSION}"
    TMPDIR=$(mktemp -d)
    curl -sSL "${base_url}/annactl-linux-${ARCH}" -o "${TMPDIR}/annactl" 2>/dev/null && print_item_ok "annactl-${ARCH}" || fail "failed to download annactl"
    curl -sSL "${base_url}/annad-linux-${ARCH}" -o "${TMPDIR}/annad" 2>/dev/null && print_item_ok "annad-${ARCH}" || fail "failed to download annad"
    curl -sSL "${base_url}/SHA256SUMS" -o "${TMPDIR}/SHA256SUMS" 2>/dev/null && print_item_ok "SHA256SUMS" || fail "failed to download SHA256SUMS"
    echo ""
}

verify_checksums() {
    print_section "verify" "checksums"
    cd "$TMPDIR"
    local annactl_expected=$(grep "annactl-linux-${ARCH}" SHA256SUMS | awk '{print $1}')
    local annad_expected=$(grep "annad-linux-${ARCH}" SHA256SUMS | awk '{print $1}')
    local annactl_actual=$(sha256sum annactl | awk '{print $1}')
    local annad_actual=$(sha256sum annad | awk '{print $1}')
    [[ "$annactl_expected" = "$annactl_actual" ]] && printf "  annactl  ${C_OK}OK${C_RESET}\n" || fail "annactl checksum mismatch"
    [[ "$annad_expected" = "$annad_actual" ]] && printf "  annad    ${C_OK}OK${C_RESET}\n" || fail "annad checksum mismatch"
    echo ""
}

request_sudo() {
    print_section "sudo" "needed to write to /usr/local/bin, /etc, systemd, /var/lib"
    echo ""; echo "  Anna needs root access to:"
    echo "    ${SYM_ARROW} Install binaries to /usr/local/bin"
    echo "    ${SYM_ARROW} Create config in /etc/anna"
    echo "    ${SYM_ARROW} Create data directory in /var/lib/anna"
    echo "    ${SYM_ARROW} Install systemd service"; echo ""
    if [[ "$EUID" -eq 0 ]]; then SUDO=""; print_ok "already running as root"
    else
        echo "  ${SYM_ARROW} Requesting sudo access..."
        sudo -v && { SUDO="sudo"; print_ok "sudo access granted"; } || fail "sudo access required but denied"
    fi
    echo ""
}

stop_existing_service() {
    if systemctl is-active --quiet annad 2>/dev/null; then
        print_section "upgrade" "stopping existing annad service"
        $SUDO systemctl stop annad; print_ok "annad stopped"; echo ""; UPGRADE_MODE=true
    else UPGRADE_MODE=false; fi
}

cleanup_stale_binaries() {
    local user_local_bin="${HOME}/.local/bin" stale_found=false
    if [[ -x "${user_local_bin}/annactl" ]]; then
        stale_found=true; print_section "cleanup" "removing stale binaries"
        rm -f "${user_local_bin}/annactl"; print_ok "removed ${user_local_bin}/annactl"
    fi
    if [[ -x "${user_local_bin}/annad" ]]; then
        [[ "$stale_found" = false ]] && print_section "cleanup" "removing stale binaries"
        rm -f "${user_local_bin}/annad"; print_ok "removed ${user_local_bin}/annad"; stale_found=true
    fi
    [[ "$stale_found" = true ]] && echo "" || true
}

install_binaries() {
    print_section "install" "binaries"
    chmod +x "${TMPDIR}/annactl" "${TMPDIR}/annad"
    $SUDO cp "${TMPDIR}/annactl" "${INSTALL_DIR}/annactl"; print_item_ok "/usr/local/bin/annactl"
    $SUDO cp "${TMPDIR}/annad" "${INSTALL_DIR}/annad"; print_item_ok "/usr/local/bin/annad"
    echo ""
}

setup_group() {
    print_section "security" "group setup"
    getent group "$ANNA_GROUP" >/dev/null 2>&1 && print_ok "group exists: ${ANNA_GROUP}" || { $SUDO groupadd "$ANNA_GROUP"; print_ok "created group: ${ANNA_GROUP}"; }
    if groups "$USERNAME" 2>/dev/null | grep -q "\b${ANNA_GROUP}\b"; then
        print_ok "${USERNAME} already in ${ANNA_GROUP} group"
    else
        $SUDO usermod -aG "$ANNA_GROUP" "$USERNAME"; print_ok "added ${USERNAME} to ${ANNA_GROUP} group"
        # Group membership only activates on next login — flag this for print_footer
        GROUP_ADDED=true
    fi
    echo ""
}

install_directories() {
    print_section "install" "directories"
    $SUDO mkdir -p "$CONFIG_DIR"; $SUDO chmod 755 "$CONFIG_DIR"; print_item_ok "/etc/anna (755 root:root)"
    $SUDO mkdir -p "$STATE_DIR"; $SUDO chown root:$ANNA_GROUP "$STATE_DIR"; $SUDO chmod 750 "$STATE_DIR"; print_item_ok "/var/lib/anna (750 root:anna)"
    for subdir in backups wiki recipes; do
        $SUDO mkdir -p "${STATE_DIR}/${subdir}"; $SUDO chown root:$ANNA_GROUP "${STATE_DIR}/${subdir}"; $SUDO chmod 750 "${STATE_DIR}/${subdir}"
        print_item_ok "/var/lib/anna/${subdir} (750 root:anna)"
    done
    $SUDO mkdir -p "$LOG_DIR"; $SUDO chown root:$ANNA_GROUP "$LOG_DIR"; $SUDO chmod 750 "$LOG_DIR"; print_item_ok "/var/log/anna (750 root:anna)"
    $SUDO mkdir -p "$RUN_DIR"; $SUDO chown root:$ANNA_GROUP "$RUN_DIR"; $SUDO chmod 750 "$RUN_DIR"; print_item_ok "/run/anna (750 root:anna)"
    $SUDO mkdir -p "${STATE_DIR}/models"; $SUDO chown root:$ANNA_GROUP "${STATE_DIR}/models"; $SUDO chmod 750 "${STATE_DIR}/models"
    print_item_ok "/var/lib/anna/models (750 root:anna)"; echo ""
}

install_tmpfiles() {
    print_section "install" "tmpfiles.d"
    $SUDO tee "/etc/tmpfiles.d/anna.conf" >/dev/null <<'EOF'
d /run/anna 0750 root anna -
EOF
    print_item_ok "/etc/tmpfiles.d/anna.conf (750 root:anna)"; echo ""
}

install_config() {
    print_section "install" "config (create if missing)"
    if [[ ! -f "${CONFIG_DIR}/config.toml" ]]; then
        $SUDO tee "${CONFIG_DIR}/config.toml" >/dev/null <<'EOF'
[daemon]
debug_mode = true
auto_update = true
update_interval = 600
[llm]
provider = "ollama"
EOF
    fi
    print_item_ok "/etc/anna/config.toml"; echo ""
}

setup_telegram() {
    print_section "telegram" "optional mobile access"
    echo ""
    echo "  ${C_BOLD}Control Anna from your phone!${C_RESET}"
    echo ""
    echo "  Anna can send you:"
    echo "    ${SYM_ARROW} Morning briefings with system health charts"
    echo "    ${SYM_ARROW} Critical alerts when attention is needed"
    echo "    ${SYM_ARROW} Instant answers to your questions"
    echo ""
    echo "  This requires creating a free Telegram bot (takes 2 minutes)."
    echo ""
    read -p "  Set up Telegram now? [y/N] " -n 1 -r; echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        print_ok "Skipped - you can set it up anytime with:"
        echo "    ${C_BOLD}annactl telegram setup${C_RESET}"
        echo ""; return
    fi

    echo ""
    echo "  ${C_CYAN}Step 1: Create a bot${C_RESET}"
    echo "    1. Open Telegram and search for ${C_BOLD}@BotFather${C_RESET}"
    echo "    2. Send: ${C_BOLD}/newbot${C_RESET}"
    echo "    3. Choose a name (e.g., 'My Anna Bot')"
    echo "    4. Choose a username (must end in 'bot', e.g., 'my_anna_bot')"
    echo "    5. Copy the token BotFather gives you"
    echo ""
    echo "  ${C_DIM}Example token: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz${C_RESET}"
    echo ""
    read -p "  Paste your bot token: " BOT_TOKEN

    if [[ -z "$BOT_TOKEN" ]]; then
        echo ""
        print_err "No token provided. Run ${C_BOLD}annactl telegram setup${C_RESET} later."
        echo ""; return
    fi

    echo ""
    echo "  ${C_CYAN}Step 2: Get your Telegram user ID${C_RESET}"
    echo "    1. Search for ${C_BOLD}@userinfobot${C_RESET} on Telegram"
    echo "    2. Send any message (e.g., /start)"
    echo "    3. The bot will reply with your user ID (a number)"
    echo ""
    echo "  ${C_DIM}Example ID: 123456789${C_RESET}"
    echo ""
    read -p "  Paste your Telegram user ID: " USER_ID

    if [[ -z "$USER_ID" ]]; then
        echo ""
        print_err "No user ID provided. Run ${C_BOLD}annactl telegram setup${C_RESET} later."
        echo ""; return
    fi

    # Save to telegram.env
    $SUDO tee "${CONFIG_DIR}/telegram.env" >/dev/null <<EOF
ANNA_TELEGRAM_TOKEN=${BOT_TOKEN}
ANNA_TELEGRAM_USERS=${USER_ID}
EOF
    $SUDO chmod 640 "${CONFIG_DIR}/telegram.env"
    $SUDO chown root:$ANNA_GROUP "${CONFIG_DIR}/telegram.env"

    echo ""
    print_ok "Telegram configured!"
    echo "    After install, open Telegram and message your bot to test it."
    echo ""
}

install_service() {
    print_section "service" "systemd"

    # Always include telegram.env with - prefix (optional, won't fail if missing)
    $SUDO tee "${SYSTEMD_DIR}/annad.service" >/dev/null <<'EOF'
[Unit]
Description=Anna Assistant Daemon
After=network.target ollama.service
Wants=ollama.service
[Service]
Type=notify
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=3
WatchdogSec=60
TimeoutStopSec=10
MemoryMax=2G
RuntimeDirectory=anna
RuntimeDirectoryMode=0750
RuntimeDirectoryGroup=anna
Environment=RUST_BACKTRACE=1
EnvironmentFile=-/etc/anna/telegram.env
[Install]
WantedBy=multi-user.target
EOF
    print_item_ok "annad.service installed"
    $SUDO systemctl daemon-reload; $SUDO systemctl enable annad --quiet; print_item_ok "enable"
    if $SUDO systemctl start annad; then
        # Wait up to 5s for the socket to appear
        for i in $(seq 1 10); do
            [[ -S /run/anna/anna.sock ]] && { print_item_ok "start (socket ready)"; break; }
            sleep 0.5
        done
        [[ -S /run/anna/anna.sock ]] || { print_err "annad started but socket not ready — check: journalctl -u annad -n 20"; }
    else
        print_err "annad failed to start — check: journalctl -u annad -n 20"
        DAEMON_START_FAILED=true
    fi
    echo ""
}

verify_binaries() {
    print_section "verify" "installed binaries"
    local annactl_ver annad_ver
    annactl_ver=$("${INSTALL_DIR}/annactl" --version 2>/dev/null | head -1)
    [[ -n "$annactl_ver" ]] && printf "  annactl  ${C_OK}${annactl_ver}${C_RESET}\n" || fail "annactl --version failed"
    annad_ver=$("${INSTALL_DIR}/annad" --version 2>/dev/null | head -1)
    [[ -n "$annad_ver" ]] && printf "  annad    ${C_OK}${annad_ver}${C_RESET}\n" || fail "annad --version failed"
    local annactl_base=$(echo "$annactl_ver" | sed 's/annactl //' | cut -d' ' -f1)
    local annad_base=$(echo "$annad_ver" | sed 's/annad //' | cut -d' ' -f1)
    [[ "$annactl_base" = "$annad_base" ]] && print_ok "versions match: ${annactl_base}" || { print_err "version mismatch: annactl=${annactl_base} annad=${annad_base}"; echo "  ${C_ERR}Warning: Client and daemon versions should match${C_RESET}"; }
    echo ""
}

print_handoff() { print_section "handoff" "annad will bootstrap the required local LLM (ollama + models)"; }
cleanup() { rm -rf "$TMPDIR" 2>/dev/null || true; }
trap cleanup EXIT

main() {
    print_header; print_greeting; preflight; fetch_artifacts; verify_checksums
    cleanup_stale_binaries; request_sudo; stop_existing_service
    install_binaries; verify_binaries; setup_group
    install_directories; install_tmpfiles; install_config
    setup_telegram  # Interactive Telegram setup
    install_service
    print_handoff; print_footer
}

main "$@"
