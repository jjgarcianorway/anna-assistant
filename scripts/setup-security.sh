#!/bin/bash
# Anna Assistant - Security Setup Script
# Phase 0.4: Configure secure directories and system group
#
# This script is called by the installer to set up:
# - anna system group for socket access
# - Secure directory structure with correct permissions
#
# Citation: [archwiki:Users_and_groups]
# Citation: [archwiki:Security#Mandatory_access_control]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "This script must be run as root"
fi

info "Setting up Anna security configuration..."

# Phase 0.4: Create anna system group
if ! getent group anna >/dev/null 2>&1; then
    info "Creating anna system group..."
    groupadd --system anna
    info "✓ Created anna group"
else
    info "✓ anna group already exists"
fi

# Phase 0.4: Create secure directory structure
# /etc/anna - Configuration (root:root 0700)
if [ ! -d /etc/anna ]; then
    info "Creating /etc/anna..."
    mkdir -p /etc/anna
fi
chown root:root /etc/anna
chmod 700 /etc/anna
info "✓ /etc/anna configured (root:root 0700)"

# /var/log/anna - Logs (root:root 0700)
if [ ! -d /var/log/anna ]; then
    info "Creating /var/log/anna..."
    mkdir -p /var/log/anna
fi
chown root:root /var/log/anna
chmod 700 /var/log/anna
info "✓ /var/log/anna configured (root:root 0700)"

# /var/lib/anna - State directory (root:root 0700)
if [ ! -d /var/lib/anna ]; then
    info "Creating /var/lib/anna..."
    mkdir -p /var/lib/anna
fi
chown root:root /var/lib/anna
chmod 700 /var/lib/anna
info "✓ /var/lib/anna configured (root:root 0700)"

# /usr/local/lib/anna - Static assets (root:root 0755)
if [ ! -d /usr/local/lib/anna ]; then
    info "Creating /usr/local/lib/anna..."
    mkdir -p /usr/local/lib/anna
fi
chown root:root /usr/local/lib/anna
chmod 755 /usr/local/lib/anna
info "✓ /usr/local/lib/anna configured (root:root 0755)"

# /run/anna will be created by systemd with RuntimeDirectory
# but we ensure it doesn't exist with wrong permissions
if [ -d /run/anna ]; then
    if [ -S /run/anna/anna.sock ]; then
        warn "Removing existing socket at /run/anna/anna.sock"
        rm -f /run/anna/anna.sock
    fi
fi

info "✓ Security setup complete"
info ""
info "Directory structure:"
info "  /etc/anna           → Configuration (600)"
info "  /var/log/anna       → Logs (700)"
info "  /var/lib/anna       → State (700)"
info "  /usr/local/lib/anna → Static assets (755)"
info "  /run/anna           → Runtime socket (750, managed by systemd)"
info ""
info "Socket permissions:"
info "  /run/anna/anna.sock → root:anna 0660"
echo ""
