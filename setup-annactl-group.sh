#!/usr/bin/env bash
#
# Anna Assistant - Group Setup Script
#
# This script creates the 'annactl' group and adds the current user to it.
# This is required for the group-based access control feature (Beta.86+).
#
# Usage:
#   sudo ./setup-annactl-group.sh [username]
#
# If username is not provided, uses $SUDO_USER (the user who ran sudo).

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Group name
GROUP_NAME="annactl"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root${NC}"
   echo "Usage: sudo $0 [username]"
   exit 1
fi

# Determine username
if [[ $# -eq 1 ]]; then
    USERNAME="$1"
elif [[ -n "${SUDO_USER:-}" ]]; then
    USERNAME="$SUDO_USER"
else
    echo -e "${RED}Error: Could not determine username${NC}"
    echo "Usage: sudo $0 username"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Anna Assistant - Group Setup            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo

# Check if user exists
if ! id "$USERNAME" &>/dev/null; then
    echo -e "${RED}Error: User '$USERNAME' does not exist${NC}"
    exit 1
fi

echo -e "${YELLOW}This script will:${NC}"
echo "  1. Create '${GROUP_NAME}' group (if it doesn't exist)"
echo "  2. Add user '${USERNAME}' to '${GROUP_NAME}' group"
echo "  3. Configure group for Anna daemon access"
echo
echo -e "${YELLOW}Why is this needed?${NC}"
echo "  Beta.86+ includes group-based access control for security."
echo "  Only users in the '${GROUP_NAME}' group can connect to the daemon."
echo
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

echo

# Step 1: Create group if it doesn't exist
if getent group "$GROUP_NAME" >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Group '${GROUP_NAME}' already exists"
else
    echo -n "Creating group '${GROUP_NAME}'... "
    if groupadd "$GROUP_NAME"; then
        echo -e "${GREEN}✓ Done${NC}"
    else
        echo -e "${RED}✗ Failed${NC}"
        exit 1
    fi
fi

# Step 2: Add user to group
echo -n "Adding user '${USERNAME}' to group '${GROUP_NAME}'... "
if usermod -aG "$GROUP_NAME" "$USERNAME"; then
    echo -e "${GREEN}✓ Done${NC}"
else
    echo -e "${RED}✗ Failed${NC}"
    exit 1
fi

# Step 3: Verify
echo -n "Verifying group membership... "
if groups "$USERNAME" | grep -q "$GROUP_NAME"; then
    echo -e "${GREEN}✓ Success${NC}"
else
    echo -e "${RED}✗ Failed${NC}"
    exit 1
fi

echo
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Setup Complete!                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo
echo -e "${YELLOW}IMPORTANT:${NC} User '${USERNAME}' must log out and back in"
echo "for group membership to take effect."
echo
echo "To verify group membership:"
echo "  1. Log out and log back in"
echo "  2. Run: groups"
echo "  3. You should see '${GROUP_NAME}' in the list"
echo
echo "To test Anna access:"
echo "  annactl status"
echo
echo -e "${BLUE}Group Setup Summary:${NC}"
echo "  Group created:  ${GROUP_NAME}"
echo "  User added:     ${USERNAME}"
echo "  Access level:   Authorized for Anna daemon"
echo
echo -e "${GREEN}All done! Remember to log out and back in.${NC}"
