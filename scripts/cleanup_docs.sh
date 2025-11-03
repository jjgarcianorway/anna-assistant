#!/usr/bin/env bash
# Clean up obsolete documentation - keep only essential files
set -euo pipefail

echo "Cleaning up obsolete documentation..."

# Keep these essential files
KEEP=(
  "README.md"
  "CHANGELOG.md"
  "docs/V1.0-QUICKSTART.md"
)

# Remove all old doc files except essentials
cd "$(git rev-parse --show-toplevel)"

# Remove obsolete root-level docs
for f in *.md *.txt; do
  if [[ "$f" == "README.md" || "$f" == "CHANGELOG.md" ]]; then
    continue
  fi
  if [[ -f "$f" ]]; then
    echo "  Removing: $f"
    rm "$f"
  fi
done

# Remove all old docs in docs/ except V1.0-QUICKSTART.md
if [[ -d docs ]]; then
  for f in docs/*.md; do
    if [[ "$f" == "docs/V1.0-QUICKSTART.md" ]]; then
      continue
    fi
    if [[ -f "$f" ]]; then
      echo "  Removing: $f"
      rm "$f"
    fi
  done
fi

# Remove empty directories
find docs -type d -empty -delete 2>/dev/null || true

echo "✓ Cleanup complete"
echo ""
echo "Remaining documentation:"
echo "  README.md - Main project readme"
echo "  CHANGELOG.md - Version history"
echo "  docs/V1.0-QUICKSTART.md - User quickstart guide"
