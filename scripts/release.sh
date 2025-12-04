#!/bin/bash
# Anna Release Script v1.0.0
#
# Usage: ./scripts/release.sh [VERSION]
#
# If VERSION is omitted, uses version from Cargo.toml.
# This script:
#   1. Verifies working tree is clean
#   2. Verifies Cargo.toml version matches (or use provided VERSION)
#   3. Verifies all docs are updated (README, CLAUDE.md, TODO.md, RELEASE_NOTES.md)
#   4. Runs tests
#   5. Creates annotated git tag
#   6. Pushes commit and tag to origin
#   7. Waits for CI release workflow to start
#   8. Provides link to monitor release
#
# The GitHub Actions release.yml workflow handles:
#   - Building binaries in Arch Linux container
#   - Creating GitHub release with artifacts
#   - Uploading checksums
#
# After release completes, the curl installer will automatically see the new version.

set -euo pipefail

# Colors
RED=$'\033[31m'
GREEN=$'\033[32m'
YELLOW=$'\033[33m'
BLUE=$'\033[34m'
BOLD=$'\033[1m'
NC=$'\033[0m'

REPO="jjgarcianorway/anna-assistant"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$REPO_ROOT"

log_info() { echo "${BLUE}[INFO]${NC} $1"; }
log_ok() { echo "${GREEN}[OK]${NC} $1"; }
log_warn() { echo "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo "${RED}[ERROR]${NC} $1"; }
log_step() { echo "${BOLD}==> $1${NC}"; }

# =============================================================================
# Parse Arguments
# =============================================================================

if [[ $# -ge 1 ]]; then
    VERSION="$1"
else
    VERSION=""
fi

# =============================================================================
# Step 1: Verify working tree is clean
# =============================================================================

log_step "Checking working tree status..."

if ! git diff --quiet HEAD 2>/dev/null; then
    log_error "Working tree has uncommitted changes."
    echo ""
    git status --short
    echo ""
    log_error "Commit or stash changes before releasing."
    exit 1
fi

log_ok "Working tree is clean"

# =============================================================================
# Step 2: Extract and verify version
# =============================================================================

log_step "Checking version..."

CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
log_info "Cargo.toml version: $CARGO_VERSION"

if [[ -n "$VERSION" ]]; then
    if [[ "$VERSION" != "$CARGO_VERSION" ]]; then
        log_error "Provided version ($VERSION) does not match Cargo.toml ($CARGO_VERSION)"
        log_error "Either update Cargo.toml or use: ./scripts/release.sh"
        exit 1
    fi
else
    VERSION="$CARGO_VERSION"
fi

log_ok "Version: $VERSION"

# =============================================================================
# Step 3: Verify documentation is updated
# =============================================================================

log_step "Verifying documentation..."

# Check README.md
if ! grep -q "Anna Assistant v$VERSION" README.md; then
    log_error "README.md not updated for v$VERSION"
    log_error "Expected: 'Anna Assistant v$VERSION' in README.md"
    exit 1
fi
log_ok "README.md updated"

# Check CLAUDE.md
if ! grep -q "Version: $VERSION" CLAUDE.md; then
    log_error "CLAUDE.md not updated for v$VERSION"
    log_error "Expected: 'Version: $VERSION' in CLAUDE.md"
    exit 1
fi
log_ok "CLAUDE.md updated"

# Check TODO.md
TODO_VERSION=$(grep -oP 'Current Version: \K[\d.]+' TODO.md || echo "not found")
if [[ "$TODO_VERSION" != "$VERSION" ]]; then
    log_error "TODO.md version ($TODO_VERSION) does not match $VERSION"
    log_error "Update 'Current Version: $VERSION' in TODO.md"
    exit 1
fi
log_ok "TODO.md updated"

# Check RELEASE_NOTES.md
if ! grep -q "## v$VERSION" RELEASE_NOTES.md; then
    log_error "RELEASE_NOTES.md missing entry for v$VERSION"
    log_error "Add '## v$VERSION' section to RELEASE_NOTES.md"
    exit 1
fi
log_ok "RELEASE_NOTES.md updated"

# Check SPEC.md exists
if [[ ! -f "SPEC.md" ]]; then
    log_error "SPEC.md not found - authoritative specification required"
    exit 1
fi
log_ok "SPEC.md exists"

# Check SPEC.md version
if ! grep -q "# Anna Specification v$VERSION" SPEC.md; then
    log_warn "SPEC.md header version mismatch (expected v$VERSION)"
    log_info "Update SPEC.md header to: # Anna Specification v$VERSION"
    # Advisory only - don't block release for this
fi

# =============================================================================
# Step 4: Run tests
# =============================================================================

log_step "Running tests..."

if ! cargo test --workspace 2>&1; then
    log_error "Tests failed. Fix tests before releasing."
    exit 1
fi

log_ok "All tests pass"

# =============================================================================
# Step 5: Check if tag already exists
# =============================================================================

log_step "Checking for existing tag..."

TAG="v$VERSION"

if git rev-parse "$TAG" >/dev/null 2>&1; then
    log_warn "Tag $TAG already exists locally"

    # Check if it's on remote
    if git ls-remote --tags origin "$TAG" | grep -q "$TAG"; then
        log_error "Tag $TAG already exists on remote!"
        log_error "If you need to re-release, delete the tag first:"
        log_error "  git tag -d $TAG"
        log_error "  git push origin :refs/tags/$TAG"
        exit 1
    else
        log_info "Tag exists locally but not on remote. Will push."
    fi
else
    log_step "Creating tag $TAG..."

    # Create annotated tag
    git tag -a "$TAG" -m "Release $TAG

$(grep -A 50 "## v$VERSION" RELEASE_NOTES.md | head -20)
"
    log_ok "Tag $TAG created"
fi

# =============================================================================
# Step 6: Push to remote
# =============================================================================

log_step "Pushing to origin..."

# Get current branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Push branch
git push origin "$BRANCH"
log_ok "Pushed branch $BRANCH"

# Push tag
git push origin "$TAG"
log_ok "Pushed tag $TAG"

# =============================================================================
# Step 7: Provide release monitoring info
# =============================================================================

echo ""
echo "${BOLD}════════════════════════════════════════════════════════════${NC}"
echo "${GREEN}Release $TAG initiated!${NC}"
echo "${BOLD}════════════════════════════════════════════════════════════${NC}"
echo ""
echo "The GitHub Actions release workflow is now running."
echo ""
echo "Monitor progress:"
echo "  ${BLUE}https://github.com/$REPO/actions${NC}"
echo ""
echo "Once complete, the release will be available at:"
echo "  ${BLUE}https://github.com/$REPO/releases/tag/$TAG${NC}"
echo ""
echo "Users can then install with:"
echo "  ${YELLOW}curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | bash${NC}"
echo ""
echo "The installer automatically fetches the latest release (now $TAG)."
echo ""

# =============================================================================
# Step 8: Optional - wait and verify
# =============================================================================

read -p "Wait for release to complete and verify? [y/N] " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_step "Waiting for release workflow..."

    # Give CI a moment to start
    sleep 5

    # Check workflow status (requires gh CLI)
    if command -v gh &>/dev/null; then
        log_info "Checking workflow status..."
        gh run list --repo "$REPO" --limit 3

        echo ""
        log_info "Waiting for release workflow to complete (this may take a few minutes)..."

        # Wait for the release workflow
        if gh run watch --repo "$REPO" --exit-status 2>/dev/null; then
            log_ok "Release workflow completed successfully!"

            # Verify release exists
            echo ""
            log_step "Verifying release..."

            RELEASE_INFO=$(gh release view "$TAG" --repo "$REPO" 2>/dev/null || echo "")

            if [[ -n "$RELEASE_INFO" ]]; then
                log_ok "Release $TAG is live!"
                echo ""
                gh release view "$TAG" --repo "$REPO"
            else
                log_warn "Release not found yet. Check GitHub manually."
            fi
        else
            log_error "Release workflow failed. Check GitHub Actions."
        fi
    else
        log_warn "GitHub CLI (gh) not installed. Monitor release manually at:"
        log_info "https://github.com/$REPO/actions"
    fi
fi

echo ""
log_ok "Done!"
