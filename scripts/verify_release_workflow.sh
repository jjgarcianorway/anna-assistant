#!/usr/bin/env bash
# Verify Release Workflow - Pre-flight checks
# Ensures release.sh and install.sh are robust and working correctly
set -Eeuo pipefail

CYAN='\033[96m'
GREEN='\033[92m'
RED='\033[91m'
YELLOW='\033[93m'
RESET='\033[0m'

say() { echo -e "${CYAN}→${RESET} $*"; }
ok() { echo -e "${GREEN}✓${RESET} $*"; }
err() { echo -e "${RED}✗${RESET} $*"; }
warn() { echo -e "${YELLOW}⚠${RESET} $*"; }

cd "$(dirname "$0")/.."

echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}║         Release Workflow Verification - Pre-Flight Checks             ║${RESET}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Test 1: Script Syntax
# ═══════════════════════════════════════════════════════════════════════════
say "Test 1: Verifying script syntax..."
if bash -n scripts/release.sh 2>/dev/null; then
  ok "release.sh syntax valid"
else
  err "release.sh has syntax errors"
  exit 1
fi

if bash -n scripts/install.sh 2>/dev/null; then
  ok "install.sh syntax valid"
else
  err "install.sh has syntax errors"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 2: Required Commands Present
# ═══════════════════════════════════════════════════════════════════════════
say "Test 2: Checking required commands..."
for cmd in git curl jq cargo; do
  if command -v "$cmd" >/dev/null 2>&1; then
    ok "$cmd found"
  else
    err "$cmd not found (required)"
    exit 1
  fi
done

# ═══════════════════════════════════════════════════════════════════════════
# Test 3: Cargo.toml Version Format
# ═══════════════════════════════════════════════════════════════════════════
say "Test 3: Verifying Cargo.toml version format..."
CURRENT_VER=$(grep -E '^version = ".*"' Cargo.toml | head -1 | sed -E 's/.*"(.*)".*/\1/')

if [[ "$CURRENT_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
  ok "Version format valid: $CURRENT_VER"
else
  err "Invalid version format in Cargo.toml: $CURRENT_VER"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 4: Git Working Directory Clean
# ═══════════════════════════════════════════════════════════════════════════
say "Test 4: Checking git status..."
if git diff --quiet && git diff --cached --quiet; then
  ok "Working directory is clean"
else
  warn "Working directory has uncommitted changes"
  warn "This is OK for testing, but release.sh requires a clean state"
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 5: GitHub API Access
# ═══════════════════════════════════════════════════════════════════════════
say "Test 5: Testing GitHub API access..."
OWNER="jjgarcianorway"
REPO="anna-assistant"

api_response=$(curl -fsSL "https://api.github.com/repos/$OWNER/$REPO/releases?per_page=1" 2>/dev/null || echo "null")
if [[ "$api_response" != "null" && -n "$api_response" ]]; then
  ok "GitHub API accessible"
else
  err "Cannot access GitHub API"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 6: Latest Release Has Assets
# ═══════════════════════════════════════════════════════════════════════════
say "Test 6: Checking latest release has required assets..."
latest_tag=$(echo "$api_response" | jq -r '.[0].tag_name' 2>/dev/null)
has_tarball=$(echo "$api_response" | jq -r '.[0].assets[] | select(.name=="anna-linux-x86_64.tar.gz") | .name' 2>/dev/null)
has_checksum=$(echo "$api_response" | jq -r '.[0].assets[] | select(.name=="anna-linux-x86_64.tar.gz.sha256") | .name' 2>/dev/null)

if [[ "$has_tarball" == "anna-linux-x86_64.tar.gz" ]]; then
  ok "Latest release ($latest_tag) has tarball"
else
  warn "Latest release ($latest_tag) missing tarball"
fi

if [[ "$has_checksum" == "anna-linux-x86_64.tar.gz.sha256" ]]; then
  ok "Latest release ($latest_tag) has checksum"
else
  warn "Latest release ($latest_tag) missing checksum"
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 7: Build Test
# ═══════════════════════════════════════════════════════════════════════════
say "Test 7: Testing cargo build (this may take a minute)..."
if cargo build --release --bin annad --bin annactl >/dev/null 2>&1; then
  ok "Cargo build successful"
else
  err "Cargo build failed"
  err "Fix build errors before releasing"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 8: Verify Binaries Exist
# ═══════════════════════════════════════════════════════════════════════════
say "Test 8: Verifying built binaries..."
if [[ -f target/release/annad ]]; then
  ok "annad binary exists"
else
  err "annad binary not found"
  exit 1
fi

if [[ -f target/release/annactl ]]; then
  ok "annactl binary exists"
else
  err "annactl binary not found"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 9: GitHub Actions Workflow
# ═══════════════════════════════════════════════════════════════════════════
say "Test 9: Verifying GitHub Actions workflow..."
if [[ -f .github/workflows/release.yml ]]; then
  ok "release.yml workflow exists"
else
  err "release.yml workflow missing"
  exit 1
fi

# Verify workflow has required steps
if grep -q "anna-linux-x86_64.tar.gz" .github/workflows/release.yml; then
  ok "Workflow builds required artifacts"
else
  err "Workflow missing artifact generation"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Test 10: Version Consistency
# ═══════════════════════════════════════════════════════════════════════════
say "Test 10: Checking version consistency across workspace..."
workspace_ver=$(grep -E '^\[workspace.package\]' -A 10 Cargo.toml | grep -E '^version = ' | head -1 | sed -E 's/.*"(.*)".*/\1/')

if [[ "$workspace_ver" == "$CURRENT_VER" ]]; then
  ok "Workspace version matches: $workspace_ver"
else
  err "Version mismatch: workspace=$workspace_ver, current=$CURRENT_VER"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════╗${RESET}"
echo -e "${GREEN}║                    ✨ All Checks Passed! ✨                           ║${RESET}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════╝${RESET}"
echo ""
echo -e "${CYAN}Release workflow is ready to use!${RESET}"
echo ""
echo "Current version: ${GREEN}v$CURRENT_VER${RESET}"
echo "Latest release:  ${GREEN}$latest_tag${RESET}"
echo ""
echo "To create a new release:"
echo "  ${YELLOW}./scripts/release.sh${RESET}"
echo ""
echo "The release script will:"
echo "  1. ✓ Verify build succeeds"
echo "  2. ✓ Bump version to next RC"
echo "  3. ✓ Update Cargo.toml and commit"
echo "  4. ✓ Create and push git tag"
echo "  5. ✓ Wait for GitHub Actions to build"
echo "  6. ✓ Verify assets are uploaded"
echo ""
echo "The install script will:"
echo "  1. ✓ Only install releases with complete assets"
echo "  2. ✓ Retry if GitHub Actions is still building"
echo "  3. ✓ Verify checksums before installation"
echo "  4. ✓ Never install incomplete releases"
echo ""
