#!/usr/bin/env bash
# Anna v0.11.0 Comprehensive Diagnostics Script
# Collects system state, permissions, logs, and configuration for troubleshooting
#
# Usage: bash scripts/anna-diagnostics.sh [--output FILE]

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
OUTPUT_FILE=""
if [ "${1:-}" = "--output" ] && [ -n "${2:-}" ]; then
    OUTPUT_FILE="$2"
fi

# Output function
output() {
    if [ -n "$OUTPUT_FILE" ]; then
        echo "$@" | tee -a "$OUTPUT_FILE"
    else
        echo "$@"
    fi
}

# Colored output helpers
info() {
    output -e "${BLUE}→${NC} $1"
}

ok() {
    output -e "${GREEN}✓${NC} $1"
}

warn() {
    output -e "${YELLOW}⚠${NC} $1"
}

fail() {
    output -e "${RED}✗${NC} $1"
}

section() {
    output ""
    output "═══════════════════════════════════════════════════"
    output "$1"
    output "═══════════════════════════════════════════════════"
    output ""
}

# Start diagnostics
if [ -n "$OUTPUT_FILE" ]; then
    echo "Anna Diagnostics - $(date)" > "$OUTPUT_FILE"
fi

section "ANNA v0.11.0 DIAGNOSTICS"
output "Generated: $(date)"
output "Hostname: $(hostname)"
output "User: $USER"
output "PWD: $(pwd)"
output ""

# ============================================================================
# SECTION 1: System Information
# ============================================================================

section "1. SYSTEM INFORMATION"

info "Operating System:"
if [ -f /etc/os-release ]; then
    output "$(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '"')"
else
    output "Unknown (no /etc/os-release)"
fi

info "Kernel:"
output "$(uname -r)"

info "Architecture:"
output "$(uname -m)"

info "Init System:"
if [ -d /run/systemd/system ]; then
    SYSTEMD_VERSION=$(systemctl --version | head -1 | awk '{print $2}')
    ok "systemd $SYSTEMD_VERSION"
else
    fail "systemd not detected (Anna requires systemd)"
fi

# ============================================================================
# SECTION 2: Anna Binaries
# ============================================================================

section "2. ANNA BINARIES"

info "Checking annad binary:"
if [ -f /usr/local/bin/annad ]; then
    ANNAD_SIZE=$(stat -c%s /usr/local/bin/annad 2>/dev/null || stat -f%z /usr/local/bin/annad)
    ANNAD_PERMS=$(stat -c '%a %U:%G' /usr/local/bin/annad 2>/dev/null || stat -f '%Lp %Su:%Sg' /usr/local/bin/annad)
    ok "/usr/local/bin/annad exists (${ANNAD_SIZE} bytes, ${ANNAD_PERMS})"

    if [ -x /usr/local/bin/annad ]; then
        output "   Version: $(/usr/local/bin/annad --version 2>&1 || echo 'Cannot determine version')"
    else
        warn "   Not executable"
    fi
else
    fail "/usr/local/bin/annad not found"
fi

info "Checking annactl binary:"
if [ -f /usr/local/bin/annactl ]; then
    ANNACTL_SIZE=$(stat -c%s /usr/local/bin/annactl 2>/dev/null || stat -f%z /usr/local/bin/annactl)
    ANNACTL_PERMS=$(stat -c '%a %U:%G' /usr/local/bin/annactl 2>/dev/null || stat -f '%Lp %Su:%Sg' /usr/local/bin/annactl)
    ok "/usr/local/bin/annactl exists (${ANNACTL_SIZE} bytes, ${ANNACTL_PERMS})"

    if [ -x /usr/local/bin/annactl ]; then
        output "   Version: $(/usr/local/bin/annactl --version 2>&1 || echo 'Cannot determine version')"
    else
        warn "   Not executable"
    fi
else
    fail "/usr/local/bin/annactl not found"
fi

# ============================================================================
# SECTION 3: User and Group
# ============================================================================

section "3. USER AND GROUP"

info "Checking anna user:"
if id anna &>/dev/null; then
    ANNA_UID=$(id -u anna)
    ANNA_GID=$(id -g anna)
    ANNA_GROUPS=$(id -Gn anna | tr ' ' ',')
    ok "User 'anna' exists (uid=$ANNA_UID, gid=$ANNA_GID)"
    output "   Groups: $ANNA_GROUPS"
    output "   Shell: $(getent passwd anna | cut -d: -f7)"
    output "   Home: $(getent passwd anna | cut -d: -f6)"
else
    fail "User 'anna' does not exist"
fi

info "Checking anna group:"
if getent group anna &>/dev/null; then
    ANNA_GROUP_GID=$(getent group anna | cut -d: -f3)
    ANNA_GROUP_MEMBERS=$(getent group anna | cut -d: -f4)
    ok "Group 'anna' exists (gid=$ANNA_GROUP_GID)"
    if [ -n "$ANNA_GROUP_MEMBERS" ]; then
        output "   Members: $ANNA_GROUP_MEMBERS"
    else
        output "   Members: (none)"
    fi
else
    fail "Group 'anna' does not exist"
fi

info "Checking current user's group membership:"
if groups | grep -q anna; then
    ok "Current user ($USER) is in anna group"
else
    warn "Current user ($USER) is NOT in anna group"
    output "   Run: sudo usermod -aG anna $USER && newgrp anna"
fi

# ============================================================================
# SECTION 4: Directories and Permissions
# ============================================================================

section "4. DIRECTORIES AND PERMISSIONS"

DIRS=(
    "/var/lib/anna"
    "/var/log/anna"
    "/run/anna"
    "/etc/anna"
    "/usr/lib/anna"
    "/etc/anna/policies.d"
    "/etc/anna/personas.d"
)

for dir in "${DIRS[@]}"; do
    info "Checking $dir:"
    if [ -d "$dir" ]; then
        PERMS=$(stat -c '%a %U:%G' "$dir" 2>/dev/null || stat -f '%Lp %Su:%Sg' "$dir")
        output "   Exists: $PERMS"

        # Check expected ownership
        if [[ "$dir" == "/var/lib/anna" ]] || [[ "$dir" == "/var/log/anna" ]] || [[ "$dir" == "/run/anna" ]]; then
            OWNER=$(stat -c '%U' "$dir" 2>/dev/null || stat -f '%Su' "$dir")
            GROUP=$(stat -c '%G' "$dir" 2>/dev/null || stat -f '%Sg' "$dir")
            MODE=$(stat -c '%a' "$dir" 2>/dev/null || stat -f '%Lp' "$dir")

            if [ "$OWNER" = "anna" ] && [ "$GROUP" = "anna" ] && [ "$MODE" = "750" ]; then
                ok "   Ownership correct (anna:anna 0750)"
            else
                warn "   Ownership incorrect (expected anna:anna 0750, got $OWNER:$GROUP $MODE)"
            fi
        fi

        # Check if writable by anna
        if [[ "$dir" == "/var/lib/anna" ]] || [[ "$dir" == "/var/log/anna" ]]; then
            if sudo -u anna test -w "$dir" 2>/dev/null; then
                ok "   Writable by anna user"
            else
                fail "   NOT writable by anna user"
            fi
        fi
    else
        fail "Does not exist"
    fi
done

# ============================================================================
# SECTION 5: Configuration Files
# ============================================================================

section "5. CONFIGURATION FILES"

CONFIG_FILES=(
    "/etc/anna/config.toml"
    "/etc/anna/policy.toml"
    "/etc/anna/policies.d/00-bootstrap.yaml"
    "/usr/lib/anna/CAPABILITIES.toml"
)

for file in "${CONFIG_FILES[@]}"; do
    info "Checking $file:"
    if [ -f "$file" ]; then
        SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
        PERMS=$(stat -c '%a %U:%G' "$file" 2>/dev/null || stat -f '%Lp %Su:%Sg' "$file")
        ok "Exists (${SIZE} bytes, ${PERMS})"

        if [ "$file" = "/usr/lib/anna/CAPABILITIES.toml" ]; then
            # Check version
            if grep -q 'version.*=.*"0.11' "$file"; then
                output "   Version: $(grep 'version.*=' "$file" | head -1 | cut -d'"' -f2)"
            else
                warn "   Version string not found or outdated"
            fi
        fi
    else
        warn "Does not exist"
    fi
done

# ============================================================================
# SECTION 6: Systemd Service
# ============================================================================

section "6. SYSTEMD SERVICE"

info "Checking systemd unit file:"
if [ -f /etc/systemd/system/annad.service ]; then
    ok "/etc/systemd/system/annad.service exists"

    # Check key directives
    if grep -q "User=anna" /etc/systemd/system/annad.service; then
        ok "   User=anna present"
    else
        warn "   User=anna missing"
    fi

    if grep -q "StateDirectory=anna" /etc/systemd/system/annad.service; then
        ok "   StateDirectory=anna present"
    else
        warn "   StateDirectory=anna missing"
    fi

    if grep -q "LogsDirectory=anna" /etc/systemd/system/annad.service; then
        ok "   LogsDirectory=anna present"
    else
        warn "   LogsDirectory=anna missing"
    fi

    if grep -q "RuntimeDirectory=anna" /etc/systemd/system/annad.service; then
        ok "   RuntimeDirectory=anna present"
    else
        warn "   RuntimeDirectory=anna missing"
    fi
else
    fail "/etc/systemd/system/annad.service not found"
fi

info "Checking service status:"
if systemctl is-enabled --quiet annad 2>/dev/null; then
    ok "Service is enabled"
else
    warn "Service is NOT enabled"
fi

if systemctl is-active --quiet annad 2>/dev/null; then
    ok "Service is active (running)"

    # Get PID and resource usage
    PID=$(systemctl show -p MainPID --value annad 2>/dev/null || echo "0")
    if [ "$PID" != "0" ] && [ -n "$PID" ]; then
        output "   PID: $PID"

        # Memory usage
        if [ -f /proc/$PID/status ]; then
            RSS_KB=$(grep VmRSS /proc/$PID/status | awk '{print $2}')
            RSS_MB=$((RSS_KB / 1024))
            output "   Memory: ${RSS_MB}MB"
        fi

        # Uptime
        START_TIME=$(systemctl show -p ActiveEnterTimestamp --value annad 2>/dev/null || echo "unknown")
        output "   Started: $START_TIME"
    fi
else
    fail "Service is NOT active"

    # Show last exit status
    EXIT_STATUS=$(systemctl show -p ExecMainStatus --value annad 2>/dev/null || echo "unknown")
    if [ "$EXIT_STATUS" != "0" ] && [ "$EXIT_STATUS" != "unknown" ]; then
        output "   Last exit status: $EXIT_STATUS"
    fi
fi

# ============================================================================
# SECTION 7: RPC Socket
# ============================================================================

section "7. RPC SOCKET"

info "Checking /run/anna/annad.sock:"
if [ -S /run/anna/annad.sock ]; then
    PERMS=$(stat -c '%a %U:%G' /run/anna/annad.sock 2>/dev/null || stat -f '%Lp %Su:%Sg' /run/anna/annad.sock)
    ok "Socket exists ($PERMS)"

    # Test connectivity
    if timeout 2 annactl status &>/dev/null; then
        ok "   annactl can connect"
    else
        fail "   annactl cannot connect"
    fi
else
    fail "Socket does not exist"

    if systemctl is-active --quiet annad; then
        warn "   Daemon is running but socket not created"
        output "   This may indicate a startup failure"
    fi
fi

# ============================================================================
# SECTION 8: Database
# ============================================================================

section "8. DATABASE"

info "Checking /var/lib/anna/telemetry.db:"
if [ -f /var/lib/anna/telemetry.db ]; then
    SIZE=$(stat -c%s /var/lib/anna/telemetry.db 2>/dev/null || stat -f%z /var/lib/anna/telemetry.db)
    PERMS=$(stat -c '%a %U:%G' /var/lib/anna/telemetry.db 2>/dev/null || stat -f '%Lp %Su:%Sg' /var/lib/anna/telemetry.db)
    ok "Database exists (${SIZE} bytes, ${PERMS})"

    # Check if writable
    OWNER=$(stat -c '%U' /var/lib/anna/telemetry.db 2>/dev/null || stat -f '%Su' /var/lib/anna/telemetry.db)
    if [ "$OWNER" = "anna" ]; then
        ok "   Owned by anna user"
    else
        warn "   NOT owned by anna (owner: $OWNER)"
    fi
else
    warn "Database does not exist (will be created on first run)"
fi

info "Testing write access to /var/lib/anna:"
TEST_FILE="/var/lib/anna/.diagnostics_test"
if sudo -u anna touch "$TEST_FILE" 2>/dev/null; then
    sudo -u anna rm -f "$TEST_FILE"
    ok "Write access OK"
else
    fail "Write access DENIED"
fi

# ============================================================================
# SECTION 9: Recent Logs
# ============================================================================

section "9. RECENT LOGS (last 30 lines)"

if journalctl -u annad -n 30 --no-pager &>/dev/null; then
    journalctl -u annad -n 30 --no-pager 2>&1 | while IFS= read -r line; do
        output "$line"
    done
else
    warn "Cannot access journal logs (may need sudo)"
fi

# Check for specific error patterns
output ""
info "Checking for common error patterns:"

if journalctl -u annad --no-pager 2>/dev/null | grep -q "readonly database"; then
    fail "Found 'readonly database' errors"
    output "   Run: sudo bash scripts/fix_v011_installation.sh"
else
    ok "No 'readonly database' errors"
fi

if journalctl -u annad --no-pager 2>/dev/null | grep -q "Permission denied"; then
    fail "Found 'Permission denied' errors"
    output "   Check directory ownership in SECTION 4"
else
    ok "No 'Permission denied' errors"
fi

if journalctl -u annad --no-pager 2>/dev/null | grep -q "CAPABILITIES.toml"; then
    warn "Found CAPABILITIES.toml warnings"
    output "   Ensure /usr/lib/anna/CAPABILITIES.toml exists"
else
    ok "No CAPABILITIES.toml warnings"
fi

# ============================================================================
# SECTION 10: CLI Commands
# ============================================================================

section "10. CLI COMMANDS"

if [ -x /usr/local/bin/annactl ]; then
    info "Testing annactl status:"
    if timeout 5 annactl status &>/dev/null; then
        ok "annactl status works"
        output ""
        annactl status 2>&1 | while IFS= read -r line; do
            output "   $line"
        done
    else
        fail "annactl status failed"
    fi

    output ""
    info "Testing annactl events:"
    if timeout 5 annactl events --limit 3 &>/dev/null; then
        ok "annactl events works"
    else
        warn "annactl events failed or returned no events"
    fi
else
    warn "annactl not found or not executable"
fi

# ============================================================================
# SECTION 11: Optional Dependencies
# ============================================================================

section "11. OPTIONAL DEPENDENCIES"

OPTIONAL_CMDS=(
    "sensors:lm_sensors"
    "ip:iproute2"
    "smartctl:smartmontools"
    "ethtool:ethtool"
)

for item in "${OPTIONAL_CMDS[@]}"; do
    CMD=$(echo "$item" | cut -d: -f1)
    PKG=$(echo "$item" | cut -d: -f2)

    info "Checking $CMD ($PKG):"
    if command -v "$CMD" &>/dev/null; then
        VERSION=$("$CMD" --version 2>&1 | head -1 || echo "installed")
        ok "Available: $VERSION"
    else
        warn "Not found (some telemetry features disabled)"
        output "   Install: sudo pacman -S $PKG"
    fi
done

# ============================================================================
# SECTION 12: Summary and Recommendations
# ============================================================================

section "12. SUMMARY AND RECOMMENDATIONS"

# Count issues
ISSUES_COUNT=0

if ! systemctl is-active --quiet annad 2>/dev/null; then
    ((ISSUES_COUNT++))
    fail "Issue #$ISSUES_COUNT: Daemon not running"
    output "   → Check logs in SECTION 9"
    output "   → Try: sudo systemctl restart annad"
fi

if [ ! -S /run/anna/annad.sock ]; then
    ((ISSUES_COUNT++))
    fail "Issue #$ISSUES_COUNT: Socket missing"
    output "   → Run: sudo bash scripts/fix_v011_installation.sh"
fi

if [ -f /var/lib/anna/telemetry.db ]; then
    DB_OWNER=$(stat -c '%U' /var/lib/anna/telemetry.db 2>/dev/null || stat -f '%Su' /var/lib/anna/telemetry.db)
    if [ "$DB_OWNER" != "anna" ]; then
        ((ISSUES_COUNT++))
        fail "Issue #$ISSUES_COUNT: Database wrong ownership"
        output "   → Run: sudo chown anna:anna /var/lib/anna/telemetry.db"
    fi
fi

if [ ! -f /usr/lib/anna/CAPABILITIES.toml ]; then
    ((ISSUES_COUNT++))
    fail "Issue #$ISSUES_COUNT: CAPABILITIES.toml missing"
    output "   → Run: sudo bash scripts/fix_v011_installation.sh"
fi

if [ $ISSUES_COUNT -eq 0 ]; then
    output ""
    ok "No critical issues detected"
    output ""
    output "Anna appears to be functioning correctly."
    output ""
else
    output ""
    fail "Found $ISSUES_COUNT critical issue(s)"
    output ""
    output "Quick fix: sudo bash scripts/fix_v011_installation.sh"
    output "Documentation: docs/V0.11.0_INSTALLATION_FIXES.md"
    output ""
fi

# End of diagnostics
section "END OF DIAGNOSTICS"

if [ -n "$OUTPUT_FILE" ]; then
    output ""
    ok "Diagnostics saved to: $OUTPUT_FILE"
    output "Share this file when reporting issues."
fi
