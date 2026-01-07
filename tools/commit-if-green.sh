#!/bin/bash
# tools/commit-if-green.sh — Only commit if fmt/clippy/tests/build all pass
# Usage: MSG="your commit message" tools/commit-if-green.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

VERSION_FILE="$REPO_ROOT/VERSION"
CHANGES_FILE="$REPO_ROOT/CHANGES.txt"

if [[ -z "${MSG:-}" ]]; then
    echo "Error: MSG environment variable required" >&2
    echo "Usage: MSG=\"commit message\" $0" >&2
    exit 1
fi

if [[ ! -f "$VERSION_FILE" ]]; then
    echo "Error: VERSION file not found" >&2
    exit 1
fi

VERSION=$(cat "$VERSION_FILE")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Quality gates for v$VERSION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo
echo "▶ Running cargo fmt --check..."
if ! cargo fmt --all -- --check; then
    echo "❌ Format check failed. Run: cargo fmt --all" >&2
    exit 1
fi
echo "✅ Format check passed"

echo
echo "▶ Running cargo clippy..."
if ! cargo clippy --all -- -D warnings; then
    echo "❌ Clippy failed" >&2
    exit 1
fi
echo "✅ Clippy passed"

echo
echo "▶ Running cargo test..."
if ! cargo test --all; then
    echo "❌ Tests failed" >&2
    exit 1
fi
echo "✅ Tests passed"

echo
echo "▶ Running cargo build --release..."
if ! cargo build --release; then
    echo "❌ Build failed" >&2
    exit 1
fi
echo "✅ Build passed"

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All quality gates passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Append to CHANGES.txt
echo "v$VERSION — $(date -u +%Y-%m-%dT%H:%M:%SZ) — $MSG" >> "$CHANGES_FILE"

# Create commit with trailers
COMMIT_MSG="$MSG

Version: $VERSION

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

git add -A
git commit -m "$COMMIT_MSG"
git tag "v$VERSION"

echo
echo "✅ Committed and tagged as v$VERSION"
echo "📝 Updated $CHANGES_FILE"
