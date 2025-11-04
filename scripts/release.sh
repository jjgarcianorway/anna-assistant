#!/usr/bin/env bash
# Anna Release Script — Smart Version Bumping
# Only bumps version if there are actual code changes
# GitHub Actions handles building and uploading binaries
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

OWNER="jjgarcianorway"
REPO="anna-assistant"

say() { printf "%s\n" "$*"; }
die() { printf "ERROR: %s\n" "$*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || die "Missing '$1'"; }
require git
require curl
require jq

# Compute next RC from GitHub API (never recreate existing tags)
next_rc() {
  local latest

  # Fetch tags from GitHub API
  latest=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/git/refs/tags" 2>/dev/null | \
           jq -r '.[].ref' | \
           sed 's|refs/tags/||' | \
           grep -E '^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$' | \
           sort -V | \
           tail -n1)

  if [[ -z "$latest" || "$latest" == "null" ]]; then
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

# Get current version from Cargo.toml
say "→ Reading current version from Cargo.toml..."
CURRENT_VER=$(grep -E '^version = ".*"' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')
CURRENT_TAG="v${CURRENT_VER}"

say "→ Current version in Cargo.toml: $CURRENT_TAG"

# Check if there are any changes since last tag
if git rev-parse "$CURRENT_TAG" >/dev/null 2>&1; then
  # Tag exists, check for changes
  say "→ Checking for code changes since $CURRENT_TAG..."
  CHANGES=$(git diff "$CURRENT_TAG" --stat -- ':!Cargo.toml' ':!docs/*.md' | wc -l)

  if [[ "$CHANGES" -eq 0 ]]; then
    say ""
    say "✓ No code changes since $CURRENT_TAG"
    say "✓ Version $CURRENT_TAG is up to date"
    say ""
    say "→ Nothing to release. To force a release, make a code change first."
    say "  (Documentation-only changes don't trigger releases)"
    exit 0
  fi

  say "→ Found $CHANGES changed files since $CURRENT_TAG"
fi

say "→ Fetching latest tag from GitHub API..."
NEW_TAG=$(next_rc)
NEW_VER="${NEW_TAG#v}"  # Remove 'v' prefix for Cargo.toml

say "→ Latest remote tag: ${NEW_TAG%-rc.*}-rc.$((${NEW_TAG##*-rc.} - 1))"
say "→ Next release will be: $NEW_TAG"

# Check if tag already exists locally
if git rev-parse "$NEW_TAG" >/dev/null 2>&1; then
  die "Tag $NEW_TAG already exists locally. Fetch and sync with remote first."
fi

# Verify build succeeds before committing
say "→ Verifying build succeeds..."
if ! cargo build --release --bin annad --bin annactl >/dev/null 2>&1; then
  die "Build failed. Fix errors before releasing."
fi
say "→ Build successful"

# Update Cargo.toml
say "→ Updating Cargo.toml to $NEW_VER"
sed -i -E 's/^version = ".*"$/version = "'"$NEW_VER"'"/' Cargo.toml

# Verify the version was updated correctly
UPDATED_VER=$(grep -E '^version = ".*"' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')
if [[ "$UPDATED_VER" != "$NEW_VER" ]]; then
  die "Failed to update Cargo.toml version (expected $NEW_VER, got $UPDATED_VER)"
fi
say "→ Verified Cargo.toml updated to $NEW_VER"

# Commit changes
say "→ Committing Cargo.toml update…"
git add Cargo.toml
if ! git diff --cached --quiet; then
  git commit -m "chore(release): bump version to $NEW_TAG

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
  say "→ Committed"
else
  say "→ No changes to commit"
fi

# Create annotated tag
say "→ Creating tag $NEW_TAG…"
git tag -a "$NEW_TAG" -m "$NEW_TAG

$(cat .release-notes-v1.0-draft.md 2>/dev/null || echo 'Release candidate - automated RC pipeline')

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Push branch and tags
say "→ Pushing to origin…"
git push origin HEAD:main --tags

echo ""
echo "✔ Release $NEW_TAG created and pushed"
echo "▶ Tag: $NEW_TAG"
echo ""
say "→ Waiting for GitHub Actions to build and upload assets..."

# Wait for GitHub Actions to complete (up to 5 minutes)
for i in {1..60}; do
  sleep 5

  # Check if release has assets
  assets=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases/tags/$NEW_TAG" 2>/dev/null | \
           jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz") | .name' 2>/dev/null)

  if [[ "$assets" == "anna-linux-x86_64.tar.gz" ]]; then
    echo ""
    echo "✔ GitHub Actions completed successfully"
    echo "✔ Assets uploaded: anna-linux-x86_64.tar.gz"
    echo ""
    echo "Release is ready!"
    echo "  → https://github.com/$OWNER/$REPO/releases/tag/$NEW_TAG"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 POST-RELEASE CHECKLIST"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "1. Test the installer:"
    echo "   sudo ./scripts/install.sh"
    echo ""
    echo "2. Verify version sync:"
    echo "   ./scripts/verify_versions.sh"
    echo ""
    echo "3. Confirm installation:"
    echo "   annactl version --check"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    exit 0
  fi

  printf "\r  ⏳ Waiting for build... (%ds)" $((i * 5))
done

echo ""
echo "⚠ GitHub Actions is taking longer than expected"
echo "  Check manually: https://github.com/$OWNER/$REPO/actions"
echo "  Release page: https://github.com/$OWNER/$REPO/releases/tag/$NEW_TAG"
echo ""
echo "The installer will wait for assets to be available before installing."
echo ""
