#!/bin/bash
# Anna Assistant - Release Validation Script
# Tests clean installation on fresh Arch Linux system
#
# Usage: curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/validate_release.sh | bash
#
# This script validates:
# - One-line installer functionality
# - Binary installation and permissions
# - Service startup and socket availability
# - annactl command functionality
# - Version verification
# - Self-update script presence

set -e

EXPECTED_VERSION="1.16.1-alpha.1"
INSTALL_DIR="/usr/local/bin"
SOCKET_PATH="/run/anna/anna.sock"

# Colors
if [ -t 1 ] && command -v tput >/dev/null 2>&1 && [ "$(tput colors)" -ge 256 ]; then
    BLUE='\033[38;5;117m'
    GREEN='\033[38;5;120m'
    YELLOW='\033[38;5;228m'
    RED='\033[38;5;210m'
    CYAN='\033[38;5;159m'
    GRAY='\033[38;5;250m'
    RESET='\033[0m'
    BOLD='\033[1m'
    CHECK="✓"; CROSS="✗"; ARROW="→"
else
    BLUE=''; GREEN=''; YELLOW=''; RED=''; CYAN=''; GRAY=''; RESET=''; BOLD=''
    CHECK="[OK]"; CROSS="[X]"; ARROW="->"
fi

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILURES=()

print_header() {
    echo
    echo -e "${BOLD}${CYAN}========================================${RESET}"
    echo -e "${BOLD}${CYAN}  Anna Assistant Release Validator${RESET}"
    echo -e "${BOLD}${CYAN}  Target: v${EXPECTED_VERSION}${RESET}"
    echo -e "${BOLD}${CYAN}========================================${RESET}"
    echo
}

test_pass() {
    echo -e "${GREEN}${CHECK}${RESET} $1"
    ((TESTS_PASSED++))
}

test_fail() {
    echo -e "${RED}${CROSS}${RESET} $1"
    FAILURES+=("$1")
    ((TESTS_FAILED++))
}

test_warn() {
    echo -e "${YELLOW}⚠${RESET}  $1"
}

test_info() {
    echo -e "${CYAN}${ARROW}${RESET} $1"
}

# Test 1: Check if Anna is already installed
test_info "Checking for existing installation..."
if command -v annad >/dev/null 2>&1 || command -v annactl >/dev/null 2>&1; then
    CURRENT_VERSION=$(annad --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
    test_warn "Anna already installed (version: ${CURRENT_VERSION})"
    test_info "This script validates existing installation"
    echo
else
    test_info "No existing installation found - fresh install test"
    test_info "Run the installer first:"
    echo -e "  ${GRAY}curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo bash${RESET}"
    exit 1
fi

# Test 2: Verify binaries exist
echo
test_info "Test 2: Binary Installation"
if [ -f "$INSTALL_DIR/annad" ]; then
    test_pass "annad binary exists at $INSTALL_DIR/annad"
else
    test_fail "annad binary missing"
fi

if [ -f "$INSTALL_DIR/annactl" ]; then
    test_pass "annactl binary exists at $INSTALL_DIR/annactl"
else
    test_fail "annactl binary missing"
fi

# Test 3: Verify binary permissions
echo
test_info "Test 3: Binary Permissions"
if [ -x "$INSTALL_DIR/annad" ]; then
    test_pass "annad is executable"
else
    test_fail "annad is not executable"
fi

if [ -x "$INSTALL_DIR/annactl" ]; then
    test_pass "annactl is executable"
else
    test_fail "annactl is not executable"
fi

# Test 4: Verify version
echo
test_info "Test 4: Version Verification"
ANNAD_VERSION=$(annad --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
ANNACTL_VERSION=$(annactl --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")

if [ "$ANNAD_VERSION" = "$EXPECTED_VERSION" ]; then
    test_pass "annad version: $ANNAD_VERSION (expected: $EXPECTED_VERSION)"
else
    test_fail "annad version mismatch: got $ANNAD_VERSION, expected $EXPECTED_VERSION"
fi

if [ "$ANNACTL_VERSION" = "$EXPECTED_VERSION" ]; then
    test_pass "annactl version: $ANNACTL_VERSION (expected: $EXPECTED_VERSION)"
else
    test_fail "annactl version mismatch: got $ANNACTL_VERSION, expected $EXPECTED_VERSION"
fi

# Test 5: Systemd service
echo
test_info "Test 5: Systemd Service"
if systemctl is-enabled annad >/dev/null 2>&1; then
    test_pass "annad service is enabled"
else
    test_fail "annad service is not enabled"
fi

if systemctl is-active annad >/dev/null 2>&1; then
    test_pass "annad service is active"
else
    test_fail "annad service is not active"
    test_info "Checking service status..."
    sudo systemctl status annad --no-pager || true
fi

# Test 6: Socket availability
echo
test_info "Test 6: Unix Socket"
if [ -S "$SOCKET_PATH" ]; then
    test_pass "Socket exists at $SOCKET_PATH"
else
    test_fail "Socket missing at $SOCKET_PATH"
fi

# Wait for socket to be ready
SOCKET_READY=0
for i in $(seq 1 10); do
    if annactl help >/dev/null 2>&1; then
        SOCKET_READY=1
        break
    fi
    sleep 1
done

if [ "$SOCKET_READY" -eq 1 ]; then
    test_pass "Socket is responsive"
else
    test_fail "Socket is not responsive"
fi

# Test 7: annactl commands
echo
test_info "Test 7: annactl Commands"

# Test help command
if annactl help >/dev/null 2>&1; then
    test_pass "annactl help works"
else
    test_fail "annactl help failed"
fi

# Test status command
if annactl status >/dev/null 2>&1; then
    test_pass "annactl status works"
else
    test_fail "annactl status failed"
fi

# Test doctor command
if annactl doctor >/dev/null 2>&1; then
    test_pass "annactl doctor works"
else
    test_fail "annactl doctor failed"
fi

# Test 8: Self-update script
echo
test_info "Test 8: Self-Update Infrastructure"

# Check if self-update script exists in repo
if [ -f "/usr/local/lib/anna/scripts/self_update.sh" ]; then
    test_pass "self_update.sh exists"
    if [ -x "/usr/local/lib/anna/scripts/self_update.sh" ]; then
        test_pass "self_update.sh is executable"
    else
        test_fail "self_update.sh is not executable"
    fi
elif [ -f "/opt/anna-assistant/scripts/self_update.sh" ]; then
    test_pass "self_update.sh exists (legacy location)"
else
    test_warn "self_update.sh not found (check if installer deployed it)"
fi

# Test 9: Configuration files
echo
test_info "Test 9: Configuration"

if [ -f "/etc/anna/config.toml" ]; then
    test_pass "config.toml exists"
else
    test_warn "config.toml not found (using defaults)"
fi

# Test 10: User and group
echo
test_info "Test 10: User and Group Setup"

if id anna >/dev/null 2>&1; then
    test_pass "anna user exists"
else
    test_fail "anna user missing"
fi

if getent group anna >/dev/null 2>&1; then
    test_pass "anna group exists"
else
    test_fail "anna group missing"
fi

# Test 11: Runtime directory
echo
test_info "Test 11: Runtime Directory"

if [ -d "/run/anna" ]; then
    test_pass "/run/anna directory exists"

    OWNER=$(stat -c '%U' /run/anna)
    GROUP=$(stat -c '%G' /run/anna)
    PERMS=$(stat -c '%a' /run/anna)

    if [ "$OWNER" = "root" ] && [ "$GROUP" = "anna" ]; then
        test_pass "Ownership correct (root:anna)"
    else
        test_fail "Ownership incorrect: $OWNER:$GROUP (expected root:anna)"
    fi

    if [ "$PERMS" = "750" ]; then
        test_pass "Permissions correct (750)"
    else
        test_fail "Permissions incorrect: $PERMS (expected 750)"
    fi
else
    test_fail "/run/anna directory missing"
fi

# Test 12: Systemd hardening
echo
test_info "Test 12: Security Hardening"

HARDENING_CHECKS=(
    "NoNewPrivileges=yes"
    "PrivateTmp=yes"
    "ProtectSystem=strict"
    "ProtectHome=yes"
)

SYSTEMD_UNIT=$(systemctl cat annad 2>/dev/null)
if [ -n "$SYSTEMD_UNIT" ]; then
    HARDENING_OK=0
    for check in "${HARDENING_CHECKS[@]}"; do
        if echo "$SYSTEMD_UNIT" | grep -q "^$check"; then
            ((HARDENING_OK++))
        fi
    done

    if [ "$HARDENING_OK" -ge 3 ]; then
        test_pass "Systemd hardening enabled ($HARDENING_OK/${#HARDENING_CHECKS[@]} checks)"
    else
        test_warn "Partial systemd hardening ($HARDENING_OK/${#HARDENING_CHECKS[@]} checks)"
    fi
else
    test_fail "Could not read systemd unit"
fi

# Summary
echo
echo -e "${BOLD}${CYAN}========================================${RESET}"
echo -e "${BOLD}${CYAN}  Validation Summary${RESET}"
echo -e "${BOLD}${CYAN}========================================${RESET}"
echo
echo -e "${GREEN}${CHECK} Passed:${RESET} ${BOLD}${TESTS_PASSED}${RESET}"
echo -e "${RED}${CROSS} Failed:${RESET} ${BOLD}${TESTS_FAILED}${RESET}"
echo

if [ ${#FAILURES[@]} -gt 0 ]; then
    echo -e "${RED}${BOLD}Failed Tests:${RESET}"
    for failure in "${FAILURES[@]}"; do
        echo -e "  ${RED}${CROSS}${RESET} $failure"
    done
    echo
fi

# Exit code
if [ "$TESTS_FAILED" -eq 0 ]; then
    echo -e "${BOLD}${GREEN}✓ All tests passed!${RESET}"
    echo -e "${GRAY}Anna Assistant v${EXPECTED_VERSION} is correctly installed.${RESET}"
    echo
    exit 0
else
    echo -e "${BOLD}${RED}✗ Some tests failed${RESET}"
    echo -e "${GRAY}Check the failures above and consult the documentation.${RESET}"
    echo
    exit 1
fi
