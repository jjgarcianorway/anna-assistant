#!/usr/bin/env bash
# Anna Release Script — zero-arg
# Behavior:
#  - Auto-detect current version from Cargo.toml
#  - If version already ends with -rc.* → bump rc number
#  - Else if version is 0.x.y or x.y.z → create rc.1 tag without bumping Cargo
#  - Commit all changes with an auto message compiled from release notes
#  - Update README badges/version blocks automatically
#  - Create annotated tag and push with tags

set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

say() { printf "%s\n" "$*"; }
die() { printf "ERROR: %s\n" "$*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || die "Missing '$1'"; }
require git
require awk
require sed

# 1) Detect version from Cargo.toml
ver_line=$(awk -F '"' '/^version *=/ {print $2; exit}' Cargo.toml || true)
[[ -n "${ver_line:-}" ]] || die "Cannot read version from Cargo.toml"
current="$ver_line"

# 2) Compute new tag
#   - If already rc: 1.0.0-rc.N -> rc.(N+1)
#   - Else: 1.0.0 -> 1.0.0-rc.1
if [[ "$current" =~ ^([0-9]+\.[0-9]+\.[0-9]+)-rc\.([0-9]+)$ ]]; then
  base="${BASH_REMATCH[1]}"; rc="${BASH_REMATCH[2]}"
  new_tag="v${base}-rc.$((rc+1))"
else
  # leave Cargo.toml as-is; tag is rc.1 for current version
  new_tag="v${current}-rc.1"
fi

# 3) Synthesize commit message
notes_file=".release-notes-v1.0-draft.md"
if [[ -f "$notes_file" ]]; then
  msg_title="chore(release): ${new_tag#v} Hildegard"
  msg_body=$(sed -n '1,200p' "$notes_file")
else
  msg_title="chore(release): ${new_tag#v} Hildegard"
  msg_body="$(git log -1 --pretty=%B || true)"
fi
commit_msg="${msg_title}

${msg_body}

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 4) Update README badges/version snippets in-place (best-effort)
#    Replace any 'Anna vX.Y.Z-rc.*' or 'Anna vX.Y.Z' with new tag
if [[ -f README.md ]] && grep -qE 'Anna v[0-9]+\.[0-9]+\.[0-9]+' README.md; then
  sed -i -E "s/Anna v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?/Anna ${new_tag}/" README.md
  say "→ Updated README.md version to ${new_tag}"
fi

# 5) Stage and commit everything (no prompt)
git add -A
if ! git diff --cached --quiet; then
  git commit -m "$commit_msg"
  say "→ Committed changes"
fi

# 6) Tag and push
if git rev-parse "$new_tag" >/dev/null 2>&1; then
  say "Tag $new_tag already exists, skipping tag creation"
else
  git tag -a "$new_tag" -m "$commit_msg"
  say "→ Created tag ${new_tag}"
fi

git push origin HEAD --tags
say "→ Pushed to origin with tags"

# 7) Summary
printf "\n✔ Release prepared and pushed\n▶ Tag: %s\n" "$new_tag"
