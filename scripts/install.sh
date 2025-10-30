#!/usr/bin/env bash
set -Eeuo pipefail

# Anna Assistant Installer
# Runs unprivileged; self-escalates only when needed
# Version: 0.9.6-alpha.6 (Hotfix - Working System)

# ═══════════════════════════════════════════════════════════
#  PHASE 1: DETECTION
# ═══════════════════════════════════════════════════════════

echo "╭─────────────────────────────────────────╮"
echo "│  Anna Assistant Installer               │"
echo "│  Version: 0.9.6-alpha.6                 │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Terminal capabilities
TTY_MODE=false
if [ -t 1 ]; then TTY_MODE=true; fi

# Color support
if command -v tput &>/dev/null && [ "$TTY_MODE" = true ]; then
    COLOR_GREEN=$(tput setaf 2)
    COLOR_BLUE=$(tput setaf 4)
    COLOR_YELLOW=$(tput setaf 3)
    COLOR_RED=$(tput setaf 1)
    COLOR_RESET=$(tput sgr0)
    SYM_OK="✓"
    SYM_FAIL="✗"
    SYM_WARN="⚠"
    SYM_INFO="→"
else
    COLOR_GREEN=""
    COLOR_BLUE=""
    COLOR_YELLOW=""
    COLOR_RED=""
    COLOR_RESET=""
    SYM_OK="[OK]"
    SYM_FAIL="[FAIL]"
    SYM_WARN="[WARN]"
    SYM_INFO="-->"
fi

# Check we're in project root
if [[ ! -f Cargo.toml ]]; then
    echo "${COLOR_RED}${SYM_FAIL} Please run from anna-assistant project root${COLOR_RESET}"
    exit 1
fi

# Read version from Cargo.toml
if command -v cargo &>/dev/null; then
    VERSION=$(cargo metadata --no-deps --format-version 1 2>/dev/null | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)
    if [ -z "$VERSION" ]; then
        VERSION="0.9.6-alpha.6"
    fi
else
    VERSION="0.9.6-alpha.6"
fi

echo "${SYM_INFO} Version: ${VERSION}"
echo ""

# Detect existing installation
OLD_VERSION=""
if [ -f /etc/anna/version ]; then
    OLD_VERSION=$(cat /etc/anna/version 2>/dev/null || echo "")
fi

MODE="install"
if [ -n "$OLD_VERSION" ]; then
    MODE="upgrade"
    echo "${SYM_INFO} Detected existing installation: ${OLD_VERSION}"
    echo "${SYM_INFO} This will upgrade Anna to ${VERSION}"
    echo ""
    read -p "Proceed with upgrade? [y/N] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        exit 0
    fi
    echo ""
fi

# Check dependencies
echo "${SYM_INFO} Checking dependencies..."
MISSING_DEPS=()

if ! command -v systemctl &>/dev/null; then
    echo "${COLOR_RED}${SYM_FAIL} systemd is required${COLOR_RESET}"
    exit 1
fi

if ! command -v cargo &>/dev/null; then
    MISSING_DEPS+=("cargo (rust)")
fi

if ! command -v sqlite3 &>/dev/null; then
    echo "${COLOR_YELLOW}${SYM_WARN} sqlite3 CLI not found (optional)${COLOR_RESET}"
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "${COLOR_RED}${SYM_FAIL} Missing required dependencies:${COLOR_RESET}"
    for dep in "${MISSING_DEPS[@]}"; do
        echo "  - $dep"
    done
    echo ""
    echo "On Arch Linux: sudo pacman -S rust sqlite"
    exit 1
fi

echo "${COLOR_GREEN}${SYM_OK} All dependencies found${COLOR_RESET}"
echo ""

# ═══════════════════════════════════════════════════════════
#  PHASE 2: PREPARATION
# ═══════════════════════════════════════════════════════════

echo "╭─────────────────────────────────────────╮"
echo "│  Phase 2: Preparation                   │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Backup existing installation if upgrading
if [ "$MODE" = "upgrade" ]; then
    BACKUP_DIR="/var/lib/anna/backups/backup-$(date +%Y%m%d-%H%M%S)"
    echo "${SYM_INFO} Creating backup at ${BACKUP_DIR}"

    # Need sudo for this
    if ! sudo mkdir -p "$BACKUP_DIR" 2>/dev/null; then
        echo "${COLOR_YELLOW}${SYM_WARN} Could not create backup directory${COLOR_RESET}"
    else
        sudo cp -r /etc/anna "$BACKUP_DIR/" 2>/dev/null || true
        sudo cp /usr/local/bin/annad "$BACKUP_DIR/" 2>/dev/null || true
        sudo cp /usr/local/bin/annactl "$BACKUP_DIR/" 2>/dev/null || true
        echo "${COLOR_GREEN}${SYM_OK} Backup created${COLOR_RESET}"
    fi
    echo ""
fi

# Build binaries
echo "${SYM_INFO} Building binaries (this may take a minute)..."
if cargo build --release --quiet 2>&1; then
    echo "${COLOR_GREEN}${SYM_OK} Build complete${COLOR_RESET}"
else
    echo "${COLOR_RED}${SYM_FAIL} Build failed${COLOR_RESET}"
    exit 1
fi
echo ""

# ═══════════════════════════════════════════════════════════
#  PHASE 3: INSTALLATION
# ═══════════════════════════════════════════════════════════

echo "╭─────────────────────────────────────────╮"
echo "│  Phase 3: Installation                  │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Stop daemon if running
if systemctl is-active --quiet annad 2>/dev/null; then
    echo "${SYM_INFO} Stopping existing daemon..."
    sudo systemctl stop annad
fi

# Install binaries
echo "${SYM_INFO} Installing binaries to /usr/local/bin..."
echo "  (requires elevated privileges)"
echo ""

if sudo install -m 755 target/release/annad /usr/local/bin/ && \
   sudo install -m 755 target/release/annactl /usr/local/bin/; then
    echo "${COLOR_GREEN}${SYM_OK} Binaries installed${COLOR_RESET}"
else
    echo "${COLOR_RED}${SYM_FAIL} Binary installation failed${COLOR_RESET}"
    exit 1
fi

# Create runtime directories
echo "${SYM_INFO} Setting up runtime directories..."

sudo mkdir -p /run/anna || true
sudo chmod 0755 /run/anna
sudo mkdir -p /var/lib/anna
sudo chmod 0750 /var/lib/anna
sudo mkdir -p /var/log/anna
sudo chmod 0750 /var/log/anna
sudo mkdir -p /etc/anna/policies.d
sudo mkdir -p /etc/anna/personas.d
sudo chmod 0755 /etc/anna

echo "${COLOR_GREEN}${SYM_OK} Directories created${COLOR_RESET}"

# Create anna group if it doesn't exist
if ! getent group anna &>/dev/null; then
    echo "${SYM_INFO} Creating 'anna' group..."
    sudo groupadd --system anna
    echo "${COLOR_GREEN}${SYM_OK} Group created${COLOR_RESET}"
fi

# Add current user to anna group
if ! groups | grep -q anna; then
    echo "${SYM_INFO} Adding ${USER} to 'anna' group..."
    sudo usermod -aG anna "$USER"
    echo "${COLOR_GREEN}${SYM_OK} User added to group${COLOR_RESET}"
    echo "${COLOR_YELLOW}${SYM_WARN} You may need to log out and back in for group changes to take effect${COLOR_RESET}"
fi

# Set ownership
sudo chown root:anna /run/anna
sudo chown root:anna /var/lib/anna
sudo chown root:anna /var/log/anna

# Write version file
echo "$VERSION" | sudo tee /etc/anna/version >/dev/null
echo "${COLOR_GREEN}${SYM_OK} Version file written${COLOR_RESET}"

# Install default config if not present
if [ ! -f /etc/anna/config.toml ]; then
    echo "${SYM_INFO} Installing default configuration..."
    cat <<'EOF_CONFIG' | sudo tee /etc/anna/config.toml >/dev/null
# Anna Assistant Configuration
# Managed by Anna - use 'annactl config' to modify
# Manual edits are ignored when Anna is managing config

[autonomy]
level = "low"

[ui]
emojis = false
color = true
verbose = false

[telemetry]
enabled = false
collection_interval_sec = 60
retention_days = 7

[persona]
active = "default"
EOF_CONFIG
    echo "${COLOR_GREEN}${SYM_OK} Default config installed${COLOR_RESET}"
fi

# Install default policies if directory is empty
if [ -z "$(ls -A /etc/anna/policies.d 2>/dev/null)" ]; then
    echo "${SYM_INFO} Installing default policies..."

    cat <<'EOF_POLICY' | sudo tee /etc/anna/policies.d/00-bootstrap.yaml >/dev/null
# Bootstrap policies - loaded at daemon startup
# Managed by Anna - use 'annactl policy' to modify

- when: "always"
  then: "log"
  message: "Anna policy engine initialized"
  enabled: true

- when: "telemetry.cpu_usage > 90"
  then: "alert"
  message: "High CPU usage detected"
  enabled: true

- when: "telemetry.mem_usage > 95"
  then: "alert"
  message: "Critical memory usage"
  enabled: true
EOF_POLICY
    echo "${COLOR_GREEN}${SYM_OK} Default policies installed${COLOR_RESET}"
fi

# Install systemd service
echo "${SYM_INFO} Installing systemd service..."

sudo cp etc/systemd/annad.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable annad

echo "${COLOR_GREEN}${SYM_OK} Service installed and enabled${COLOR_RESET}"
echo ""

# ═══════════════════════════════════════════════════════════
#  PHASE 4: VERIFICATION
# ═══════════════════════════════════════════════════════════

echo "╭─────────────────────────────────────────╮"
echo "│  Phase 4: Verification                  │"
echo "╰─────────────────────────────────────────╯"
echo ""

# Start daemon
echo "${SYM_INFO} Starting daemon..."
if sudo systemctl start annad; then
    sleep 2

    if systemctl is-active --quiet annad; then
        echo "${COLOR_GREEN}${SYM_OK} Daemon started successfully${COLOR_RESET}"

        # Check if socket exists
        if [ -S /run/anna/annad.sock ]; then
            echo "${COLOR_GREEN}${SYM_OK} Socket created${COLOR_RESET}"
        else
            echo "${COLOR_YELLOW}${SYM_WARN} Socket not found (daemon may still be initializing)${COLOR_RESET}"
        fi
    else
        echo "${COLOR_RED}${SYM_FAIL} Daemon failed to start${COLOR_RESET}"
        echo ""
        echo "Checking logs:"
        sudo journalctl -u annad -n 20 --no-pager
        echo ""
        echo "${SYM_INFO} Run 'annactl doctor repair' to attempt automatic fixes"
        exit 1
    fi
else
    echo "${COLOR_RED}${SYM_FAIL} Failed to start daemon${COLOR_RESET}"
    exit 1
fi

# Run doctor check (creates telemetry DB if needed)
echo ""
echo "${SYM_INFO} Running post-install health check..."
if /usr/local/bin/annactl doctor check &>/dev/null; then
    echo "${COLOR_GREEN}${SYM_OK} Health check passed${COLOR_RESET}"
else
    echo "${COLOR_YELLOW}${SYM_WARN} Some health checks failed${COLOR_RESET}"
    echo "${SYM_INFO} Running automatic repair..."
    /usr/local/bin/annactl doctor repair || true
fi

echo ""

# ═══════════════════════════════════════════════════════════
#  SUCCESS
# ═══════════════════════════════════════════════════════════

echo "╭─────────────────────────────────────────╮"
echo "│  ${COLOR_GREEN}${SYM_OK} Installation Successful${COLOR_RESET}              │"
echo "│                                         │"
echo "│  Anna is ready to serve!                │"
echo "╰─────────────────────────────────────────╯"
echo ""
echo "Version:    ${VERSION}"
echo "Daemon:     $(systemctl is-active annad)"
echo "Socket:     /run/anna/annad.sock"
echo "Logs:       /var/log/anna/"
echo ""
echo "Next steps:"
echo "  ${SYM_INFO} annactl status         - View system status"
echo "  ${SYM_INFO} annactl profile show   - System health profile"
echo "  ${SYM_INFO} annactl doctor check   - Run diagnostics"
echo "  ${SYM_INFO} journalctl -u annad -f - Follow daemon logs"
echo ""

if [ "$MODE" = "upgrade" ]; then
    echo "Upgraded from ${OLD_VERSION} to ${VERSION}"
    echo "Backup location: ${BACKUP_DIR}"
    echo ""
fi

exit 0
