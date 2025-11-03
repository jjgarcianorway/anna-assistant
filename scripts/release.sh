#!/usr/bin/env bash
# Anna Release Script — zero-arg
# Detects next RC from remote tags, builds tarball + checksum, never recreates tags

set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

say() { printf "%s\n" "$*"; }
die() { printf "ERROR: %s\n" "$*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || die "Missing '$1'"; }
require git
require cargo
require tar
require sha256sum

# Compute next RC from remote tags (never recreate existing tags)
next_rc() {
  local latest
  # Fetch remote tags and find the highest RC tag
  latest=$(git ls-remote --tags origin 'refs/tags/v*' | \
           awk -F/ '{print $3}' | \
           grep -E '^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$' | \
           sort -V | \
           tail -n1)

  if [[ -z "$latest" ]]; then
    # No RC tags exist, start with rc.1
    echo "v1.0.0-rc.1"
    return
  fi

  # Extract base and RC number
  local base="${latest%-rc.*}"
  local rc_num="${latest##*-rc.}"

  # Bump RC number
  echo "${base}-rc.$((rc_num + 1))"
}

NEW_TAG=$(next_rc)
NEW_VER="${NEW_TAG#v}"  # Remove 'v' prefix for Cargo.toml

say "→ Computed next release: $NEW_TAG"

# Check if tag already exists locally
if git rev-parse "$NEW_TAG" >/dev/null 2>&1; then
  die "Tag $NEW_TAG already exists locally. Fetch and sync with remote first."
fi

# Update Cargo.toml
say "→ Updating Cargo.toml to $NEW_VER"
sed -i -E 's/^version = ".*"$/version = "'"$NEW_VER"'"/' Cargo.toml

# Build release binaries
say "→ Building release binaries…"
cargo build --release --bin annad --bin annactl

# Create dist directory and package tarball
say "→ Packaging tarball…"
mkdir -p dist
cp target/release/annad dist/
cp target/release/annactl dist/
tar -C dist -czf dist/anna-linux-x86_64.tar.gz annad annactl

# Generate checksum
say "→ Generating checksum…"
(cd dist && sha256sum anna-linux-x86_64.tar.gz > anna-linux-x86_64.tar.gz.sha256)

# Show checksum for verification
say "→ Checksum:"
cat dist/anna-linux-x86_64.tar.gz.sha256

# Commit changes
say "→ Committing changes…"
git add Cargo.toml
if ! git diff --cached --quiet; then
  git commit -m "chore(release): $NEW_TAG

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
  say "→ Committed"
else
  say "→ No changes to commit"
fi

# Create annotated tag
say "→ Creating tag $NEW_TAG…"
git tag -a "$NEW_TAG" -m "$NEW_TAG

$(cat .release-notes-v1.0-draft.md 2>/dev/null || echo 'Release candidate - complete RC pipeline')

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Push branch and tags
say "→ Pushing to origin…"
git push origin HEAD:main --tags

# Upload assets with gh if available
if command -v gh >/dev/null 2>&1; then
  say "→ Uploading assets with gh CLI…"

  # Try to create release
  if gh release create "$NEW_TAG" \
       dist/anna-linux-x86_64.tar.gz \
       dist/anna-linux-x86_64.tar.gz.sha256 \
       --prerelease \
       --title "$NEW_TAG" \
       --notes-file .release-notes-v1.0-draft.md 2>/dev/null; then
    say "→ GitHub release created with assets"
  else
    # Release exists, upload assets
    say "→ Release exists, uploading assets…"
    gh release upload "$NEW_TAG" \
       dist/anna-linux-x86_64.tar.gz \
       dist/anna-linux-x86_64.tar.gz.sha256 \
       --clobber
    say "→ Assets uploaded"
  fi
else
  say "→ gh CLI not found; GitHub Actions will upload assets"
fi

echo ""
echo "✔ Release $NEW_TAG prepared and pushed"
echo "▶ Tag: $NEW_TAG"
echo "▶ Assets:"
echo "  - dist/anna-linux-x86_64.tar.gz"
echo "  - dist/anna-linux-x86_64.tar.gz.sha256"
echo ""
echo "Next:"
echo "  1. Wait for GitHub Actions to complete (~2 min)"
echo "  2. Verify assets appear in release: https://github.com/jjgarcianorway/anna-assistant/releases/tag/$NEW_TAG"
echo "  3. Test installer: sudo ./scripts/install.sh"
