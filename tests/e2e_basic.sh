#!/usr/bin/env bash
# Basic End-to-End Test for Anna Assistant
# Tests core functionality without requiring a full install

set -euo pipefail

echo "═══════════════════════════════════════════"
echo "  Anna Assistant - Basic E2E Tests"
echo "═══════════════════════════════════════════"
echo ""

FAILED=0
PASSED=0

pass() {
    echo "✓ $1"
    ((PASSED++))
}

fail() {
    echo "✗ $1"
    ((FAILED++))
}

test_build() {
    echo "[TEST] Building binaries..."
    if cargo build --release --quiet 2>&1; then
        pass "Build succeeded"
    else
        fail "Build failed"
    fi
}

test_version_consistency() {
    echo "[TEST] Checking version consistency..."

    # Extract version from Cargo.toml
    CARGO_VERSION=$(grep -A 1 '\[workspace.package\]' Cargo.toml | grep version | cut -d'"' -f2)

    # Check annactl version
    ANNACTL_VERSION=$(./target/release/annactl --version | awk '{print $2}')

    if [ "$CARGO_VERSION" = "$ANNACTL_VERSION" ]; then
        pass "Version consistency: $CARGO_VERSION"
    else
        fail "Version mismatch: Cargo=$CARGO_VERSION, annactl=$ANNACTL_VERSION"
    fi
}

test_annactl_help() {
    echo "[TEST] Testing annactl --help..."
    if ./target/release/annactl --help > /dev/null 2>&1; then
        pass "annactl help works"
    else
        fail "annactl help failed"
    fi
}

test_profile_checks() {
    echo "[TEST] Testing annactl profile checks --json..."
    if ./target/release/annactl profile checks --json > /tmp/anna_checks.json 2>&1; then
        # Validate JSON
        if command -v jq &>/dev/null; then
            if jq empty /tmp/anna_checks.json 2>/dev/null; then
                pass "Profile checks JSON valid"
            else
                fail "Profile checks JSON invalid"
            fi
        else
            pass "Profile checks command succeeded (jq not available to validate)"
        fi
    else
        fail "Profile checks command failed"
    fi
}

test_doctor_check_without_daemon() {
    echo "[TEST] Testing annactl doctor check (without daemon)..."
    # Doctor should work even without daemon running
    if ./target/release/annactl doctor check 2>&1 | grep -q "Check"; then
        pass "Doctor check works without daemon"
    else
        fail "Doctor check failed"
    fi
}

test_installer_syntax() {
    echo "[TEST] Checking installer syntax..."
    if bash -n scripts/install.sh; then
        pass "Installer syntax valid"
    else
        fail "Installer syntax invalid"
    fi
}

test_systemd_service_file() {
    echo "[TEST] Checking systemd service file..."
    if [ -f etc/systemd/annad.service ]; then
        if grep -q "ExecStart=/usr/local/bin/annad" etc/systemd/annad.service; then
            pass "Systemd service file valid"
        else
            fail "Systemd service file missing ExecStart"
        fi
    else
        fail "Systemd service file not found"
    fi
}

test_default_config_template() {
    echo "[TEST] Checking default config template in installer..."
    if grep -q "EOF_CONFIG" scripts/install.sh; then
        pass "Default config template present"
    else
        fail "Default config template missing"
    fi
}

# Run tests
echo ""
test_build
test_version_consistency
test_annactl_help
test_profile_checks
test_doctor_check_without_daemon
test_installer_syntax
test_systemd_service_file
test_default_config_template

# Summary
echo ""
echo "═══════════════════════════════════════════"
echo "  Test Summary"
echo "═══════════════════════════════════════════"
echo ""
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi
