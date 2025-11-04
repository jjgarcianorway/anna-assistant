#!/usr/bin/env bash
# Anna Version Verification Script
# Single Source of Truth: VERSION file
# Verifies consistency across source, binaries, and GitHub
set -Eeuo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
RESET='\033[0m'

OWNER="jjgarcianorway"
REPO="anna-assistant"
ERRORS=0
WARNINGS=0

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}║               Anna Version Verification System                        ║${RESET}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
echo ""

# Helper Functions
error() {
    echo -e "${RED}✗ ERROR:${RESET} $*" >&2
    ERRORS=$((ERRORS + 1))
}

warn() {
    echo -e "${YELLOW}⚠ WARNING:${RESET} $*"
    WARNINGS=$((WARNINGS + 1))
}

success() {
    echo -e "${GREEN}✓${RESET} $*"
}

info() {
    echo -e "${BLUE}ℹ${RESET} $*"
}

section() {
    echo ""
    echo -e "${CYAN}━━━ $* ${RESET}"
    echo ""
}

# Read Single Source of Truth
section "📋 Version Information"

if [[ ! -f VERSION ]]; then
    error "VERSION file not found at repo root"
    echo ""
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${RED}║              ✗ VERSION File Missing - Fatal Error                    ║${RESET}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    exit 1
fi

VERSION=$(cat VERSION | tr -d '[:space:]')
VERSION_NO_V="${VERSION#v}"

info "Source (VERSION file):   $VERSION"

# Cargo.toml version
CARGO_VERSION=$(grep -E '^\s*version\s*=' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')
info "Source (Cargo.toml):     $CARGO_VERSION"

# Installed system version
if command -v annactl >/dev/null 2>&1; then
    INSTALLED_VERSION=$(annactl --version 2>/dev/null | awk '{print $NF}' || echo "UNKNOWN")
    info "Installed (system):      $INSTALLED_VERSION"
else
    INSTALLED_VERSION="NOT_INSTALLED"
    info "Installed (system):      NOT_INSTALLED"
fi

# Local build version
if [[ -f ./target/release/annactl ]]; then
    LOCAL_VERSION=$(./target/release/annactl --version 2>/dev/null | awk '{print $NF}' || echo "UNKNOWN")
    info "Local build:             $LOCAL_VERSION"
else
    LOCAL_VERSION="NOT_BUILT"
    info "Local build:             NOT_BUILT"
fi

# Latest GitHub tag
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/git/refs/tags" 2>/dev/null | \
             jq -r '.[].ref' | \
             sed 's|refs/tags/||' | \
             grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$' | \
             sort -V | \
             tail -n1 || echo "UNKNOWN")
info "Latest GitHub tag:       ${LATEST_TAG:-UNKNOWN}"

# Latest GitHub release with assets
LATEST_RELEASE=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases?per_page=10" 2>/dev/null | \
                 jq -r '.[] | select(.draft==false) | select(.assets[] | .name=="anna-linux-x86_64.tar.gz") | .tag_name' | \
                 sort -Vr | \
                 head -n1 || echo "UNKNOWN")
info "Latest GitHub release:   ${LATEST_RELEASE:-UNKNOWN}"

# Validation Checks
section "🔍 Validation Checks"

# Check 1: Cargo.toml must match VERSION file
if [[ "$CARGO_VERSION" != "$VERSION_NO_V" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) != VERSION file ($VERSION_NO_V)"
    info "Run: make bump VERSION=$VERSION  (to fix)"
else
    success "Source version matches VERSION file"
fi

# Check 2: Installed version (if present)
if [[ "$INSTALLED_VERSION" == "NOT_INSTALLED" ]]; then
    warn "Anna is not installed on this system"
    info "Run: sudo ./scripts/install.sh"
elif [[ "$INSTALLED_VERSION" != "$VERSION_NO_V" ]]; then
    warn "Installed version ($INSTALLED_VERSION) != VERSION file ($VERSION_NO_V)"
    info "Run: sudo ./scripts/install.sh  (to upgrade)"
else
    success "Installed version matches VERSION file"
fi

# Check 3: Local build (if present)
if [[ "$LOCAL_VERSION" == "NOT_BUILT" ]]; then
    warn "No local build found in ./target/release/"
    info "Run: cargo build --release"
elif [[ "$LOCAL_VERSION" != "$VERSION_NO_V" ]]; then
    error "Local build version ($LOCAL_VERSION) != VERSION file ($VERSION_NO_V)"
    info "Run: cargo clean && cargo build --release"
else
    success "Local build version matches VERSION file"
fi

# Check 4: Latest GitHub tag
if [[ "$LATEST_TAG" == "$VERSION" ]]; then
    success "VERSION file matches latest GitHub tag"
elif [[ "$LATEST_TAG" == "UNKNOWN" ]]; then
    warn "Could not fetch GitHub tags (network issue or no tags exist)"
else
    warn "VERSION file ($VERSION) != latest GitHub tag ($LATEST_TAG)"
    info "This is normal if you haven't released $VERSION yet"
fi

# Check 5: GitHub release with assets
if [[ "$LATEST_RELEASE" == "$VERSION" ]]; then
    success "VERSION file matches latest published release with assets"
elif [[ "$LATEST_RELEASE" == "UNKNOWN" ]]; then
    warn "Could not fetch GitHub releases (network issue or no releases exist)"
else
    warn "VERSION file ($VERSION) != latest release with assets ($LATEST_RELEASE)"
    if [[ "$LATEST_TAG" == "$VERSION" ]]; then
        info "GitHub Actions may still be building, or the build failed"
        info "Check: https://github.com/$OWNER/$REPO/actions"
    else
        info "Run: make release  (to create $VERSION release)"
    fi
fi

# Summary
section "📊 Summary"

if [[ $ERRORS -gt 0 ]]; then
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${RED}║              ✗ Version Validation Failed ($ERRORS errors)                  ║${RESET}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    info "Fix errors above before releasing"
    exit 1
elif [[ $WARNINGS -gt 0 ]]; then
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${YELLOW}║        ⚠ Version Validation Passed with $WARNINGS Warnings                ║${RESET}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    info "VERSION file ($VERSION) is the authoritative source"
    info "Warnings are normal during development"
    exit 0
else
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║              ✓ Perfect Version Consistency!                           ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    info "All versions match VERSION file: $VERSION"
    exit 0
fi
