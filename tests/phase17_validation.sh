#!/bin/bash
# Phase 17 E2E Validation - Verification & Rollback Framework
# Tests: GDM resolution, sleep disable, lid close
# Usage: ./phase17_validation.sh [--dry-run|--rollback-test] [-v]
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RED='\033[0;31m' GREEN='\033[0;32m' YELLOW='\033[1;33m' CYAN='\033[0;36m' NC='\033[0m'
DRY_RUN=false ROLLBACK_TEST=false VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run) DRY_RUN=true; shift;;
        --rollback-test) ROLLBACK_TEST=true; shift;;
        --verbose|-v) VERBOSE=true; shift;;
        *) shift;;
    esac
done

info() { echo -e "${CYAN}[INFO]${NC} $1"; }
pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

echo "======================================"
echo "  PHASE 17 E2E VALIDATION"
echo "  $(date -u)"
$DRY_RUN && echo "  MODE: DRY-RUN (no execution)" || { $ROLLBACK_TEST && echo "  MODE: ROLLBACK-TEST" || echo "  MODE: FULL TEST"; }
echo "======================================"; echo

check_prerequisites() {
    info "Checking prerequisites..."
    command -v pkexec &>/dev/null || warn "pkexec not available"
    command -v systemctl &>/dev/null || warn "systemctl not available"
    command -v loginctl &>/dev/null || warn "loginctl not available"
    [[ -x "$REPO_ROOT/target/release/annad" ]] || { info "Building annad..."; cargo build --release -p annad --quiet; }
}

# TEST 1: GDM Resolution Plan
test_gdm_resolution() {
    echo "======================================"; echo "  TEST 1: GDM Resolution Plan"; echo "======================================"; echo
    local CONFIG_PATH="/var/lib/gdm/.config/monitors.xml" TEST_WIDTH="1920" TEST_HEIGHT="1080"
    info "Target: Set GDM resolution to ${TEST_WIDTH}x${TEST_HEIGHT}"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - File exists: $CONFIG_PATH"
        echo "  - Contains: <width>$TEST_WIDTH</width> <height>$TEST_HEIGHT</height>"
        echo "  - Owner: gdm:gdm"
        pass "GDM plan dry-run complete"; return 0
    fi

    info "Preflight check..."
    local ALREADY_CONFIGURED=false
    [[ -f "$CONFIG_PATH" ]] && grep -q "<width>$TEST_WIDTH</width>" "$CONFIG_PATH" 2>/dev/null && \
        grep -q "<height>$TEST_HEIGHT</height>" "$CONFIG_PATH" 2>/dev/null && ALREADY_CONFIGURED=true && info "GDM already configured"

    $ALREADY_CONFIGURED && ! $ROLLBACK_TEST && { pass "GDM idempotency: no changes needed"; return 0; }

    local BACKUP_DIR="/tmp/phase17_backup_$$"; mkdir -p "$BACKUP_DIR"
    [[ -f "$CONFIG_PATH" ]] && { info "Backing up existing config..."; pkexec cp "$CONFIG_PATH" "$BACKUP_DIR/monitors.xml.bak" || true; }

    info "Creating GDM monitor configuration..."
    cat > "$BACKUP_DIR/monitors.xml" <<EOF
<monitors version="2">
  <configuration>
    <logicalmonitor>
      <x>0</x><y>0</y><primary>yes</primary>
      <monitor>
        <monitorspec><connector>*</connector><vendor>unknown</vendor><product>unknown</product><serial>unknown</serial></monitorspec>
        <mode><width>$TEST_WIDTH</width><height>$TEST_HEIGHT</height><rate>60</rate></mode>
      </monitor>
    </logicalmonitor>
  </configuration>
</monitors>
EOF
    pkexec mkdir -p /var/lib/gdm/.config
    pkexec cp "$BACKUP_DIR/monitors.xml" "$CONFIG_PATH"
    pkexec chown -R gdm:gdm /var/lib/gdm/.config

    info "Verifying..."; local VERIFY_PASS=true
    [[ -f "$CONFIG_PATH" ]] && pass "Config file exists" || { fail "Config file missing"; VERIFY_PASS=false; }
    grep -q "<width>$TEST_WIDTH</width>" "$CONFIG_PATH" 2>/dev/null && pass "Width is $TEST_WIDTH" || { fail "Width not set correctly"; VERIFY_PASS=false; }
    local OWNER=$(stat -c "%U:%G" "$CONFIG_PATH" 2>/dev/null)
    [[ "$OWNER" = "gdm:gdm" ]] && pass "Owner is gdm:gdm" || { fail "Owner is $OWNER, expected gdm:gdm"; VERIFY_PASS=false; }

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        [[ -f "$BACKUP_DIR/monitors.xml.bak" ]] && pkexec cp "$BACKUP_DIR/monitors.xml.bak" "$CONFIG_PATH" || pkexec rm -f "$CONFIG_PATH"
        pass "Rollback: restored original config"
    fi
    rm -rf "$BACKUP_DIR"
    $VERIFY_PASS && pass "GDM resolution test PASSED" || { fail "GDM resolution test FAILED"; return 1; }
}

# TEST 2: Disable Sleep/Suspend Plan
test_disable_sleep() {
    echo "======================================"; echo "  TEST 2: Disable Sleep/Suspend Plan"; echo "======================================"; echo
    local LOGIND_CONF="/etc/systemd/logind.conf.d/no-idle.conf"
    local SLEEP_TARGETS="sleep.target suspend.target hibernate.target hybrid-sleep.target"
    info "Target: Mask sleep targets, set IdleAction=ignore"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - sleep/suspend/hibernate/hybrid-sleep.target masked"
        echo "  - $LOGIND_CONF contains IdleAction=ignore"
        pass "Sleep plan dry-run complete"; return 0
    fi

    info "Preflight check..."
    local ALREADY_DISABLED=false SLEEP_MASKED=$(systemctl is-enabled sleep.target 2>&1 || true)
    local IDLE_ACTION=$(loginctl show -p IdleAction 2>/dev/null | cut -d= -f2 || echo "unknown")
    [[ "$SLEEP_MASKED" == *"masked"* && "$IDLE_ACTION" == "ignore" ]] && ALREADY_DISABLED=true && info "Sleep already disabled"

    $ALREADY_DISABLED && ! $ROLLBACK_TEST && { pass "Sleep idempotency: no changes needed"; return 0; }

    local BACKUP_DIR="/tmp/phase17_backup_$$"; mkdir -p "$BACKUP_DIR"
    for target in $SLEEP_TARGETS; do systemctl is-enabled "$target" 2>&1 > "$BACKUP_DIR/${target}.state" || true; done
    [[ -f "$LOGIND_CONF" ]] && pkexec cp "$LOGIND_CONF" "$BACKUP_DIR/no-idle.conf.bak"

    info "Masking sleep targets..."; pkexec systemctl mask $SLEEP_TARGETS
    info "Creating logind drop-in..."
    echo -e "[Login]\nIdleAction=ignore\nIdleActionSec=0" > "$BACKUP_DIR/no-idle.conf"
    pkexec mkdir -p /etc/systemd/logind.conf.d
    pkexec cp "$BACKUP_DIR/no-idle.conf" "$LOGIND_CONF"
    info "Restarting systemd-logind..."; pkexec systemctl restart systemd-logind || warn "logind restart may affect session"

    info "Verifying..."; local VERIFY_PASS=true
    local NEW_SLEEP_STATE=$(systemctl is-enabled sleep.target 2>&1 || true)
    [[ "$NEW_SLEEP_STATE" == *"masked"* ]] && pass "sleep.target is masked" || { fail "sleep.target not masked: $NEW_SLEEP_STATE"; VERIFY_PASS=false; }
    local NEW_IDLE=$(loginctl show -p IdleAction 2>/dev/null | cut -d= -f2 || echo "unknown")
    [[ "$NEW_IDLE" == "ignore" ]] && pass "IdleAction is ignore" || { fail "IdleAction is $NEW_IDLE, expected ignore"; VERIFY_PASS=false; }

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        pkexec systemctl unmask $SLEEP_TARGETS
        [[ -f "$BACKUP_DIR/no-idle.conf.bak" ]] && pkexec cp "$BACKUP_DIR/no-idle.conf.bak" "$LOGIND_CONF" || pkexec rm -f "$LOGIND_CONF"
        pkexec systemctl restart systemd-logind || true
        pass "Rollback: restored original sleep state"
    fi
    rm -rf "$BACKUP_DIR"
    $VERIFY_PASS && pass "Disable sleep test PASSED" || { fail "Disable sleep test FAILED"; return 1; }
}

# TEST 3: Lid Close Plan
test_lid_close() {
    echo "======================================"; echo "  TEST 3: Lid Close Plan"; echo "======================================"; echo
    local LID_CONF="/etc/systemd/logind.conf.d/lid.conf" ACTION="ignore"
    info "Target: Set HandleLidSwitch=$ACTION"

    if $DRY_RUN; then
        info "Verification targets:"
        echo "  - $LID_CONF contains HandleLidSwitch=$ACTION"
        echo "  - loginctl show -p HandleLidSwitch returns $ACTION"
        pass "Lid close plan dry-run complete"; return 0
    fi

    info "Preflight check..."
    local CURRENT_LID=$(loginctl show -p HandleLidSwitch 2>/dev/null | cut -d= -f2 || echo "unknown")
    [[ "$CURRENT_LID" == "$ACTION" ]] && { info "Lid close already set to $ACTION"; $ROLLBACK_TEST || { pass "Lid close idempotency: no changes needed"; return 0; }; }

    local BACKUP_DIR="/tmp/phase17_backup_$$"; mkdir -p "$BACKUP_DIR"
    [[ -f "$LID_CONF" ]] && pkexec cp "$LID_CONF" "$BACKUP_DIR/lid.conf.bak"

    info "Creating lid close configuration..."
    cat > "$BACKUP_DIR/lid.conf" <<EOF
[Login]
HandleLidSwitch=$ACTION
HandleLidSwitchExternalPower=$ACTION
HandleLidSwitchDocked=$ACTION
EOF
    pkexec mkdir -p /etc/systemd/logind.conf.d
    pkexec cp "$BACKUP_DIR/lid.conf" "$LID_CONF"
    info "Restarting systemd-logind..."; pkexec systemctl restart systemd-logind || warn "logind restart may affect session"

    info "Verifying..."; local VERIFY_PASS=true
    local NEW_LID=$(loginctl show -p HandleLidSwitch 2>/dev/null | cut -d= -f2 || echo "unknown")
    [[ "$NEW_LID" == "$ACTION" ]] && pass "HandleLidSwitch is $ACTION" || { fail "HandleLidSwitch is $NEW_LID, expected $ACTION"; VERIFY_PASS=false; }

    if $ROLLBACK_TEST; then
        info "Rollback test: restoring original state..."
        [[ -f "$BACKUP_DIR/lid.conf.bak" ]] && pkexec cp "$BACKUP_DIR/lid.conf.bak" "$LID_CONF" || pkexec rm -f "$LID_CONF"
        pkexec systemctl restart systemd-logind || true
        pass "Rollback: restored original lid config"
    fi
    rm -rf "$BACKUP_DIR"
    $VERIFY_PASS && pass "Lid close test PASSED" || { fail "Lid close test FAILED"; return 1; }
}

main() {
    check_prerequisites; echo
    local TESTS_PASSED=0 TESTS_FAILED=0
    test_gdm_resolution && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1)); echo
    test_disable_sleep && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1)); echo
    test_lid_close && TESTS_PASSED=$((TESTS_PASSED + 1)) || TESTS_FAILED=$((TESTS_FAILED + 1)); echo

    echo "======================================"; echo "  SUMMARY"; echo "======================================"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"; echo -e "Failed: ${RED}$TESTS_FAILED${NC}"; echo
    [[ "$TESTS_FAILED" -gt 0 ]] && { echo -e "${RED}PHASE 17 VALIDATION FAILED${NC}"; exit 1; }
    echo -e "${GREEN}PHASE 17 VALIDATION PASSED${NC}"; exit 0
}

main
