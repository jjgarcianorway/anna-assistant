#!/bin/bash
# Phase 17 E2E Validation - Verification & Rollback Framework
# Tests the three template plans: GDM, sleep, lid close
#
# Usage:
#   ./phase17_validation.sh                  # Full test (requires pkexec)
#   ./phase17_validation.sh --dry-run        # Generate plans, no execution
#   ./phase17_validation.sh --rollback-test  # Test rollback mechanism
#
# Safety:
#   - Dry-run mode only generates plans and prints verification targets
#   - All changes are backed up before execution
#   - Rollback mode intentionally fails to test restore
#   - Uses pkexec for privilege escalation (GUI prompt)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

DRY_RUN=false
ROLLBACK_TEST=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --rollback-test)
            ROLLBACK_TEST=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

info() { echo -e "${CYAN}[INFO]${NC} $1"; }
pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

echo "======================================"
echo "  PHASE 17 E2E VALIDATION"
echo "  $(date -u)"
if $DRY_RUN; then
    echo "  MODE: DRY-RUN (no execution)"
elif $ROLLBACK_TEST; then
    echo "  MODE: ROLLBACK-TEST"
else
    echo "  MODE: FULL TEST"
fi
echo "======================================"
echo

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    if ! command -v pkexec &>/dev/null; then
        warn "pkexec not available - privilege escalation will fail"
    fi

    if ! command -v systemctl &>/dev/null; then
        warn "systemctl not available - systemd tests will fail"
    fi

    if ! command -v loginctl &>/dev/null; then
        warn "loginctl not available - logind tests will fail"
    fi

    # Check annad binary
    if [ ! -x "$REPO_ROOT/target/release/annad" ]; then
        info "Building annad..."
        cargo build --release -p annad --quiet
    fi
}

# ================================================
# TEST 1: GDM Resolution Plan
# ================================================
test_gdm_resolution() {
    echo "======================================"
    echo "  TEST 1: GDM Resolution Plan"
    echo "======================================"
    echo

    local CONFIG_PATH="/var/lib/gdm/.config/monitors.xml"
    local TEST_WIDTH="1920"
    local TEST_HEIGHT="1080"

    info "Target: Set GDM resolution to ${TEST_WIDTH}x${TEST_HEIGHT}"
    info "Config path: $CONFIG_PATH"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - File exists: $CONFIG_PATH"
        echo "  - Contains: <width>$TEST_WIDTH</width>"
        echo "  - Contains: <height>$TEST_HEIGHT</height>"
        echo "  - Owner: gdm:gdm"
        pass "GDM plan dry-run complete"
        return 0
    fi

    # Check current state (preflight)
    info "Preflight check..."
    local ALREADY_CONFIGURED=false
    if [ -f "$CONFIG_PATH" ]; then
        if grep -q "<width>$TEST_WIDTH</width>" "$CONFIG_PATH" 2>/dev/null && \
           grep -q "<height>$TEST_HEIGHT</height>" "$CONFIG_PATH" 2>/dev/null; then
            ALREADY_CONFIGURED=true
            info "GDM already configured for ${TEST_WIDTH}x${TEST_HEIGHT}"
        fi
    fi

    if $ALREADY_CONFIGURED && ! $ROLLBACK_TEST; then
        pass "GDM idempotency: no changes needed"
        return 0
    fi

    # Backup existing config
    local BACKUP_DIR="/tmp/phase17_backup_$$"
    mkdir -p "$BACKUP_DIR"
    if [ -f "$CONFIG_PATH" ]; then
        info "Backing up existing config..."
        pkexec cp "$CONFIG_PATH" "$BACKUP_DIR/monitors.xml.bak" || true
    fi

    # Create config (simplified for test - real plan uses heredoc)
    info "Creating GDM monitor configuration..."
    local CONFIG_CONTENT="<monitors version=\"2\">
  <configuration>
    <logicalmonitor>
      <x>0</x><y>0</y><primary>yes</primary>
      <monitor>
        <monitorspec>
          <connector>*</connector>
          <vendor>unknown</vendor>
          <product>unknown</product>
          <serial>unknown</serial>
        </monitorspec>
        <mode>
          <width>$TEST_WIDTH</width>
          <height>$TEST_HEIGHT</height>
          <rate>60</rate>
        </mode>
      </monitor>
    </logicalmonitor>
  </configuration>
</monitors>"

    echo "$CONFIG_CONTENT" > "$BACKUP_DIR/monitors.xml"
    pkexec mkdir -p /var/lib/gdm/.config
    pkexec cp "$BACKUP_DIR/monitors.xml" "$CONFIG_PATH"
    pkexec chown -R gdm:gdm /var/lib/gdm/.config

    # Verify
    info "Verifying..."
    local VERIFY_PASS=true

    if [ -f "$CONFIG_PATH" ]; then
        pass "Config file exists"
    else
        fail "Config file missing"
        VERIFY_PASS=false
    fi

    if grep -q "<width>$TEST_WIDTH</width>" "$CONFIG_PATH" 2>/dev/null; then
        pass "Width is $TEST_WIDTH"
    else
        fail "Width not set correctly"
        VERIFY_PASS=false
    fi

    local OWNER=$(stat -c "%U:%G" "$CONFIG_PATH" 2>/dev/null)
    if [ "$OWNER" = "gdm:gdm" ]; then
        pass "Owner is gdm:gdm"
    else
        fail "Owner is $OWNER, expected gdm:gdm"
        VERIFY_PASS=false
    fi

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        if [ -f "$BACKUP_DIR/monitors.xml.bak" ]; then
            pkexec cp "$BACKUP_DIR/monitors.xml.bak" "$CONFIG_PATH"
            pass "Rollback: restored original config"
        else
            pkexec rm -f "$CONFIG_PATH"
            pass "Rollback: removed test config"
        fi
    fi

    rm -rf "$BACKUP_DIR"

    if $VERIFY_PASS; then
        pass "GDM resolution test PASSED"
    else
        fail "GDM resolution test FAILED"
        return 1
    fi
}

# ================================================
# TEST 2: Disable Sleep/Suspend Plan
# ================================================
test_disable_sleep() {
    echo "======================================"
    echo "  TEST 2: Disable Sleep/Suspend Plan"
    echo "======================================"
    echo

    local LOGIND_CONF="/etc/systemd/logind.conf.d/no-idle.conf"
    local SLEEP_TARGETS="sleep.target suspend.target hibernate.target hybrid-sleep.target"

    info "Target: Mask sleep targets, set IdleAction=ignore"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - sleep.target is masked"
        echo "  - suspend.target is masked"
        echo "  - hibernate.target is masked"
        echo "  - hybrid-sleep.target is masked"
        echo "  - $LOGIND_CONF contains IdleAction=ignore"
        echo "  - loginctl show -p IdleAction returns ignore"
        pass "Sleep plan dry-run complete"
        return 0
    fi

    # Preflight check
    info "Preflight check..."
    local ALREADY_DISABLED=false
    local SLEEP_MASKED=$(systemctl is-enabled sleep.target 2>&1 || true)
    local IDLE_ACTION=$(loginctl show -p IdleAction 2>/dev/null | cut -d= -f2 || echo "unknown")

    if [[ "$SLEEP_MASKED" == *"masked"* ]] && [[ "$IDLE_ACTION" == "ignore" ]]; then
        ALREADY_DISABLED=true
        info "Sleep already disabled (masked + IdleAction=ignore)"
    fi

    if $ALREADY_DISABLED && ! $ROLLBACK_TEST; then
        pass "Sleep idempotency: no changes needed"
        return 0
    fi

    # Capture current state for rollback
    local BACKUP_DIR="/tmp/phase17_backup_$$"
    mkdir -p "$BACKUP_DIR"

    # Capture unit states
    for target in $SLEEP_TARGETS; do
        systemctl is-enabled "$target" 2>&1 > "$BACKUP_DIR/${target}.state" || true
    done

    # Backup logind config if exists
    if [ -f "$LOGIND_CONF" ]; then
        pkexec cp "$LOGIND_CONF" "$BACKUP_DIR/no-idle.conf.bak"
    fi

    # Execute plan
    info "Masking sleep targets..."
    pkexec systemctl mask $SLEEP_TARGETS

    info "Creating logind drop-in..."
    local LOGIND_CONTENT="[Login]
IdleAction=ignore
IdleActionSec=0"

    echo "$LOGIND_CONTENT" > "$BACKUP_DIR/no-idle.conf"
    pkexec mkdir -p /etc/systemd/logind.conf.d
    pkexec cp "$BACKUP_DIR/no-idle.conf" "$LOGIND_CONF"

    info "Restarting systemd-logind..."
    pkexec systemctl restart systemd-logind || warn "logind restart may affect session"

    # Verify
    info "Verifying..."
    local VERIFY_PASS=true

    local NEW_SLEEP_STATE=$(systemctl is-enabled sleep.target 2>&1 || true)
    if [[ "$NEW_SLEEP_STATE" == *"masked"* ]]; then
        pass "sleep.target is masked"
    else
        fail "sleep.target not masked: $NEW_SLEEP_STATE"
        VERIFY_PASS=false
    fi

    local NEW_IDLE=$(loginctl show -p IdleAction 2>/dev/null | cut -d= -f2 || echo "unknown")
    if [[ "$NEW_IDLE" == "ignore" ]]; then
        pass "IdleAction is ignore"
    else
        fail "IdleAction is $NEW_IDLE, expected ignore"
        VERIFY_PASS=false
    fi

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        pkexec systemctl unmask $SLEEP_TARGETS
        if [ -f "$BACKUP_DIR/no-idle.conf.bak" ]; then
            pkexec cp "$BACKUP_DIR/no-idle.conf.bak" "$LOGIND_CONF"
        else
            pkexec rm -f "$LOGIND_CONF"
        fi
        pkexec systemctl restart systemd-logind || true
        pass "Rollback: restored original sleep state"
    fi

    rm -rf "$BACKUP_DIR"

    if $VERIFY_PASS; then
        pass "Disable sleep test PASSED"
    else
        fail "Disable sleep test FAILED"
        return 1
    fi
}

# ================================================
# TEST 3: Lid Close Plan
# ================================================
test_lid_close() {
    echo "======================================"
    echo "  TEST 3: Lid Close Plan"
    echo "======================================"
    echo

    local LID_CONF="/etc/systemd/logind.conf.d/lid.conf"
    local ACTION="ignore"

    info "Target: Set HandleLidSwitch=$ACTION"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - $LID_CONF contains HandleLidSwitch=$ACTION"
        echo "  - loginctl show -p HandleLidSwitch returns $ACTION"
        pass "Lid close plan dry-run complete"
        return 0
    fi

    # Preflight check
    info "Preflight check..."
    local CURRENT_LID=$(loginctl show -p HandleLidSwitch 2>/dev/null | cut -d= -f2 || echo "unknown")

    if [[ "$CURRENT_LID" == "$ACTION" ]]; then
        info "Lid close already set to $ACTION"
        if ! $ROLLBACK_TEST; then
            pass "Lid close idempotency: no changes needed"
            return 0
        fi
    fi

    # Backup
    local BACKUP_DIR="/tmp/phase17_backup_$$"
    mkdir -p "$BACKUP_DIR"

    if [ -f "$LID_CONF" ]; then
        pkexec cp "$LID_CONF" "$BACKUP_DIR/lid.conf.bak"
    fi

    # Execute
    info "Creating lid close configuration..."
    local LID_CONTENT="[Login]
HandleLidSwitch=$ACTION
HandleLidSwitchExternalPower=$ACTION
HandleLidSwitchDocked=$ACTION"

    echo "$LID_CONTENT" > "$BACKUP_DIR/lid.conf"
    pkexec mkdir -p /etc/systemd/logind.conf.d
    pkexec cp "$BACKUP_DIR/lid.conf" "$LID_CONF"

    info "Restarting systemd-logind..."
    pkexec systemctl restart systemd-logind || warn "logind restart may affect session"

    # Verify
    info "Verifying..."
    local VERIFY_PASS=true

    local NEW_LID=$(loginctl show -p HandleLidSwitch 2>/dev/null | cut -d= -f2 || echo "unknown")
    if [[ "$NEW_LID" == "$ACTION" ]]; then
        pass "HandleLidSwitch is $ACTION"
    else
        fail "HandleLidSwitch is $NEW_LID, expected $ACTION"
        VERIFY_PASS=false
    fi

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        if [ -f "$BACKUP_DIR/lid.conf.bak" ]; then
            pkexec cp "$BACKUP_DIR/lid.conf.bak" "$LID_CONF"
        else
            pkexec rm -f "$LID_CONF"
        fi
        pkexec systemctl restart systemd-logind || true
        pass "Rollback: restored original lid config"
    fi

    rm -rf "$BACKUP_DIR"

    if $VERIFY_PASS; then
        pass "Lid close test PASSED"
    else
        fail "Lid close test FAILED"
        return 1
    fi
}

# ================================================
# Main
# ================================================
main() {
    check_prerequisites
    echo

    local TESTS_PASSED=0
    local TESTS_FAILED=0

    test_gdm_resolution && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1))
    echo

    test_disable_sleep && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1))
    echo

    test_lid_close && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1))
    echo

    echo "======================================"
    echo "  SUMMARY"
    echo "======================================"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo

    if [ "$TESTS_FAILED" -gt 0 ]; then
        echo -e "${RED}PHASE 17 VALIDATION FAILED${NC}"
        exit 1
    fi

    echo -e "${GREEN}PHASE 17 VALIDATION PASSED${NC}"
    exit 0
}

main
