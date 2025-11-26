#!/usr/bin/env bash
# Anna v0.0.1 - Installation Script
# Downloads from GitHub, detects hardware, selects models, installs components

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Icons
ICON_CHECK="✓"
ICON_CROSS="✗"
ICON_INFO="ℹ"
ICON_WARN="⚠"
ICON_ROCKET="🚀"
ICON_DOWN="⬇"

VERSION="0.0.1"
GITHUB_REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"

print_banner() {
    echo -e "\n${MAGENTA}${BOLD}  Anna v${VERSION}${NC}"
    echo -e "   Your intelligent Linux assistant\n"
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

    # CPU cores
    CPU_CORES=$(nproc)
    echo "   CPU: ${CPU_CORES} cores"

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
        LLM_A="mistral-nemo-instruct"
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

    echo "   Orchestrator (LLM-A): ${CYAN}${LLM_A}${NC}"
    echo "   Expert (LLM-B): ${CYAN}${LLM_B}${NC}"
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

    echo -e "   ${ICON_DOWN}  Downloading $(basename "$output")..."

    if [[ "$DOWNLOADER" == "curl" ]]; then
        curl -sL "$url" -o "$output"
    else
        wget -q "$url" -O "$output"
    fi
}

# Download and install binaries from GitHub
install_binaries() {
    log_info "Downloading binaries from GitHub..."

    local base_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}"

    # Create temp directory
    local tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download annad
    download_file "${base_url}/annad-${VERSION}-${ARCH_NAME}" "${tmp_dir}/annad"

    # Download annactl
    download_file "${base_url}/annactl-${VERSION}-${ARCH_NAME}" "${tmp_dir}/annactl"

    # Download checksums
    download_file "${base_url}/SHA256SUMS" "${tmp_dir}/SHA256SUMS"

    # Verify checksums
    log_info "Verifying checksums..."
    cd "$tmp_dir"
    if grep -q "$(sha256sum annad | awk '{print $1}')" SHA256SUMS && \
       grep -q "$(sha256sum annactl | awk '{print $1}')" SHA256SUMS; then
        log_success "Checksums verified"
    else
        log_error "Checksum verification failed!"
        exit 1
    fi

    # Install binaries
    log_info "Installing binaries..."
    mv annad "${INSTALL_DIR}/annad"
    mv annactl "${INSTALL_DIR}/annactl"
    chmod 755 "${INSTALL_DIR}/annad"
    chmod 755 "${INSTALL_DIR}/annactl"

    log_success "Installed annad and annactl to ${INSTALL_DIR}"
}

# Download LLM models
download_models() {
    if [[ "$OLLAMA_MISSING" == "true" ]]; then
        log_warn "Skipping model download (Ollama not installed)"
        return
    fi

    log_info "Downloading LLM models..."

    echo "   Pulling ${LLM_A}..."
    if ollama pull "$LLM_A" 2>/dev/null; then
        log_success "Downloaded ${LLM_A}"
    else
        log_warn "Failed to download ${LLM_A}"
    fi

    echo "   Pulling ${LLM_B}..."
    if ollama pull "$LLM_B" 2>/dev/null; then
        log_success "Downloaded ${LLM_B}"
    else
        log_warn "Failed to download ${LLM_B}"
    fi
}

# Create anna user
create_user() {
    log_info "Creating anna user..."

    if id "anna" &>/dev/null; then
        log_success "User 'anna' already exists"
    else
        useradd -r -s /bin/false -d /var/lib/anna anna
        log_success "Created user 'anna'"
    fi
}

# Create directories
create_directories() {
    log_info "Creating directories..."

    mkdir -p /var/lib/anna
    mkdir -p /var/log/anna
    mkdir -p /run/anna
    mkdir -p /etc/anna
    mkdir -p /usr/share/anna/probes

    chown anna:anna /var/lib/anna
    chown anna:anna /var/log/anna
    chown anna:anna /run/anna

    log_success "Created directories"
}

# Install systemd service
install_service() {
    log_info "Installing systemd service..."

    cat > /etc/systemd/system/annad.service << 'EOF'
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

    systemctl daemon-reload
    log_success "Installed systemd service"
}

# Write configuration
write_config() {
    log_info "Writing configuration..."

    cat > /etc/anna/config.toml << EOF
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

# Install probes from GitHub
install_probes() {
    log_info "Installing probes..."

    local base_url="https://raw.githubusercontent.com/${GITHUB_REPO}/v${VERSION}/probes"

    download_file "${base_url}/cpu.info.json" "/usr/share/anna/probes/cpu.info.json"
    download_file "${base_url}/mem.info.json" "/usr/share/anna/probes/mem.info.json"
    download_file "${base_url}/disk.lsblk.json" "/usr/share/anna/probes/disk.lsblk.json"

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
        local version=$(${INSTALL_DIR}/annactl --version 2>&1)
        log_success "Test 2: annactl binary OK (${version})"
    else
        log_error "Test 2: annactl binary FAILED"
        return 1
    fi

    # Test 3: Check config
    if [[ -f "/etc/anna/config.toml" ]]; then
        log_success "Test 3: Configuration OK"
    else
        log_error "Test 3: Configuration FAILED"
        return 1
    fi

    # Test 4: Check probes
    if [[ -f "/usr/share/anna/probes/cpu.info.json" ]]; then
        log_success "Test 4: Probes OK"
    else
        log_error "Test 4: Probes FAILED"
        return 1
    fi
}

# Main installation
main() {
    print_banner

    check_root
    detect_arch
    detect_hardware
    select_models
    check_dependencies
    create_user
    create_directories
    install_binaries
    install_service
    write_config
    install_probes
    download_models
    run_self_test

    echo ""
    log_success "${BOLD}Installation complete!${NC}"
    echo ""
    echo "   Start the daemon:"
    echo "   ${CYAN}sudo systemctl start annad${NC}"
    echo ""
    echo "   Enable at boot:"
    echo "   ${CYAN}sudo systemctl enable annad${NC}"
    echo ""
    echo "   Check status:"
    echo "   ${CYAN}annactl status${NC}"
    echo ""
    echo -e "${MAGENTA}${ICON_ROCKET}${NC}  ${BOLD}Welcome to Anna!${NC}"
    echo ""
}

main "$@"
