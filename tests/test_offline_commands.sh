#!/usr/bin/env bash
# Test all commands that work without daemon
set -euo pipefail

echo "Testing Anna offline commands..."
echo ""

PASS=0
FAIL=0

test_cmd() {
    local name="$1"
    shift
    echo -n "Testing $name... "
    if "$@" >/dev/null 2>&1; then
        echo "✓"
        ((PASS++))
    else
        echo "✗"
        ((FAIL++))
    fi
}

# Basic commands
test_cmd "annactl --version" ./target/release/annactl --version
test_cmd "annactl --help" ./target/release/annactl --help

# Doctor (works without daemon)
test_cmd "annactl doctor check" ./target/release/annactl doctor check

# Profile (works without daemon)
test_cmd "annactl profile show" ./target/release/annactl profile show
test_cmd "annactl profile checks" ./target/release/annactl profile checks
test_cmd "annactl profile checks --json" ./target/release/annactl profile checks --json

# Persona (works without daemon)
test_cmd "annactl persona list" ./target/release/annactl persona list
test_cmd "annactl persona get" ./target/release/annactl persona get

# Config (works without daemon)
test_cmd "annactl config list" ./target/release/annactl config list

echo ""
echo "Results: $PASS passed, $FAIL failed"

if [ $FAIL -eq 0 ]; then
    echo "✓ All offline commands work!"
    exit 0
else
    echo "✗ Some commands failed"
    exit 1
fi
