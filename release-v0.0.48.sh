#!/bin/bash
# Release script for v0.0.48
# Run this script from the project root directory

set -e

echo "=== Building and Testing v0.0.48 ==="

# 1. Run tests
echo "Running tests..."
cargo test --workspace

# 2. Build release binaries
echo "Building release binaries..."
cargo build --release --workspace

# 3. Commit changes
echo "Committing changes..."
git add -A
git commit -m "$(cat <<'EOF'
v0.0.48: Smart clarifications & minimal probes (v0.45.3)

- Minimal Probe Policy: reduce_probes() limits max 3 (or 4 for health)
- Urgency enum: Normal/Quick/Detailed probe limits
- ClarifyRequest: ttl_seconds, allow_custom fields
- REPL clarification state with timeout handling
- ServiceDeskResult: clarification_request field

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"

# 4. Create and push tag
echo "Creating tag..."
git tag v0.0.48
git push origin main --tags

# 5. Prepare binaries for upload
echo "Preparing binaries..."
cp target/release/annactl annactl-linux-x86_64
cp target/release/annad annad-linux-x86_64
sha256sum annactl-linux-x86_64 annad-linux-x86_64 > SHA256SUMS

# 6. Create GitHub release
echo "Creating GitHub release..."
gh release create v0.0.48 --title "v0.0.48" --notes "$(cat <<'EOF'
## v0.0.48 - Smart Clarifications & Minimal Probes (v0.45.3)

### Added

- **Minimal Probe Policy** (`probe_spine.rs`):
  - `reduce_probes()` function limits probes to max 3 (default) or 4 (system health)
  - `Urgency` enum: Normal (max 3), Quick (max 2), Detailed (max 5)
  - Never runs both JournalErrors AND JournalWarnings unless Detailed

- **Enhanced Clarification Request** (`clarify_v2.rs`):
  - `ClarifyRequest.ttl_seconds` field (default 300 = 5 minutes)
  - `ClarifyRequest.allow_custom` field to control free-text input
  - `with_ttl()` and `no_custom()` builder methods

- **REPL Clarification State** (`commands.rs`):
  - `PendingClarification` struct tracks pending requests
  - REPL prompt changes to `[choice]>` when clarification pending
  - TTL enforcement: clarification expires after ttl_seconds

- **ServiceDeskResult Extension** (`rpc.rs`):
  - `clarification_request: Option<ClarifyRequest>` field

### Changed

- **RPC Handler** (`rpc_handler.rs`):
  - `reduce_probes()` called after spine enforcement
  - Probe cap applied even without spine enforcement (max 3 or 4)
EOF
)"

# 7. Upload binaries
echo "Uploading binaries..."
gh release upload v0.0.48 annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS --clobber

# 8. Cleanup
echo "Cleaning up..."
rm annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS

echo "=== Release v0.0.48 Complete! ==="
