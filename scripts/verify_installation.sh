#!/usr/bin/env bash
# Verification script - run after ./scripts/install.sh
# This script checks that Anna is properly installed and running

set -euo pipefail

echo "════════════════════════════════════════════════════════"
echo "  Anna Installation Verification"
echo "════════════════════════════════════════════════════════"
echo ""

PASS=0
FAIL=0

check() {
    local name="$1"
    local cmd="$2"

    echo -n "Checking $name... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "✓"
        ((PASS++))
        return 0
    else
        echo "✗"
        ((FAIL++))
        return 1
    fi
}

check_verbose() {
    local name="$1"
    local cmd="$2"
    local expected="$3"

    echo -n "Checking $name... "
    result=$(eval "$cmd" 2>&1 || echo "FAILED")
    if [[ "$result" == "$expected" ]]; then
        echo "✓"
        ((PASS++))
        return 0
    else
        echo "✗ (got: $result, expected: $expected)"
        ((FAIL++))
        return 1
    fi
}

echo "=== Binaries ==="
check "annad installed" "test -x /usr/local/bin/annad"
check "annactl installed" "test -x /usr/local/bin/annactl"
check "annactl version" "annactl --version | grep -q '0.9.6-alpha.6'"

echo ""
echo "=== Directories ==="
check "/etc/anna exists" "test -d /etc/anna"
check "/var/lib/anna exists" "test -d /var/lib/anna"
check "/var/log/anna exists" "test -d /var/log/anna"
check "/run/anna exists" "test -d /run/anna"
check "/etc/anna/policies.d exists" "test -d /etc/anna/policies.d"

echo ""
echo "=== Configuration ==="
check "version file" "test -f /etc/anna/version"
check "config file" "test -f /etc/anna/config.toml"
check "bootstrap policy" "test -f /etc/anna/policies.d/00-bootstrap.yaml"

echo ""
echo "=== System Service ==="
check "systemd service file" "test -f /etc/systemd/system/annad.service"
check "service enabled" "systemctl is-enabled annad >/dev/null 2>&1"
check_verbose "service status" "systemctl is-active annad 2>/dev/null" "active"

echo ""
echo "=== Runtime ==="
if check "socket exists" "test -S /run/anna/annad.sock"; then
    echo "  Socket permissions: $(ls -l /run/anna/annad.sock | awk '{print $1, $3, $4}')"
fi

echo ""
echo "=== Group Membership ==="
if groups | grep -q anna; then
    echo "✓ Current user is in 'anna' group"
    ((PASS++))
else
    echo "✗ Current user NOT in 'anna' group (may need to log out/in)"
    echo "  Run: sudo usermod -aG anna $USER"
    ((FAIL++))
fi

echo ""
echo "=== Daemon Logs ==="
if systemctl is-active annad >/dev/null 2>&1; then
    echo "Recent daemon logs:"
    journalctl -u annad -n 5 --no-pager 2>/dev/null || echo "  (cannot read logs without permission)"
fi

echo ""
echo "=== Online Commands (require running daemon) ==="
if systemctl is-active annad >/dev/null 2>&1; then
    check "annactl ping" "timeout 2 annactl ping"
    if check "annactl status" "timeout 2 annactl status >/dev/null"; then
        echo ""
        echo "Status output:"
        annactl status 2>/dev/null | head -20
    fi
else
    echo "⊘ Skipping online tests (daemon not running)"
fi

echo ""
echo "=== Offline Commands ==="
check "annactl doctor check" "annactl doctor check"
check "annactl profile show" "annactl profile show >/dev/null"
check "annactl profile checks" "annactl profile checks >/dev/null"
check "annactl persona list" "annactl persona list >/dev/null"

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Verification Summary"
echo "════════════════════════════════════════════════════════"
echo ""
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✓ Anna is properly installed and running!"
    echo ""
    echo "Try these commands:"
    echo "  annactl status"
    echo "  annactl profile show"
    echo "  annactl profile checks"
    echo "  annactl doctor check"
    exit 0
else
    echo "✗ Some checks failed. Review the errors above."
    echo ""
    echo "Common fixes:"
    echo "  • Daemon not running: sudo systemctl start annad"
    echo "  • Check logs: journalctl -u annad -n 50"
    echo "  • Run repair: annactl doctor repair"
    exit 1
fi
