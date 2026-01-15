#!/bin/bash
# Release Verification Script
# Verifies GitHub release has required binaries and valid checksums
# Uses curl fallback when gh CLI is unavailable

set -e

REPO="jjgarcianorway/anna-assistant"
REQUIRED_ASSETS=("annactl-linux-x86_64" "annad-linux-x86_64" "SHA256SUMS")

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }

usage() {
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.3.50"
    exit 1
}

verify_with_gh() {
    local version=$1

    if ! gh release view "$version" &>/dev/null; then
        fail "Release $version does not exist"
    fi
    pass "Release $version exists"

    local assets
    assets=$(gh release view "$version" --json assets -q '.assets[].name')

    for asset in "${REQUIRED_ASSETS[@]}"; do
        if echo "$assets" | grep -q "^${asset}$"; then
            pass "Asset present: $asset"
        else
            fail "Missing asset: $asset"
        fi
    done
}

verify_with_curl() {
    local version=$1
    local api_url="https://api.github.com/repos/$REPO/releases/tags/$version"

    local response
    response=$(curl -sf "$api_url" 2>/dev/null) || fail "Release $version not found"
    pass "Release $version exists"

    for asset in "${REQUIRED_ASSETS[@]}"; do
        if echo "$response" | grep -q "\"name\": \"$asset\""; then
            pass "Asset present: $asset"
        else
            fail "Missing asset: $asset"
        fi
    done
}

verify_checksums() {
    local version=$1
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    echo "=== Verifying Checksums ==="

    local base_url="https://github.com/$REPO/releases/download/$version"

    # Download SHA256SUMS
    if ! curl -sfL "$base_url/SHA256SUMS" -o "$tmpdir/SHA256SUMS"; then
        fail "Could not download SHA256SUMS"
    fi

    # Download and verify each binary
    for binary in "annactl-linux-x86_64" "annad-linux-x86_64"; do
        echo "Downloading $binary..."
        if ! curl -sfL "$base_url/$binary" -o "$tmpdir/$binary"; then
            fail "Could not download $binary"
        fi

        # Extract expected hash
        local expected
        expected=$(grep "$binary" "$tmpdir/SHA256SUMS" | awk '{print $1}')
        if [ -z "$expected" ]; then
            fail "$binary not found in SHA256SUMS"
        fi

        # Compute actual hash
        local actual
        actual=$(sha256sum "$tmpdir/$binary" | awk '{print $1}')

        if [ "$expected" = "$actual" ]; then
            pass "$binary checksum valid"
        else
            echo "Expected: $expected"
            echo "Actual:   $actual"
            fail "$binary checksum mismatch"
        fi
    done
}

main() {
    [ $# -eq 1 ] || usage
    local version=$1

    echo "======================================"
    echo "  RELEASE VERIFICATION: $version"
    echo "======================================"
    echo

    echo "=== Checking Release Assets ==="
    if command -v gh &>/dev/null; then
        verify_with_gh "$version"
    else
        echo "(gh CLI not available, using curl fallback)"
        verify_with_curl "$version"
    fi
    echo

    verify_checksums "$version"
    echo

    echo "======================================"
    echo -e "${GREEN}RELEASE VERIFIED${NC}"
    echo "======================================"
}

main "$@"
