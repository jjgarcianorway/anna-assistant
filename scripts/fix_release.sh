#!/bin/bash
# Fix Release Script - Upload missing assets to a GitHub release
#
# Usage: ./scripts/fix_release.sh <version>
# Example: ./scripts/fix_release.sh 0.0.43
#
# This script:
# 1. Builds release binaries
# 2. Creates versioned binary names
# 3. Generates SHA256SUMS
# 4. Uploads to existing GitHub release
#
# Requirements:
# - Rust toolchain (cargo)
# - GitHub CLI (gh) authenticated
# - Must be run from project root

set -euo pipefail

# Colors
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
CYAN=$'\033[0;36m'
NC=$'\033[0m'

log_info() { echo "${CYAN}[INFO]${NC} $1"; }
log_ok() { echo "${GREEN}[OK]${NC} $1"; }
log_warn() { echo "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo "${RED}[ERROR]${NC} $1"; }

# Check arguments
if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.0.43"
    exit 1
fi

VERSION="$1"
TAG="v${VERSION}"
ARCH="x86_64-unknown-linux-gnu"

# Verify we're in project root
if [[ ! -f "Cargo.toml" ]]; then
    log_error "Must be run from project root (Cargo.toml not found)"
    exit 1
fi

# Verify gh is authenticated
if ! gh auth status &>/dev/null; then
    log_error "GitHub CLI not authenticated. Run: gh auth login"
    exit 1
fi

# Verify release exists
if ! gh release view "$TAG" &>/dev/null; then
    log_error "Release $TAG not found. Create it first."
    exit 1
fi

log_info "Fixing release ${TAG}..."

# Create temp directory for artifacts
DIST_DIR=$(mktemp -d)
trap "rm -rf $DIST_DIR" EXIT

# Build release
log_info "Building release binaries..."
cargo build --release --workspace

# Copy and rename binaries
ANNAD_NAME="annad-${VERSION}-${ARCH}"
ANNACTL_NAME="annactl-${VERSION}-${ARCH}"

cp target/release/annad "${DIST_DIR}/${ANNAD_NAME}"
cp target/release/annactl "${DIST_DIR}/${ANNACTL_NAME}"

chmod 755 "${DIST_DIR}/${ANNAD_NAME}"
chmod 755 "${DIST_DIR}/${ANNACTL_NAME}"

log_ok "Built binaries"

# Generate checksums
log_info "Generating SHA256SUMS..."
cd "$DIST_DIR"
sha256sum "$ANNAD_NAME" "$ANNACTL_NAME" > SHA256SUMS

log_ok "Generated checksums:"
cat SHA256SUMS

# Upload to release
log_info "Uploading to release ${TAG}..."
gh release upload "$TAG" \
    "$ANNAD_NAME" \
    "$ANNACTL_NAME" \
    SHA256SUMS \
    --clobber

log_ok "Assets uploaded to ${TAG}"

# Verify
log_info "Verifying release assets..."
ASSETS=$(gh release view "$TAG" --json assets -q '.assets[].name' | wc -l)
if [[ "$ASSETS" -ge 3 ]]; then
    log_ok "Release ${TAG} now has ${ASSETS} assets"
else
    log_warn "Release ${TAG} has ${ASSETS} assets (expected 3)"
fi

echo ""
log_ok "Release ${TAG} fixed!"
echo ""
echo "Test with:"
echo "  curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash"
