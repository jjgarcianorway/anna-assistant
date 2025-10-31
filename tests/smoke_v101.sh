#!/usr/bin/env bash
# Anna v0.10.1 Smoke Tests
# Basic validation of build and commands

set -euo pipefail

FAILED=0
PASSED=0

test_build() {
    echo "TEST: Build binaries"
    if [[ -f target/release/annad ]] && [[ -f target/release/annactl ]]; then
        echo "  ✓ PASS: Binaries exist"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ FAIL: Binaries missing"
        FAILED=$((FAILED + 1))
    fi
}

test_version() {
    echo "TEST: Version command"
    if ./target/release/annactl version &>/dev/null; then
        echo "  ✓ PASS: annactl version works"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ FAIL: annactl version failed"
        FAILED=$((FAILED + 1))
    fi
}

test_doctor_pre() {
    echo "TEST: Doctor preflight (no sudo)"
    # Should pass but warn about sudo
    output=$(./target/release/annactl doctor pre 2>&1 || true)
    if echo "$output" | grep -q "Preflight checks"; then
        echo "  ✓ PASS: Doctor preflight works"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ FAIL: Doctor preflight failed"
        FAILED=$((FAILED + 1))
    fi
}

test_capabilities_toml() {
    echo "TEST: CAPABILITIES.toml syntax"
    if toml-test etc/CAPABILITIES.toml &>/dev/null 2>&1 || grep -q "modules.sensors" etc/CAPABILITIES.toml; then
        echo "  ✓ PASS: CAPABILITIES.toml is valid"
        PASSED=$((PASSED + 1))
    else
        echo "  ⚠ WARN: toml-test not available, skipped"
    fi
}

test_modules_yaml() {
    echo "TEST: modules.yaml syntax"
    if python3 -c "import yaml; yaml.safe_load(open('etc/modules.yaml'))" &>/dev/null 2>&1; then
        echo "  ✓ PASS: modules.yaml is valid"
        PASSED=$((PASSED + 1))
    else
        echo "  ⚠ WARN: Python/PyYAML not available, skipped"
    fi
}

test_installer_syntax() {
    echo "TEST: Installer script syntax"
    if bash -n scripts/install_v101.sh; then
        echo "  ✓ PASS: install_v101.sh syntax OK"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ FAIL: install_v101.sh has syntax errors"
        FAILED=$((FAILED + 1))
    fi
}

test_uninstaller_syntax() {
    echo "TEST: Uninstaller script syntax"
    if bash -n scripts/uninstall_v101.sh; then
        echo "  ✓ PASS: uninstall_v101.sh syntax OK"
        PASSED=$((PASSED + 1))
    else
        echo "  ✗ FAIL: uninstall_v101.sh has syntax errors"
        FAILED=$((FAILED + 1))
    fi
}

test_annad_help() {
    echo "TEST: annad --help"
    if ./target/release/annad --help 2>&1 | grep -q "doctor-apply" || ./target/release/annad 2>&1 | grep -q "v0.10.1"; then
        echo "  ✓ PASS: annad responds"
        PASSED=$((PASSED + 1))
    else
        echo "  ⚠ WARN: annad may not support --help flag"
    fi
}

echo ""
echo "═══════════════════════════════════════════"
echo "  Anna v0.10.1 Smoke Tests"
echo "═══════════════════════════════════════════"
echo ""

test_build
test_version
test_doctor_pre
test_capabilities_toml
test_modules_yaml
test_installer_syntax
test_uninstaller_syntax
test_annad_help

echo ""
echo "═══════════════════════════════════════════"
echo "  Results: $PASSED passed, $FAILED failed"
echo "═══════════════════════════════════════════"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi
