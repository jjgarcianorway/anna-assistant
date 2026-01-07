#!/bin/bash
# tools/changelog.sh — Append commit trailers to CHANGES.txt
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHANGES_FILE="$REPO_ROOT/CHANGES.txt"

# Get the latest commit message
COMMIT_MSG=$(git log -1 --pretty=%B)

# Extract version trailer if present
VERSION=$(echo "$COMMIT_MSG" | grep -oP '^Version: \K.*' || echo "unknown")

# Extract first line (summary)
SUMMARY=$(echo "$COMMIT_MSG" | head -n1)

# Append to changelog
echo "v$VERSION — $(date -u +%Y-%m-%dT%H:%M:%SZ) — $SUMMARY" >> "$CHANGES_FILE"

echo "Updated $CHANGES_FILE"
