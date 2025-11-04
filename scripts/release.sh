#!/usr/bin/env bash
# Anna Auto-Release Script - Zero Interaction Required
set -Eeuo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

OWNER="jjgarcianorway"
REPO="anna-assistant"

# Simple output
info() { echo "→ $*"; }
success() { echo "✓ $*"; }
error() { echo "✗ ERROR: $*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || error "Missing: $1"; }
require git
require curl
require jq
require cargo

echo ""
echo "═══════════════════════════════════════════"
echo "  Anna Auto-Release (Zero Interaction)"
echo "═══════════════════════════════════════════"
echo ""

# Read VERSION file
[[ -f VERSION ]] || error "VERSION file not found"
VER=$(cat VERSION | tr -d '[:space:]')
[[ -n "$VER" ]] || error "VERSION file is empty"
[[ "$VER" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]] || error "Invalid VERSION format: $VER"

info "Target version: $VER"

# Check if release already complete
info "Checking if release already exists..."
if git ls-remote --tags origin | grep -q "refs/tags/$VER$"; then
    release_json=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases/tags/$VER" 2>/dev/null || echo '{}')
    has_tarball=$(echo "$release_json" | jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz") | .name' 2>/dev/null || true)

    if [[ "$has_tarball" == "anna-linux-x86_64.tar.gz" ]]; then
        success "Release $VER already complete - nothing to do"
        echo ""
        echo "Release URL: https://github.com/$OWNER/$REPO/releases/tag/$VER"
        exit 0
    fi
fi

# Auto-fix Cargo.toml if needed
info "Syncing Cargo.toml with VERSION..."
VERSION_NO_V="${VER#v}"
CARGO_VERSION=$(grep -E '^\s*version\s*=' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')

if [[ "$CARGO_VERSION" != "$VERSION_NO_V" ]]; then
    info "Updating Cargo.toml from $CARGO_VERSION to $VERSION_NO_V"
    sed -i -E "s/^version = \".*\"/version = \"$VERSION_NO_V\"/" Cargo.toml
fi

# Build and verify
info "Building binaries..."
if ! cargo build --release --bin annad --bin annactl 2>&1 | tail -3; then
    error "Build failed"
fi

info "Verifying versions..."
annactl_ver=$(./target/release/annactl --version 2>/dev/null | awk '{print $NF}' || echo "UNKNOWN")
[[ "$annactl_ver" == "$VERSION_NO_V" ]] || error "annactl version mismatch: $annactl_ver != $VERSION_NO_V"

success "Build complete, versions verified"

# Auto-commit if there are changes
info "Checking for uncommitted changes..."
if [[ -n "$(git status --porcelain)" ]]; then
    info "Auto-committing changes..."
    git add -A
    git commit -m "chore(release): prepare release $VER

- Updated VERSION to $VER
- Synced Cargo.toml
- Auto-commit by release script

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
    success "Changes committed"
fi

# Create and push tag
if ! git rev-parse "$VER" >/dev/null 2>&1; then
    info "Creating tag $VER..."
    git tag -a "$VER" -m "Release $VER

Automated release

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

    info "Pushing commit and tag..."
    git push origin HEAD:main --tags || error "Push failed"
    success "Tag $VER pushed"
else
    info "Tag $VER already exists locally"
    info "Pushing to ensure remote is synced..."
    git push origin HEAD:main --tags || true
fi

echo ""
echo "══════════════════════════════════════════════"
echo "  ✓ Release $VER Complete!"
echo "══════════════════════════════════════════════"
echo ""
echo "GitHub Actions is building the release assets now."
echo "Check: https://github.com/$OWNER/$REPO/actions"
echo ""
echo "When ready (2-3 minutes), install with:"
echo "  sudo ./scripts/install.sh"
echo ""
