#!/bin/bash
# Anna Release Script
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 0.0.21
#
# This script:
# 1. Updates version in all Cargo.toml files
# 2. Cleans up tags/releases higher than target version
# 3. Runs tests
# 4. Commits, tags, and pushes
# 5. Creates GitHub release with built binaries

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Get version from argument or prompt
if [ -n "$1" ]; then
    VERSION="$1"
else
    CURRENT=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo -e "${BLUE}Current version: $CURRENT${NC}"
    read -p "Enter new version (or press Enter for $CURRENT): " VERSION
    VERSION="${VERSION:-$CURRENT}"
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "Invalid version format: $VERSION (expected X.Y.Z)"
fi

TAG="v$VERSION"
log_info "Preparing release $TAG"

# Parse version for comparison
IFS='.' read -r V_MAJOR V_MINOR V_PATCH <<< "$VERSION"

# Compare versions: returns 0 if $1 > $2
version_gt() {
    local a="$1" b="$2"
    a="${a#v}"; b="${b#v}"
    a=$(echo "$a" | sed 's/-.*$//')
    b=$(echo "$b" | sed 's/-.*$//')

    IFS='.' read -r a1 a2 a3 <<< "$a"
    IFS='.' read -r b1 b2 b3 <<< "$b"

    a1=${a1:-0}; a2=${a2:-0}; a3=${a3:-0}
    b1=${b1:-0}; b2=${b2:-0}; b3=${b3:-0}

    (( a1 > b1 )) && return 0
    (( a1 < b1 )) && return 1
    (( a2 > b2 )) && return 0
    (( a2 < b2 )) && return 1
    (( a3 > b3 )) && return 0
    return 1
}

# Step 1: Update version in Cargo.toml files
log_info "Updating Cargo.toml files..."
sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" Cargo.toml
for toml in crates/*/Cargo.toml; do
    if grep -q '^version = ' "$toml" 2>/dev/null; then
        sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$toml"
    fi
done
log_success "Updated Cargo.toml files"

# Step 2: Update VERSION file if it exists
if [ -f VERSION ]; then
    echo "$VERSION" > VERSION
    log_success "Updated VERSION file"
fi

# Step 3: Update install.sh VERSION if present
if [ -f scripts/install.sh ]; then
    sed -i "s/^VERSION=\"[^\"]*\"/VERSION=\"$VERSION\"/" scripts/install.sh
    log_success "Updated install.sh"
fi

# Find cargo
CARGO="${CARGO:-$(which cargo 2>/dev/null || echo "$HOME/.cargo/bin/cargo")}"
if [ ! -x "$CARGO" ]; then
    log_error "cargo not found. Install Rust first."
fi

# Step 4: Rebuild to update Cargo.lock
log_info "Building..."
"$CARGO" build --release --workspace --quiet
log_success "Build complete"

# Step 5: Run tests
log_info "Running tests..."
"$CARGO" test --workspace --quiet || log_error "Tests failed!"
log_success "Tests passed"

# Step 6: Clean up orphaned tags (versions > current)
log_info "Cleaning up orphaned tags..."
TAGS_DELETED=0
for tag in $(git tag -l 'v*'); do
    if version_gt "$tag" "$TAG"; then
        git tag -d "$tag" 2>/dev/null || true
        git push origin --delete "$tag" 2>/dev/null || true
        ((TAGS_DELETED++)) || true
    fi
done
[ $TAGS_DELETED -gt 0 ] && log_success "Deleted $TAGS_DELETED orphaned tags"

# Step 7: Clean up orphaned releases
log_info "Cleaning up orphaned releases..."
RELEASES_DELETED=0
for rel in $(gh release list --json tagName -q '.[].tagName' 2>/dev/null); do
    if version_gt "$rel" "$TAG"; then
        gh release delete "$rel" --yes 2>/dev/null || true
        ((RELEASES_DELETED++)) || true
    fi
done
[ $RELEASES_DELETED -gt 0 ] && log_success "Deleted $RELEASES_DELETED orphaned releases"

# Step 8: Delete current tag/release if exists (for re-release)
git tag -d "$TAG" 2>/dev/null || true
git push origin --delete "$TAG" 2>/dev/null || true
gh release delete "$TAG" --yes 2>/dev/null || true

# Step 9: Stage and commit
log_info "Committing changes..."
git add -A
if ! git diff --cached --quiet; then
    git commit -m "$TAG: Release

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
    log_success "Changes committed"
else
    log_info "No changes to commit"
fi

# Step 10: Create tag
log_info "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"
log_success "Tag created"

# Step 11: Push
log_info "Pushing to origin..."
git push origin main --force-with-lease
git push origin "$TAG"
log_success "Pushed to origin"

# Step 12: Create GitHub release
log_info "Creating GitHub release..."
NOTES="## Anna $TAG

### Installation
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/install.sh | sudo bash
\`\`\`

### Binaries
- \`annad\` - Anna daemon
- \`annactl\` - Anna CLI client

---
*Release created automatically*"

gh release create "$TAG" \
    --title "Anna $TAG" \
    --notes "$NOTES" \
    target/release/annad \
    target/release/annactl

log_success "GitHub release created"

echo ""
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  Release $TAG complete!${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""
echo "URL: https://github.com/jjgarcianorway/anna-assistant/releases/tag/$TAG"
echo ""
