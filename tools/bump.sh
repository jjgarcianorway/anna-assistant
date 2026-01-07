#!/bin/bash
# tools/bump.sh — Bump version (patch|minor|major) and update all Cargo.toml files
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION_FILE="$REPO_ROOT/VERSION"

if [[ ! -f "$VERSION_FILE" ]]; then
    echo "Error: VERSION file not found at $VERSION_FILE" >&2
    exit 1
fi

CURRENT=$(cat "$VERSION_FILE")
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "${1:-patch}" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    *)
        echo "Usage: $0 {patch|minor|major}" >&2
        exit 1
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo "$NEW_VERSION" > "$VERSION_FILE"

# Update workspace crate versions
for toml in "$REPO_ROOT"/cmd/*/Cargo.toml; do
    if [[ -f "$toml" ]]; then
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$toml"
    fi
done

echo "$NEW_VERSION"
