#!/usr/bin/env bash
# Anna Release Script — zero-arg
# Auto-bump RC tags, update Cargo.toml, build, tag, push, upload assets

set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

say() { printf "%s\n" "$*"; }
die() { printf "ERROR: %s\n" "$*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || die "Missing '$1'"; }
require git
require awk
require sed
require cargo

# 1) Compute next RC tag from Cargo.toml or git tags
CURRENT="$(grep -m1 '^version = "' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')"
BASE="${CURRENT%%-rc.*}"
RC_NUM="$(git tag --list 'v'"$BASE"'-rc.*' | sed -E 's/.*-rc\.([0-9]+)$/\1/' | sort -n | tail -1)"
NEXT_RC=$(( ${RC_NUM:-0} + 1 ))
TAG="v${BASE}-rc.${NEXT_RC}"

say "→ Current version: $CURRENT"
say "→ Next release tag: $TAG"

# 2) Set version in Cargo.toml
sed -i -E 's/^version = ".*"$/version = "'"${BASE}-rc.${NEXT_RC}"'"/' Cargo.toml
say "→ Updated Cargo.toml to ${BASE}-rc.${NEXT_RC}"

# 3) Commit, tag, push
git add -A
if ! git diff --cached --quiet; then
  git commit -m "chore(release): ${TAG}

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
  say "→ Committed changes"
else
  say "→ No changes to commit"
fi

if git rev-parse "$TAG" >/dev/null 2>&1; then
  say "✗ Tag $TAG already exists locally"
  exit 1
fi

git tag -a "${TAG}" -m "${TAG}

$(cat .release-notes-v1.0-draft.md 2>/dev/null || echo 'Release candidate')

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

say "→ Created tag ${TAG}"

git push origin HEAD:main --tags
say "→ Pushed to origin with tags"

# 4) Build artifacts locally so we can upload with gh if present
say "→ Building release binaries…"
cargo build --release --bin annad --bin annactl
mkdir -p dist
cp target/release/annad target/release/annactl dist/
say "→ Binaries built in dist/"

# 5) Create or update GitHub Release and upload assets if gh exists
if command -v gh >/dev/null 2>&1; then
  say "→ Creating GitHub release with gh CLI…"

  # Try to create release, if it exists, upload to it
  if gh release create "${TAG}" dist/annad dist/annactl \
       --prerelease \
       --title "${TAG}" \
       --notes-file .release-notes-v1.0-draft.md 2>/dev/null; then
    say "→ GitHub release created with assets"
  else
    say "→ Release exists, uploading assets…"
    gh release upload "${TAG}" dist/annad dist/annactl --clobber
    say "→ Assets uploaded"
  fi
else
  say "→ gh CLI not found; CI will publish assets via GitHub Actions"
fi

echo ""
echo "✔ Release ${TAG} prepared and pushed"
echo "▶ Tag: ${TAG}"
echo "▶ Binaries: dist/annad, dist/annactl"
echo ""
echo "Next: Wait for CI to attach assets, then run installer:"
echo "  sudo ./scripts/install.sh"
