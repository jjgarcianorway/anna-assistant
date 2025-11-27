#!/bin/bash
# Anna v0.6.0 - Installation Script
# Downloads from GitHub, detects hardware, selects models, installs components
# Requests sudo only when needed - not for the entire script

set -euo pipefail

# Colors
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
MAGENTA=$'\033[0;35m'
CYAN=$'\033[0;36m'
NC=$'\033[0m'
BOLD=$'\033[1m'

# Icons
ICON_CHECK="✓"
ICON_CROSS="✗"
ICON_INFO="ℹ"
ICON_WARN="⚠"
ICON_ROCKET="🚀"
ICON_DOWN="⬇"
ICON_LOCK="🔒"

VERSION="0.6.0"
GITHUB_REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"

# Temp directory for downloads
TMP_DIR=""

print_banner() {
    printf "\n${MAGENTA}${BOLD}  Anna v${VERSION}${NC}\n"
    printf "   Your intelligent Linux assistant\n\n"
}

log_info() {
    printf "${BLUE}${ICON_INFO}${NC}  %s\n" "$1"
}

log_success() {
    printf "${GREEN}${ICON_CHECK}${NC}  %s\n" "$1"
}

log_warn() {
    printf "${YELLOW}${ICON_WARN}${NC}  %s\n" "$1"
}

log_error() {
    printf "${RED}${ICON_CROSS}${NC}  %s\n" "$1"
}

# Check if we can use sudo
check_sudo() {
    if command -v sudo &> /dev/null; then
        SUDO="sudo"
        log_info "Will use ${CYAN}sudo${NC} for privileged operations"
    elif [[ $EUID -eq 0 ]]; then
        SUDO=""
        log_info "Running as root"
    else
        log_error "sudo not found and not running as root"
        echo "   Please install sudo or run as root"
        exit 1
    fi
}

# Request sudo upfront to cache credentials
request_sudo() {
    if [[ -n "$SUDO" ]]; then
        printf "\n${ICON_LOCK}  ${BOLD}Sudo access required for installation${NC}\n"
        echo "   This will install binaries to ${INSTALL_DIR}"
        echo "   and create system directories/services"
        echo ""
        $SUDO -v || {
            log_error "Failed to obtain sudo access"
            exit 1
        }
        log_success "Sudo access granted"
        echo ""
    fi
}

# Detect architecture
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
    log_info "Architecture: ${ARCH} (${ARCH_NAME})"
}

# Detect hardware
detect_hardware() {
    log_info "Detecting hardware..."

    # RAM
    RAM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    RAM_GB=$((RAM_KB / 1024 / 1024))
    echo "   RAM: ${RAM_GB} GB"

    # CPU cores (physical) and threads
    CPU_THREADS=$(nproc)
    CPU_CORES_PHYSICAL=$(lscpu 2>/dev/null | awk '/^Core\(s\) per socket:/ {print $4}')
    CPU_SOCKETS=$(lscpu 2>/dev/null | awk '/^Socket\(s\):/ {print $2}')
    if [ -n "$CPU_CORES_PHYSICAL" ] && [ -n "$CPU_SOCKETS" ] && [ "$CPU_CORES_PHYSICAL" -gt 0 ] 2>/dev/null; then
        CPU_CORES=$((CPU_CORES_PHYSICAL * CPU_SOCKETS))
        printf "   CPU: ${CYAN}%d${NC} cores (${CYAN}%d${NC} threads)\n" "$CPU_CORES" "$CPU_THREADS"
    else
        CPU_CORES=$CPU_THREADS
        printf "   CPU: ${CYAN}%d${NC} threads\n" "$CPU_CORES"
    fi

    # GPU detection
    HAS_GPU=false
    VRAM_GB=0

    if command -v nvidia-smi &> /dev/null; then
        if nvidia-smi &> /dev/null; then
            HAS_GPU=true
            VRAM_MB=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1)
            VRAM_GB=$((VRAM_MB / 1024))
            echo "   GPU: NVIDIA (${VRAM_GB} GB VRAM)"
        fi
    elif [[ -d /sys/class/drm/card0 ]]; then
        HAS_GPU=true
        echo "   GPU: Detected (DRM)"
    else
        echo "   GPU: None detected"
    fi
}

# Select LLM models based on hardware
select_models() {
    log_info "Selecting optimal LLM models..."

    # LLM-A (Orchestrator) - fast, stable
    if [[ "$HAS_GPU" == "true" ]]; then
        LLM_A="mistral-nemo"
    elif [[ $CPU_CORES -ge 8 ]]; then
        LLM_A="qwen2.5:3b"
    else
        LLM_A="llama3.2:3b"
    fi

    # LLM-B (Expert) - based on RAM
    if [[ $RAM_GB -ge 32 ]] && [[ "$HAS_GPU" == "true" ]]; then
        LLM_B="qwen2.5:32b"
    elif [[ $RAM_GB -ge 16 ]]; then
        LLM_B="qwen2.5:14b"
    else
        LLM_B="qwen2.5:7b"
    fi

    printf "   Orchestrator (LLM-A): ${CYAN}%s${NC}\n" "$LLM_A"
    printf "   Expert (LLM-B): ${CYAN}%s${NC}\n" "$LLM_B"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."

    # Check for curl or wget
    if command -v curl &> /dev/null; then
        DOWNLOADER="curl"
        log_success "curl found"
    elif command -v wget &> /dev/null; then
        DOWNLOADER="wget"
        log_success "wget found"
    else
        log_error "Neither curl nor wget found. Please install one."
        exit 1
    fi

    # Check for Ollama
    if ! command -v ollama &> /dev/null; then
        log_warn "Ollama not found"
        echo "   Install from: https://ollama.ai"
        echo "   Run: curl -fsSL https://ollama.ai/install.sh | sh"
        OLLAMA_MISSING=true
    else
        log_success "Ollama found"
        OLLAMA_MISSING=false
    fi
}

# Download file helper
download_file() {
    local url="$1"
    local output="$2"

    printf "   ${ICON_DOWN}  Downloading %s...\n" "$(basename "$output")"

    if [[ "$DOWNLOADER" == "curl" ]]; then
        curl -fsSL "$url" -o "$output"
    else
        wget -q "$url" -O "$output"
    fi
}

# Download binaries to temp directory (no sudo needed)
download_binaries() {
    log_info "Downloading binaries from GitHub..."

    local base_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}"

    # Create temp directory
    TMP_DIR=$(mktemp -d)

    # Download annad
    download_file "${base_url}/annad-${VERSION}-${ARCH_NAME}" "${TMP_DIR}/annad"

    # Download annactl
    download_file "${base_url}/annactl-${VERSION}-${ARCH_NAME}" "${TMP_DIR}/annactl"

    # Download checksums
    download_file "${base_url}/SHA256SUMS" "${TMP_DIR}/SHA256SUMS"

    # Verify checksums
    log_info "Verifying checksums..."
    cd "$TMP_DIR"
    if grep -q "$(sha256sum annad | awk '{print $1}')" SHA256SUMS && \
       grep -q "$(sha256sum annactl | awk '{print $1}')" SHA256SUMS; then
        log_success "Checksums verified"
    else
        log_error "Checksum verification failed!"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    log_success "Downloaded and verified binaries"
}

# Install binaries (requires sudo)
install_binaries() {
    log_info "Installing binaries to ${INSTALL_DIR}..."

    $SUDO mv "${TMP_DIR}/annad" "${INSTALL_DIR}/annad"
    $SUDO mv "${TMP_DIR}/annactl" "${INSTALL_DIR}/annactl"
    $SUDO chmod 755 "${INSTALL_DIR}/annad"
    $SUDO chmod 755 "${INSTALL_DIR}/annactl"

    log_success "Installed annad and annactl"
}

# Download LLM models (no sudo needed)
download_models() {
    if [ "$OLLAMA_MISSING" = "true" ]; then
        log_warn "Skipping model download (Ollama not installed)"
        return
    fi

    log_info "Downloading LLM models (this may take a while)..."
    echo ""

    # Pull model with progress
    pull_model() {
        model="$1"
        printf "   ${CYAN}⬇${NC}  Pulling ${BOLD}%s${NC}...\n" "$model"

        # Use 'script' to create a pseudo-TTY for ollama progress bar
        if command -v script >/dev/null 2>&1; then
            # script command exists - use it for TTY emulation
            if script -q -c "ollama pull '$model'" /dev/null; then
                log_success "Downloaded ${model}"
                return 0
            else
                log_warn "Failed to download ${model}"
                return 1
            fi
        else
            # Fallback without progress bar
            if ollama pull "$model"; then
                log_success "Downloaded ${model}"
                return 0
            else
                log_warn "Failed to download ${model}"
                return 1
            fi
        fi
    }

    pull_model "$LLM_A"
    echo ""
    pull_model "$LLM_B"
}

# Create anna user (requires sudo)
create_user() {
    log_info "Creating anna user..."

    if id "anna" &>/dev/null; then
        log_success "User 'anna' already exists"
    else
        $SUDO useradd -r -s /bin/false -d /var/lib/anna anna
        log_success "Created user 'anna'"
    fi
}

# Create directories (requires sudo)
create_directories() {
    log_info "Creating directories..."

    $SUDO mkdir -p /var/lib/anna
    $SUDO mkdir -p /var/log/anna
    $SUDO mkdir -p /run/anna
    $SUDO mkdir -p /etc/anna
    $SUDO mkdir -p /usr/share/anna/probes

    $SUDO chown anna:anna /var/lib/anna
    $SUDO chown anna:anna /var/log/anna
    $SUDO chown anna:anna /run/anna

    log_success "Created directories"
}

# Install systemd service (requires sudo)
install_service() {
    log_info "Installing systemd service..."

    $SUDO tee /etc/systemd/system/annad.service > /dev/null << 'EOF'
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
Restart=on-failure
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

# Allow reading system info
ReadOnlyPaths=/proc /sys
ReadWritePaths=/var/lib/anna /var/log/anna /run/anna

[Install]
WantedBy=multi-user.target
EOF

    $SUDO systemctl daemon-reload
    log_success "Installed systemd service"
}

# Write configuration (requires sudo)
write_config() {
    log_info "Writing configuration..."

    $SUDO tee /etc/anna/config.toml > /dev/null << EOF
# Anna v${VERSION} Configuration

[general]
version = "${VERSION}"

[models]
orchestrator = "${LLM_A}"
expert = "${LLM_B}"

[daemon]
listen_addr = "127.0.0.1:7865"
probes_dir = "/usr/share/anna/probes"

[ollama]
url = "http://127.0.0.1:11434"
EOF

    log_success "Configuration written to /etc/anna/config.toml"
}

# Install probes from GitHub (requires sudo)
install_probes() {
    log_info "Installing probes..."

    local base_url="https://raw.githubusercontent.com/${GITHUB_REPO}/v${VERSION}/probes"

    # Download to temp then move with sudo
    local probe_tmp=$(mktemp -d)

    download_file "${base_url}/cpu.info.json" "${probe_tmp}/cpu.info.json"
    download_file "${base_url}/mem.info.json" "${probe_tmp}/mem.info.json"
    download_file "${base_url}/disk.lsblk.json" "${probe_tmp}/disk.lsblk.json"

    $SUDO mv "${probe_tmp}"/*.json /usr/share/anna/probes/
    rm -rf "${probe_tmp}"

    log_success "Installed probes"
}

# Run self-test
run_self_test() {
    log_info "Running self-test..."

    # Test 1: Check daemon binary
    if [[ -x "${INSTALL_DIR}/annad" ]]; then
        log_success "Test 1: annad binary OK"
    else
        log_error "Test 1: annad binary FAILED"
        return 1
    fi

    # Test 2: Check annactl binary
    if [[ -x "${INSTALL_DIR}/annactl" ]]; then
        local version
        version=$(${INSTALL_DIR}/annactl --version 2>&1)
        log_success "Test 2: annactl binary OK (${version})"
    else
        log_error "Test 2: annactl binary FAILED"
        return 1
    fi

    # Test 3: Check config (use sudo to test file exists)
    if $SUDO test -f "/etc/anna/config.toml"; then
        log_success "Test 3: Configuration OK"
    else
        log_error "Test 3: Configuration FAILED"
        return 1
    fi

    # Test 4: Check probes (use sudo to test file exists)
    if $SUDO test -f "/usr/share/anna/probes/cpu.info.json"; then
        log_success "Test 4: Probes OK"
    else
        log_error "Test 4: Probes FAILED"
        return 1
    fi
}

# Cleanup
cleanup() {
    if [[ -n "$TMP_DIR" ]] && [[ -d "$TMP_DIR" ]]; then
        rm -rf "$TMP_DIR"
    fi
}

# Main installation
main() {
    trap cleanup EXIT

    print_banner

    # Phase 1: Detection (no sudo needed)
    detect_arch
    detect_hardware
    select_models
    check_dependencies

    # Phase 2: Download (no sudo needed)
    download_binaries

    # Phase 3: Request sudo for installation
    check_sudo
    request_sudo

    # Phase 4: Install (requires sudo)
    create_user
    create_directories
    install_binaries
    install_service
    write_config
    install_probes

    # Phase 5: Download models (no sudo needed)
    download_models

    # Phase 6: Verify
    run_self_test

    echo ""
    log_success "${BOLD}Installation complete!${NC}"
    echo ""
    printf "   Start the daemon:\n"
    printf "   ${CYAN}sudo systemctl start annad${NC}\n"
    echo ""
    printf "   Enable at boot:\n"
    printf "   ${CYAN}sudo systemctl enable annad${NC}\n"
    echo ""
    printf "   Check status:\n"
    printf "   ${CYAN}annactl status${NC}\n"
    echo ""
    printf "   Update Anna:\n"
    printf "   ${CYAN}annactl update${NC}\n"
    echo ""
    printf "${MAGENTA}${ICON_ROCKET}${NC}  ${BOLD}Welcome to Anna!${NC}\n"
    echo ""
}

main "$@"
