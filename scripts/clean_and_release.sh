#!/usr/bin/env bash
# Clean up local tags and prepare for fresh rc.4 release
set -euo pipefail

echo "Step 1: Fetching remote tags..."
git fetch --tags --prune

echo ""
echo "Step 2: Checking remote tags..."
REMOTE_TAGS=$(git ls-remote --tags origin 'refs/tags/v1.0.0-rc*' | awk -F/ '{print $3}' | grep -v '\^{}' | sort -V)
echo "Remote RC tags:"
echo "$REMOTE_TAGS"

echo ""
echo "Step 3: Local tags before cleanup:"
git tag -l 'v1.0.0-rc*' | sort -V

echo ""
echo "Step 4: Finding highest remote RC..."
HIGHEST_REMOTE=$(echo "$REMOTE_TAGS" | tail -n1)
echo "Highest remote: $HIGHEST_REMOTE"

# Extract RC number
REMOTE_BASE="${HIGHEST_REMOTE%-rc.*}"
REMOTE_RC="${HIGHEST_REMOTE##*-rc.}"
NEXT_RC=$((REMOTE_RC + 1))
NEXT_TAG="${REMOTE_BASE}-rc.${NEXT_RC}"

echo ""
echo "Step 5: Next release will be: $NEXT_TAG"

echo ""
echo "Step 6: Deleting local tags higher than remote..."
for tag in $(git tag -l 'v1.0.0-rc*'); do
  rc_num="${tag##*-rc.}"
  if [ "$rc_num" -gt "$REMOTE_RC" ]; then
    echo "  Deleting local tag: $tag"
    git tag -d "$tag"
  fi
done

echo ""
echo "Step 7: Local tags after cleanup:"
git tag -l 'v1.0.0-rc*' | sort -V

echo ""
echo "✔ Ready to release!"
echo ""
echo "Run: ./scripts/release.sh"
echo "Expected tag: $NEXT_TAG"
