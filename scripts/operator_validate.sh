#!/bin/bash
# Anna Assistant - Minimal Operator Validation Script
# Validates fresh installation in under 30 seconds
#
# Usage: curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/operator_validate.sh | bash

set -e

EXPECTED_VERSION="1.16.0-alpha.1"
SOCKET_PATH="/run/anna/anna.sock"
MAX_WAIT=30

# Colors
GREEN='\033[38;5;120m'
RED='\033[38;5;210m'
CYAN='\033[38;5;159m'
RESET='\033[0m'
CHECK="✓"; CROSS="✗"

echo
echo -e "${CYAN}Anna Assistant Operator Validation (v${EXPECTED_VERSION})${RESET}"
echo

# 1. Socket exists under 30s
echo -n "Waiting for socket (max ${MAX_WAIT}s)... "
START=$(date +%s)
while [ ! -S "$SOCKET_PATH" ]; do
    NOW=$(date +%s)
    ELAPSED=$((NOW - START))
    if [ $ELAPSED -ge $MAX_WAIT ]; then
        echo -e "${RED}${CROSS} TIMEOUT${RESET}"
        exit 1
    fi
    sleep 1
done
ELAPSED=$(($(date +%s) - START))
echo -e "${GREEN}${CHECK} OK (${ELAPSED}s)${RESET}"

# 2. annactl version equals 1.16.0-alpha.1
echo -n "Checking version... "
ACTUAL_VERSION=$(annactl --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9\.]+)?' || echo "unknown")
if [ "$ACTUAL_VERSION" = "$EXPECTED_VERSION" ]; then
    echo -e "${GREEN}${CHECK} ${ACTUAL_VERSION}${RESET}"
else
    echo -e "${RED}${CROSS} Got ${ACTUAL_VERSION}, expected ${EXPECTED_VERSION}${RESET}"
    exit 1
fi

# 3. status runs
echo -n "Testing annactl status... "
if annactl status >/dev/null 2>&1; then
    echo -e "${GREEN}${CHECK} OK${RESET}"
else
    echo -e "${RED}${CROSS} FAILED${RESET}"
    exit 1
fi

# 4. health runs
echo -n "Testing annactl health... "
if annactl health >/dev/null 2>&1; then
    echo -e "${GREEN}${CHECK} OK${RESET}"
else
    echo -e "${RED}${CROSS} FAILED${RESET}"
    exit 1
fi

# 5. metrics endpoint responds (if metrics are exposed via HTTP)
echo -n "Checking metrics endpoint... "
if command -v curl >/dev/null 2>&1; then
    # Try localhost:9090/metrics (default Prometheus endpoint)
    if curl -sf http://localhost:9090/metrics >/dev/null 2>&1; then
        echo -e "${GREEN}${CHECK} Responding${RESET}"
    else
        echo -e "${CYAN}⊘ Not exposed (optional)${RESET}"
    fi
else
    echo -e "${CYAN}⊘ curl not available${RESET}"
fi

# 6. self-update path is wired
echo -n "Checking self-update script... "
if [ -f "/usr/local/lib/anna/scripts/self_update.sh" ] && [ -x "/usr/local/lib/anna/scripts/self_update.sh" ]; then
    echo -e "${GREEN}${CHECK} Found${RESET}"
elif [ -f "/opt/anna-assistant/scripts/self_update.sh" ] && [ -x "/opt/anna-assistant/scripts/self_update.sh" ]; then
    echo -e "${GREEN}${CHECK} Found (legacy path)${RESET}"
else
    echo -e "${CYAN}⊘ Not found (manual install?)${RESET}"
fi

echo
echo -e "${GREEN}${CHECK} All critical checks passed${RESET}"
echo
