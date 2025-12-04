#!/bin/bash
# Anna Release Script
# Creates a complete release: build, tag, push, GitHub release, artifacts

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$REPO_ROOT/target/release"
ARTIFACTS_DIR="$REPO_ROOT/artifacts"

# Required files
REQUIRED_FILES=(
    "VERSION"
    "SPEC.md"
    "CHANGELOG.md"
    "RELEASE_CONTRACT.md"
    "scripts/install.sh"
    "scripts/uninstall.sh"
    "docs/UPDATE_PROTOCOL.md"
)

cd "$REPO_ROOT"

# Check for clean working tree
check_clean() {
    log_step "Checking working tree..."

    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working tree is dirty. Commit or stash changes first."
        git status --short
        exit 1
    fi

    log_info "Working tree is clean"
}

# Read and validate version
read_version() {
    log_step "Reading version..."

    if [ ! -f VERSION ]; then
        log_error "VERSION file not found"
        exit 1
    fi

    VERSION=$(cat VERSION | tr -d '[:space:]')

    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "Invalid version format: $VERSION (expected X.Y.Z)"
        exit 1
    fi

    TAG="v$VERSION"
    log_info "Version: $VERSION (tag: $TAG)"
}

# Check required files exist
check_required_files() {
    log_step "Checking required files..."

    local missing=0
    for file in "${REQUIRED_FILES[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Missing required file: $file"
            missing=1
        fi
    done

    if [ $missing -eq 1 ]; then
        exit 1
    fi

    log_info "All required files present"
}

# Check CHANGELOG has entry for this version
check_changelog() {
    log_step "Checking CHANGELOG..."

    if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
        log_error "CHANGELOG.md missing entry for version $VERSION"
        log_error "Add a section: ## [$VERSION] - $(date +%Y-%m-%d)"
        exit 1
    fi

    log_info "CHANGELOG has entry for $VERSION"
}

# Check version consistency
check_version_consistency() {
    log_step "Checking version consistency..."

    # Check install.sh
    local install_version=$(grep '^VERSION=' scripts/install.sh | cut -d'"' -f2)
    if [ "$install_version" != "$VERSION" ]; then
        log_error "scripts/install.sh VERSION=$install_version doesn't match $VERSION"
        exit 1
    fi

    log_info "Version is consistent across files"
}

# Check tag doesn't already exist
check_tag() {
    log_step "Checking tag..."

    if git rev-parse "$TAG" >/dev/null 2>&1; then
        log_error "Tag $TAG already exists"
        log_error "If re-releasing, delete the tag first: git tag -d $TAG && git push origin :$TAG"
        exit 1
    fi

    log_info "Tag $TAG is available"
}

# Build release binaries
build_binaries() {
    log_step "Building release binaries..."

    cargo build --release

    mkdir -p "$ARTIFACTS_DIR"

    # Copy binaries
    cp "$BUILD_DIR/annad" "$ARTIFACTS_DIR/annad-linux-x86_64"
    cp "$BUILD_DIR/annactl" "$ARTIFACTS_DIR/annactl-linux-x86_64"

    log_info "Binaries built: $ARTIFACTS_DIR/"
}

# Generate checksums
generate_checksums() {
    log_step "Generating checksums..."

    cd "$ARTIFACTS_DIR"
    sha256sum annad-* annactl-* > SHA256SUMS
    cd "$REPO_ROOT"

    log_info "Checksums generated"
    cat "$ARTIFACTS_DIR/SHA256SUMS"
}

# Create release commit
create_commit() {
    log_step "Creating release commit..."

    git add -A
    git commit --allow-empty -m "release v$VERSION"

    log_info "Release commit created"
}

# Create annotated tag
create_tag() {
    log_step "Creating annotated tag..."

    # Extract release notes from CHANGELOG
    local notes=$(sed -n "/## \[$VERSION\]/,/## \[/p" CHANGELOG.md | head -n -1)

    git tag -a "$TAG" -m "Release $VERSION" -m "$notes"

    log_info "Tag $TAG created"
}

# Push to remote
push_to_remote() {
    log_step "Pushing to remote..."

    git push origin main
    git push origin "$TAG"

    log_info "Pushed commit and tag to origin"
}

# Create GitHub release
create_github_release() {
    log_step "Creating GitHub release..."

    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) not found!"
        echo
        echo "To complete the release manually:"
        echo "1. Go to https://github.com/jjgarcianorway/anna-assistant/releases/new"
        echo "2. Choose tag: $TAG"
        echo "3. Title: $TAG"
        echo "4. Upload artifacts from: $ARTIFACTS_DIR/"
        echo "5. Copy release notes from CHANGELOG.md"
        echo
        exit 1
    fi

    # Check if authenticated
    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI not authenticated. Run: gh auth login"
        exit 1
    fi

    # Extract release notes
    local notes=$(sed -n "/## \[$VERSION\]/,/## \[/p" CHANGELOG.md | head -n -1 | tail -n +2)

    # Create release and upload artifacts
    gh release create "$TAG" \
        --title "$TAG" \
        --notes "$notes" \
        "$ARTIFACTS_DIR/annad-linux-x86_64" \
        "$ARTIFACTS_DIR/annactl-linux-x86_64" \
        "$ARTIFACTS_DIR/SHA256SUMS"

    log_info "GitHub release created"
}

# Print summary
print_summary() {
    echo
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Release $TAG completed successfully!${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo
    echo "Summary:"
    echo "  - Version: $VERSION"
    echo "  - Tag: $TAG"
    echo "  - Commit: $(git rev-parse --short HEAD)"
    echo "  - Artifacts: $ARTIFACTS_DIR/"
    echo
    echo "Artifacts uploaded:"
    ls -la "$ARTIFACTS_DIR/"
    echo
    echo "Release URL:"
    echo "  https://github.com/jjgarcianorway/anna-assistant/releases/tag/$TAG"
    echo
}

# Main release flow
main() {
    echo
    echo "════════════════════════════════════════"
    echo "        Anna Release Script"
    echo "════════════════════════════════════════"
    echo

    check_clean
    read_version
    check_required_files
    check_changelog
    check_version_consistency
    check_tag
    build_binaries
    generate_checksums
    create_commit
    create_tag
    push_to_remote
    create_github_release
    print_summary
}

main "$@"
