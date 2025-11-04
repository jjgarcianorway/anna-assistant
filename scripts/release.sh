#!/bin/bash
# Anna Assistant Release Script
# Automates version tagging and GitHub release creation

set -e

# Colors (pastel theme)
BLUE='\033[38;5;117m'
GREEN='\033[38;5;120m'
YELLOW='\033[38;5;228m'
RED='\033[38;5;210m'
CYAN='\033[38;5;159m'
GRAY='\033[38;5;250m'
RESET='\033[0m'
BOLD='\033[1m'

echo -e "${BOLD}${BLUE}╭─────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${BLUE}│         Anna Release Automation            │${RESET}"
echo -e "${BOLD}${BLUE}╰─────────────────────────────────────────────╯${RESET}"
echo

# Check we're in git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}✗ Not a git repository${RESET}"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}✗ Uncommitted changes detected${RESET}"
    echo -e "${GRAY}  Please commit or stash changes first${RESET}"
    exit 1
fi

echo -e "${GREEN}✓${RESET} Working directory clean"

# Extract version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ -z "$VERSION" ]; then
    echo -e "${RED}✗ Could not extract version from Cargo.toml${RESET}"
    exit 1
fi

TAG="v${VERSION}"
echo -e "${CYAN}→${RESET} Detected version: ${BOLD}${TAG}${RESET}"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠${RESET}  Tag ${TAG} already exists"
    echo -e "${GRAY}  Bump version in Cargo.toml first${RESET}"
    exit 1
fi

# Verify build works
echo -e "${CYAN}→${RESET} Building release binaries..."
if ! cargo build --release 2>&1 | tail -3; then
    echo -e "${RED}✗ Build failed${RESET}"
    exit 1
fi

echo -e "${GREEN}✓${RESET} Build successful"

# Create annotated tag
echo -e "${CYAN}→${RESET} Creating tag ${TAG}..."
git tag -a "$TAG" -m "Release $TAG - Anna Assistant"

echo -e "${GREEN}✓${RESET} Tag created"

# Push tag
echo -e "${CYAN}→${RESET} Pushing tag to origin..."
git push origin "$TAG"

echo -e "${GREEN}✓${RESET} Tag pushed"

echo
echo -e "${BOLD}${GREEN}╭─────────────────────────────────────────────╮${RESET}"
echo -e "${BOLD}${GREEN}│   Release ${TAG} Published Successfully!     │${RESET}"
echo -e "${BOLD}${GREEN}╰─────────────────────────────────────────────╯${RESET}"
echo
echo -e "${GRAY}Next steps:${RESET}"
echo -e "  ${CYAN}→${RESET} GitHub Actions will build and upload binaries"
echo -e "  ${CYAN}→${RESET} Check: https://github.com/$(git config --get remote.origin.url | sed 's/.*://;s/\.git$//')/releases"
echo -e "  ${CYAN}→${RESET} Users can install with:"
echo -e "    ${GRAY}curl -sSL https://raw.githubusercontent.com/$(git config --get remote.origin.url | sed 's/.*://;s/\.git$//')/main/scripts/install.sh | sudo sh${RESET}"
echo
