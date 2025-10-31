#!/usr/bin/env bash
# Anna Assistant Release Automation Script
# Usage: ./scripts/release.sh [OPTIONS]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
RELEASE_TYPE=""
COMMIT_MSG=""
VERSION=""
DRY_RUN=false

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Helper functions
print_header() {
    echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│  Anna Assistant Release Script          │${NC}"
    echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}→ $1${NC}"
}

show_help() {
    cat <<EOF
Usage: ./scripts/release.sh [OPTIONS]

Automates the release process for Anna Assistant:
  - Detects current version from Cargo.toml
  - Bumps version (major/minor/patch)
  - Updates version in all files
  - Creates git commit and tag
  - Pushes to GitHub (triggers CI build)

OPTIONS:
  -t, --type TYPE       Release type: major, minor, patch (required)
  -m, --message MSG     Commit message (required)
  -v, --version VER     Explicit version (overrides auto-increment)
  -d, --dry-run         Show what would be done without making changes
  -h, --help            Show this help message

SEMANTIC VERSIONING (-t parameter):
  Anna uses semantic versioning: MAJOR.MINOR.PATCH

  -t patch    Bump patch version (bug fixes, small changes)
              Example: 0.11.1 -> 0.11.2
              Use for: Bug fixes, documentation updates, minor improvements

  -t minor    Bump minor version (new features, backwards-compatible)
              Example: 0.11.2 -> 0.12.0
              Use for: New features, enhancements, non-breaking changes

  -t major    Bump major version (breaking changes)
              Example: 0.12.0 -> 1.0.0
              Use for: Breaking changes, major rewrites, API changes

EXAMPLES:
  # Patch release: Bug fixes (0.11.1 -> 0.11.2)
  ./scripts/release.sh -t patch -m "Fix installation bug"

  # Minor release: New features (0.11.1 -> 0.12.0)
  ./scripts/release.sh -t minor -m "Add new telemetry features"

  # Major release: Breaking changes (0.11.1 -> 1.0.0)
  ./scripts/release.sh -t major -m "Stable release with breaking changes"

  # Explicit version (bypass semantic versioning)
  ./scripts/release.sh -v 0.11.2 -m "Hotfix for critical bug"

  # Dry run (preview changes without making them)
  ./scripts/release.sh -t patch -m "Test" --dry-run

RELEASE PROCESS:
  1. Checks git status (must be clean)
  2. Detects current version from Cargo.toml
  3. Bumps version based on type
  4. Updates version in all files
  5. Creates git commit
  6. Creates git tag
  7. Pushes to GitHub
  8. Triggers GitHub Actions (builds binaries)

FILES UPDATED:
  - Cargo.toml (workspace.package.version)
  - packaging/aur/PKGBUILD (pkgver)
  - packaging/aur/PKGBUILD-bin (pkgver)

NOTE:
  The installer (install.sh) automatically fetches the LATEST release from
  GitHub, so it doesn't need version updates. After you create a release,
  the installer will automatically download that version!

AFTER RELEASE:
  - Monitor GitHub Actions: https://github.com/jjgarcianorway/anna-assistant/actions
  - Wait for binaries to be built (~10 minutes)
  - Test installation: ./scripts/install.sh
  - Update AUR packages (see docs/RELEASE-CHECKLIST.md)
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            RELEASE_TYPE="$2"
            shift 2
            ;;
        -m|--message)
            COMMIT_MSG="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
done

print_header

# Validate inputs
if [ -z "$COMMIT_MSG" ]; then
    print_error "Commit message is required (-m/--message)"
    echo ""
    show_help
    exit 1
fi

if [ -z "$RELEASE_TYPE" ] && [ -z "$VERSION" ]; then
    print_error "Either release type (-t) or explicit version (-v) is required"
    echo ""
    show_help
    exit 1
fi

if [ -n "$RELEASE_TYPE" ] && [ -n "$VERSION" ]; then
    print_error "Cannot specify both release type and explicit version"
    exit 1
fi

if [ -n "$RELEASE_TYPE" ]; then
    if [[ ! "$RELEASE_TYPE" =~ ^(major|minor|patch)$ ]]; then
        print_error "Release type must be: major, minor, or patch"
        exit 1
    fi
fi

# Check git status
print_info "Checking git status..."
if [ "$DRY_RUN" = false ] && [ -n "$(git status --porcelain)" ]; then
    print_error "Working directory is not clean. Commit or stash changes first."
    git status --short
    exit 1
fi
print_success "Git status clean"

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
if [ -z "$CURRENT_VERSION" ]; then
    print_error "Could not detect current version from Cargo.toml"
    exit 1
fi
print_success "Current version: $CURRENT_VERSION"

# Calculate new version
if [ -z "$VERSION" ]; then
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"

    case "$RELEASE_TYPE" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
    esac

    VERSION="${major}.${minor}.${patch}"
fi

print_success "New version: $VERSION"
echo ""

# Show what will be done
print_info "Release plan:"
echo "  Current version: $CURRENT_VERSION"
echo "  New version: $VERSION"
echo "  Commit message: $COMMIT_MSG"
echo "  Tag: v$VERSION"
echo ""
echo "Files to update:"
echo "  - Cargo.toml"
echo "  - packaging/aur/PKGBUILD"
echo "  - packaging/aur/PKGBUILD-bin"
echo ""
echo "Note: install.sh auto-fetches latest release (no update needed)"
echo ""

if [ "$DRY_RUN" = true ]; then
    print_warning "DRY RUN MODE - No changes will be made"
    echo ""
    exit 0
fi

# Confirm with user
read -p "Proceed with release? [y/N] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Release cancelled"
    exit 0
fi
echo ""

# Update version in files
print_info "Updating version in files..."

# Cargo.toml (workspace version)
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
print_success "Updated Cargo.toml"

# packaging/aur/PKGBUILD
sed -i "s/^pkgver=.*/pkgver=$VERSION/" packaging/aur/PKGBUILD
print_success "Updated packaging/aur/PKGBUILD"

# packaging/aur/PKGBUILD-bin
sed -i "s/^pkgver=.*/pkgver=$VERSION/" packaging/aur/PKGBUILD-bin
print_success "Updated packaging/aur/PKGBUILD-bin"

print_info "Skipping install.sh (auto-fetches latest release)"

echo ""

# Git operations
print_info "Creating git commit..."
git add Cargo.toml packaging/aur/PKGBUILD packaging/aur/PKGBUILD-bin
git commit -m "chore: bump version to v$VERSION

$COMMIT_MSG

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
print_success "Commit created"

print_info "Creating git tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION - $COMMIT_MSG"
print_success "Tag created"

print_info "Pushing to GitHub..."
git push origin main
git push origin "v$VERSION"
print_success "Pushed to GitHub"

echo ""
echo -e "${GREEN}╭─────────────────────────────────────────╮${NC}"
echo -e "${GREEN}│  ✓ Release v$VERSION Created!           ${NC}"
echo -e "${GREEN}╰─────────────────────────────────────────╯${NC}"
echo ""

print_info "Next steps:"
echo ""
echo "  1. Monitor GitHub Actions build:"
echo "     https://github.com/jjgarcianorway/anna-assistant/actions"
echo ""
echo "  2. Wait for binaries (~10 minutes)"
echo ""
echo "  3. Test binary download:"
echo "     rm -rf bin/ target/"
echo "     ./scripts/install.sh"
echo ""
echo "  4. Update AUR packages:"
echo "     See docs/RELEASE-CHECKLIST.md"
echo ""
echo "  5. Announce release (optional)"
echo ""
