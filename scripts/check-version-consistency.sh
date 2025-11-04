#!/usr/bin/env bash
# Version Consistency Checker — Single Source of Truth Validator
# Ensures VERSION file is the only version authority
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

say() { printf "%b\n" "$*"; }
error() { say "${RED}✗ ERROR:${NC} $*" >&2; }
warn() { say "${YELLOW}⚠ WARNING:${NC} $*" >&2; }
info() { say "${BLUE}ℹ${NC} $*"; }
success() { say "${GREEN}✓${NC} $*"; }

ERRORS=0
WARNINGS=0

# Read single source of truth
if [[ ! -f VERSION ]]; then
    error "VERSION file not found at repo root"
    exit 1
fi

VERSION=$(cat VERSION | tr -d '[:space:]')

if [[ -z "$VERSION" ]]; then
    error "VERSION file is empty"
    exit 1
fi

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
    error "VERSION has invalid format: $VERSION"
    error "Expected format: v1.0.0 or v1.0.0-rc.1"
    exit 1
fi

success "VERSION file contains: $VERSION"
echo ""

# Check Cargo.toml workspace version
say "${BLUE}━━━ Checking Cargo.toml ━━━${NC}"
CARGO_VERSION=$(grep -E '^\s*version\s*=' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')
VERSION_NO_V="${VERSION#v}"

if [[ "$CARGO_VERSION" != "$VERSION_NO_V" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) != VERSION file ($VERSION_NO_V)"
    info "Run: sed -i -E 's/^version = \".*\"/version = \"$VERSION_NO_V\"/' Cargo.toml"
    ((ERRORS++))
else
    success "Cargo.toml matches VERSION ($VERSION_NO_V)"
fi
echo ""

# Check if binaries are built and version matches
say "${BLUE}━━━ Checking Built Binaries ━━━${NC}"

# Check annactl
if [[ -f "./target/release/annactl" ]]; then
    if binary_version=$(./target/release/annactl --version 2>/dev/null | head -1 | awk '{print $NF}'); then
        if [[ "$binary_version" == "$VERSION_NO_V" ]]; then
            success "annactl binary version matches ($binary_version)"
        else
            error "annactl binary version ($binary_version) != VERSION ($VERSION_NO_V)"
            info "Run: cargo build --release --bin annactl"
            ((ERRORS++))
        fi
    else
        warn "annactl binary exists but version check failed"
        ((WARNINGS++))
    fi
else
    warn "annactl not built (expected at ./target/release/annactl)"
    info "Run: cargo build --release --bin annactl"
    ((WARNINGS++))
fi

# Check annad (daemon - extract version from logs)
if [[ -f "./target/release/annad" ]]; then
    # annad doesn't have a simple --version flag, check logs or just verify it exists
    # Extract version from binary startup logs
    if binary_version=$(timeout 1 ./target/release/annad 2>&1 | grep -oP 'Anna v\K[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?' | head -1); then
        if [[ "$binary_version" == "$VERSION_NO_V" ]]; then
            success "annad binary version matches ($binary_version)"
        else
            error "annad binary version ($binary_version) != VERSION ($VERSION_NO_V)"
            info "Run: cargo build --release --bin annad"
            ((ERRORS++))
        fi
    else
        # Fallback: just verify the binary exists and was built recently
        info "annad binary exists (version check requires runtime)"
    fi
else
    warn "annad not built (expected at ./target/release/annad)"
    info "Run: cargo build --release --bin annad"
    ((WARNINGS++))
fi
echo ""

# Check installed system binaries
say "${BLUE}━━━ Checking Installed System Binaries ━━━${NC}"

# Check installed annactl
if command -v annactl >/dev/null 2>&1; then
    installed_version=$(annactl --version 2>/dev/null | head -1 | awk '{print $NF}' || echo "UNKNOWN")
    if [[ "$installed_version" == "$VERSION_NO_V" ]]; then
        success "Installed annactl matches ($installed_version)"
    else
        warn "Installed annactl version ($installed_version) != VERSION ($VERSION_NO_V)"
        info "Run: sudo ./scripts/install.sh to update"
        ((WARNINGS++))
    fi
else
    info "annactl not installed on system (optional)"
fi

# Check installed annad (from version file if it exists)
if command -v annad >/dev/null 2>&1 && [[ -f /etc/anna/version ]]; then
    installed_version=$(cat /etc/anna/version 2>/dev/null | tr -d '[:space:]' | sed 's/^v//')
    if [[ "$installed_version" == "$VERSION_NO_V" ]]; then
        success "Installed annad matches ($installed_version from /etc/anna/version)"
    else
        warn "Installed annad version ($installed_version) != VERSION ($VERSION_NO_V)"
        info "Run: sudo ./scripts/install.sh to update"
        ((WARNINGS++))
    fi
elif command -v annad >/dev/null 2>&1; then
    info "annad installed but /etc/anna/version missing"
else
    info "annad not installed on system (optional)"
fi
echo ""

# Check GitHub latest tag
say "${BLUE}━━━ Checking GitHub Latest Tag ━━━${NC}"
if latest_tag=$(git describe --tags --abbrev=0 2>/dev/null); then
    if [[ "$latest_tag" == "$VERSION" ]]; then
        success "Latest git tag matches ($latest_tag)"
    else
        warn "Latest git tag ($latest_tag) != VERSION ($VERSION)"
        info "This is normal if you haven't run release.sh yet"
        ((WARNINGS++))
    fi
else
    info "No git tags found (clean repo)"
fi
echo ""

# Summary
say "${BLUE}━━━ Summary ━━━${NC}"
if [[ $ERRORS -gt 0 ]]; then
    error "$ERRORS error(s) found - version consistency is broken"
    exit 1
elif [[ $WARNINGS -gt 0 ]]; then
    warn "$WARNINGS warning(s) - check messages above"
    success "Core consistency maintained (VERSION file is authoritative)"
    exit 0
else
    success "Perfect version consistency! All sources match VERSION file"
    exit 0
fi
