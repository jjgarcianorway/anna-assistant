#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Anna Version Verification Script
# Ensures all versions are in sync: source, installed, and GitHub releases
# ═══════════════════════════════════════════════════════════════════════════
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

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}║               Anna Version Verification System                        ║${RESET}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Helper Functions
# ═══════════════════════════════════════════════════════════════════════════

error() {
    echo -e "${RED}✗ ERROR:${RESET} $*" >&2
    ERRORS=$((ERRORS + 1))
}

warn() {
    echo -e "${YELLOW}⚠ WARNING:${RESET} $*"
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

# ═══════════════════════════════════════════════════════════════════════════
# Version Extraction Functions
# ═══════════════════════════════════════════════════════════════════════════

get_source_version() {
    local cargo_version
    cargo_version=$(grep -E '^version = ".*"' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')
    echo "$cargo_version"
}

get_installed_version() {
    if command -v annactl >/dev/null 2>&1; then
        annactl --version 2>/dev/null | grep -oP 'v?[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?' | head -1
    else
        echo "NOT_INSTALLED"
    fi
}

get_local_build_version() {
    if [[ -f ./target/release/annactl ]]; then
        ./target/release/annactl --version 2>/dev/null | grep -oP 'v?[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?' | head -1 || echo "NOT_BUILT"
    else
        echo "NOT_BUILT"
    fi
}

get_latest_github_tag() {
    curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/git/refs/tags" 2>/dev/null | \
        jq -r '.[].ref' | \
        sed 's|refs/tags/||' | \
        grep -E '^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$' | \
        sort -V | \
        tail -n1 | \
        sed 's/^v//' || echo "UNKNOWN"
}

get_latest_github_release_with_assets() {
    local api="https://api.github.com/repos/$OWNER/$REPO/releases?per_page=10"
    curl -fsSL "$api" 2>/dev/null | \
        jq -r '.[] | select(.draft==false) | select(.assets[] | .name=="anna-linux-x86_64.tar.gz") | .tag_name' | \
        sort -Vr | \
        head -n1 | \
        sed 's/^v//' || echo "UNKNOWN"
}

# ═══════════════════════════════════════════════════════════════════════════
# Version Comparison
# ═══════════════════════════════════════════════════════════════════════════

versions_match() {
    local v1="$1"
    local v2="$2"

    # Remove 'v' prefix if present
    v1="${v1#v}"
    v2="${v2#v}"

    [[ "$v1" == "$v2" ]]
}

# ═══════════════════════════════════════════════════════════════════════════
# Main Validation
# ═══════════════════════════════════════════════════════════════════════════

section "📋 Version Information"

SOURCE_VERSION=$(get_source_version)
INSTALLED_VERSION=$(get_installed_version)
LOCAL_BUILD_VERSION=$(get_local_build_version)
GITHUB_TAG=$(get_latest_github_tag)
GITHUB_RELEASE=$(get_latest_github_release_with_assets)

info "Source (Cargo.toml):       ${CYAN}$SOURCE_VERSION${RESET}"
info "Installed (system):        ${CYAN}$INSTALLED_VERSION${RESET}"
info "Local build:               ${CYAN}$LOCAL_BUILD_VERSION${RESET}"
info "Latest GitHub tag:         ${CYAN}$GITHUB_TAG${RESET}"
info "Latest GitHub release:     ${CYAN}$GITHUB_RELEASE${RESET}"

section "🔍 Validation Checks"

# Check 1: Source vs Installed
if [[ "$INSTALLED_VERSION" == "NOT_INSTALLED" ]]; then
    warn "Anna is not installed on this system"
    info "Run: sudo ./scripts/install.sh"
elif versions_match "$SOURCE_VERSION" "$INSTALLED_VERSION"; then
    success "Installed version matches source version"
else
    error "Installed version ($INSTALLED_VERSION) does not match source ($SOURCE_VERSION)"
    info "Run: sudo ./scripts/install.sh  # To upgrade to latest"
fi

# Check 2: Source vs Local Build
if [[ "$LOCAL_BUILD_VERSION" == "NOT_BUILT" ]]; then
    warn "No local build found in ./target/release/"
    info "Run: cargo build --release"
elif versions_match "$SOURCE_VERSION" "$LOCAL_BUILD_VERSION"; then
    success "Local build matches source version"
else
    error "Local build ($LOCAL_BUILD_VERSION) is outdated (source: $SOURCE_VERSION)"
    info "Run: cargo build --release  # Rebuild with latest version"
fi

# Check 3: Source vs GitHub Tag
if [[ "$GITHUB_TAG" == "UNKNOWN" ]]; then
    warn "Could not fetch GitHub tags (network issue?)"
elif versions_match "$SOURCE_VERSION" "$GITHUB_TAG"; then
    success "Source version matches latest GitHub tag"
else
    error "Source version ($SOURCE_VERSION) does not match latest tag (v$GITHUB_TAG)"
    if [[ "$SOURCE_VERSION" > "$GITHUB_TAG" ]]; then
        info "Source is newer - you may need to run: ./scripts/release.sh"
    else
        info "Source is older - you may need to pull latest changes"
    fi
fi

# Check 4: GitHub Tag has Assets
if [[ "$GITHUB_RELEASE" == "UNKNOWN" ]]; then
    warn "Could not fetch GitHub releases (network issue?)"
elif versions_match "$GITHUB_TAG" "$GITHUB_RELEASE"; then
    success "Latest GitHub tag has release assets"
else
    error "Latest tag (v$GITHUB_TAG) does not have release assets"
    error "Latest release with assets: v$GITHUB_RELEASE"
    info "GitHub Actions may still be building, or the build failed"
    info "Check: https://github.com/$OWNER/$REPO/actions"
fi

# Check 5: Local Build vs Installed (if both exist)
if [[ "$LOCAL_BUILD_VERSION" != "NOT_BUILT" && "$INSTALLED_VERSION" != "NOT_INSTALLED" ]]; then
    if versions_match "$LOCAL_BUILD_VERSION" "$INSTALLED_VERSION"; then
        success "Local build matches installed version"
    else
        warn "Local build ($LOCAL_BUILD_VERSION) differs from installed ($INSTALLED_VERSION)"
        info "This is normal if you're testing a new version"
    fi
fi

# ═══════════════════════════════════════════════════════════════════════════
# Summary and Exit
# ═══════════════════════════════════════════════════════════════════════════

section "📊 Summary"

if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║                  ✓ All Version Checks Passed                          ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${RED}║              ✗ Version Validation Failed ($ERRORS errors)                  ║${RESET}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""

    section "🔧 Recommended Actions"

    if [[ "$INSTALLED_VERSION" != "NOT_INSTALLED" ]] && ! versions_match "$SOURCE_VERSION" "$INSTALLED_VERSION"; then
        echo "1. Upgrade your installation:"
        echo "   ${CYAN}sudo ./scripts/install.sh${RESET}"
        echo ""
    fi

    if [[ "$LOCAL_BUILD_VERSION" != "NOT_BUILT" ]] && ! versions_match "$SOURCE_VERSION" "$LOCAL_BUILD_VERSION"; then
        echo "2. Rebuild local binaries:"
        echo "   ${CYAN}cargo build --release${RESET}"
        echo ""
    fi

    if [[ "$GITHUB_TAG" != "UNKNOWN" ]] && [[ "$SOURCE_VERSION" > "$GITHUB_TAG" ]]; then
        echo "3. Create a new release:"
        echo "   ${CYAN}./scripts/release.sh${RESET}"
        echo ""
    fi

    exit 1
fi
