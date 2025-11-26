#!/usr/bin/env bash
# Anna v0.0.1 - Installation Script
# Detects hardware, selects models, installs components

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Icons
ICON_CHECK="✓"
ICON_CROSS="✗"
ICON_INFO="ℹ"
ICON_WARN="⚠"
ICON_ROCKET="🚀"

VERSION="0.0.1"

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

# Install binaries
install_binaries() {
    log_info "Installing binaries..."

    INSTALL_DIR="/usr/local/bin"

    # Check if binaries exist in current directory
    if [[ -f "target/release/annad" ]] && [[ -f "target/release/annactl" ]]; then
        cp target/release/annad "${INSTALL_DIR}/annad"
        cp target/release/annactl "${INSTALL_DIR}/annactl"
        chmod 755 "${INSTALL_DIR}/annad"
        chmod 755 "${INSTALL_DIR}/annactl"
        log_success "Installed annad and annactl to ${INSTALL_DIR}"
    else
        log_warn "Binaries not found. Build with: cargo build --release"
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
After=network.target

[Service]
Type=simple
User=anna
Group=anna
ExecStart=/usr/local/bin/annad
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

# Install probes
install_probes() {
    log_info "Installing probes..."

    mkdir -p /usr/share/anna/probes

    if [[ -d "probes" ]]; then
        cp probes/*.json /usr/share/anna/probes/
        log_success "Installed probes"
    else
        log_warn "Probes directory not found"
    fi
}

# Run self-test
run_self_test() {
    log_info "Running self-test..."

    # Test 1: Check daemon binary
    if [[ -x "/usr/local/bin/annad" ]]; then
        log_success "Test 1: annad binary OK"
    else
        log_error "Test 1: annad binary FAILED"
        return 1
    fi

    # Test 2: Check annactl binary
    if [[ -x "/usr/local/bin/annactl" ]]; then
        log_success "Test 2: annactl binary OK"
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
}

# Main installation
main() {
    print_banner

    check_root
    detect_hardware
    select_models
    check_dependencies
    download_models
    create_user
    create_directories
    install_binaries
    install_service
    write_config
    install_probes
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
