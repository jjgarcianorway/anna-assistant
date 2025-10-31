#!/bin/bash
# Anna v0.10.1 Pure Observer Installer
# 6 phases: Detect, Prepare, Install, Configure, Verify, Celebrate

set -e

# Color codes for pretty output
C_RESET="\e[0m"
C_BLUE="\e[34m"
C_GREEN="\e[32m"
C_YELLOW="\e[33m"
C_RED="\e[31m"

# Unicode box drawing
BOX_TL="╭"
BOX_TR="╮"
BOX_BL="╰"
BOX_BR="╯"
BOX_H="─"

phase_header() {
    local phase=$1
    local title=$2
    local width=50

    echo -e "\n${C_BLUE}${BOX_TL}${BOX_H} Phase ${phase}/6: ${title} $(printf "${BOX_H}%.0s" $(seq 1 $((width - ${#title} - 13))))${BOX_TR}${C_RESET}"
}

phase_footer() {
    echo -e "${C_BLUE}${BOX_BL}$(printf "${BOX_H}%.0s" {1..50})${BOX_BR}${C_RESET}\n"
}

info() {
    echo -e "${C_BLUE}  ⏳${C_RESET} $1"
}

success() {
    echo -e "${C_GREEN}  ✓${C_RESET} $1"
}

warning() {
    echo -e "${C_YELLOW}  ⚠${C_RESET} $1"
}

error() {
    echo -e "${C_RED}  ✗${C_RESET} $1"
}

# Check root
if [[ $EUID -ne 0 ]]; then
    error "This installer must be run as root"
    echo "  Try: sudo $0"
    exit 1
fi

# Phase 1: Detection
phase_header 1 "System Detection"

if ! command -v systemctl &>/dev/null; then
    error "systemd not found (required)"
    exit 1
fi
success "systemd found"

if ! grep -q "Arch Linux" /etc/os-release 2>/dev/null; then
    warning "Not Arch Linux (best effort support)"
else
    success "Arch Linux detected"
fi

if command -v cargo &>/dev/null; then
    success "cargo found"
else
    error "cargo not found - install rust toolchain"
    exit 1
fi

phase_footer

# Phase 2: Preparation
phase_header 2 "Build Preparation"

info "Building release binaries..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

if [[ ! -f target/release/annad ]] || [[ ! -f target/release/annactl ]]; then
    error "Build failed"
    exit 1
fi

success "Binaries built successfully"
phase_footer

# Phase 3: Installation
phase_header 3 "System Installation"

# Create anna user
if ! id anna &>/dev/null; then
    info "Creating anna system user..."
    useradd -r -s /bin/false -d /var/lib/anna anna
    success "User created"
else
    success "User exists"
fi

# Create directories
info "Creating directories..."
mkdir -p /var/lib/anna
mkdir -p /var/log/anna
mkdir -p /usr/local/bin
chown anna:anna /var/lib/anna /var/log/anna
chmod 0750 /var/lib/anna /var/log/anna
success "Directories created"

# Install binaries
info "Installing binaries..."
install -m 0755 target/release/annad /usr/local/bin/
install -m 0755 target/release/annactl /usr/local/bin/
success "Binaries installed"

phase_footer

# Phase 4: Configuration
phase_header 4 "System Configuration"

# Install systemd unit
info "Installing systemd unit..."
install -m 0644 etc/systemd/annad.service /etc/systemd/system/

systemctl daemon-reload
success "Systemd unit installed"

# Enable and start
info "Enabling and starting daemon..."
systemctl enable annad
systemctl start annad
success "Daemon started"

phase_footer

# Phase 5: Verification
phase_header 5 "Health Verification"

info "Waiting for first telemetry sample (35s)..."
sleep 35

if systemctl is-active --quiet annad; then
    success "Daemon is running"
else
    error "Daemon failed to start"
    journalctl -u annad -n 20 --no-pager
    exit 1
fi

# Test RPC
if timeout 5 /usr/local/bin/annactl version &>/dev/null; then
    success "RPC communication OK"
else
    warning "RPC test failed (may need time to stabilize)"
fi

phase_footer

# Phase 6: Celebrate!
phase_header 6 "Installation Complete"

cat <<'BANNER'
  ╭────────────────────────────────────────────╮
  │                                            │
  │     Anna is ready to observe! 🤖           │
  │                                            │
  ╰────────────────────────────────────────────╯

  Next steps:

    annactl status      # View daemon status
    annactl sensors     # CPU, memory, temps
    annactl net         # Network interfaces
    annactl disk        # Disk usage
    annactl top         # Top processes
    annactl radar       # Persona classification

  Logs:
    journalctl -u annad -f

  Database:
    /var/lib/anna/telemetry.db

BANNER

phase_footer

echo -e "${C_GREEN}Installation successful!${C_RESET}\n"
