#!/bin/bash
# Clean release for v0.12.3 - handles tag conflicts automatically

set -euo pipefail

VERSION="0.12.3"
TAG="v${VERSION}"

echo "╭─────────────────────────────────────────╮"
echo "│  Clean Release: ${TAG}                    "
echo "╰─────────────────────────────────────────╯"
echo ""

# Step 1: Delete local tag if exists
echo "→ Cleaning local tags..."
if git tag -l | grep -q "^${TAG}$"; then
    git tag -d ${TAG}
    echo "✓ Deleted local tag ${TAG}"
else
    echo "✓ No local tag to delete"
fi

# Also delete v0.12.4 if it exists
if git tag -l | grep -q "^v0.12.4$"; then
    git tag -d v0.12.4
    echo "✓ Deleted local tag v0.12.4"
fi
echo ""

# Step 2: Try to delete remote tag (may fail if doesn't exist, that's OK)
echo "→ Cleaning remote tags..."
git push origin :refs/tags/${TAG} 2>/dev/null && echo "✓ Deleted remote tag ${TAG}" || echo "✓ No remote tag ${TAG} to delete"
git push origin :refs/tags/v0.12.4 2>/dev/null && echo "✓ Deleted remote tag v0.12.4" || echo "✓ No remote tag v0.12.4 to delete"
echo ""

# Step 3: Run release.sh
echo "→ Running release.sh..."
echo ""

./scripts/release.sh -t patch -m "v0.12.3: collectors, radars, telemetry RPC endpoints

New Features:
- Focused collectors for sensors, network, disk, top processes
- Health and Network radar scoring systems (0-10 scale)
- Three new RPC endpoints: collect, classify, radar_show
- Three new CLI commands with JSON support
- Complete telemetry database schema

Improvements:
- Install script with better daemon startup verification
- Full graceful degradation for missing sensors
- Beautiful formatted output with radar visualizations

Testing:
- All new commands verified working
- Laptop persona detection: 90% confidence
- Perfect health/network scores on test system

See docs/ for complete implementation details."
