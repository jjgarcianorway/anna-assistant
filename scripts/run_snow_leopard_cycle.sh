#!/bin/bash
# run_snow_leopard_cycle.sh - Snow Leopard Benchmark Full Cycle
#
# v1.5.0: Runs a full Snow Leopard benchmark and displays the results
#
# Usage: ./scripts/run_snow_leopard_cycle.sh [--quick]

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Determine mode
MODE="full"
if [ "$1" = "--quick" ]; then
    MODE="quick"
fi

echo -e "${CYAN}"
echo "  ╔═══════════════════════════════════════════════════════════════╗"
echo "  ║        🐆  Snow Leopard Benchmark - ${MODE^} Cycle               ║"
echo "  ╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if annactl is available
if ! command -v annactl &> /dev/null; then
    echo -e "${YELLOW}Warning: annactl not in PATH. Trying /usr/local/bin/annactl${NC}"
    ANNACTL="/usr/local/bin/annactl"
else
    ANNACTL="annactl"
fi

# Check if daemon is running
echo -e "${CYAN}[1/4]${NC} Checking daemon health..."
if ! $ANNACTL "are you healthy?" 2>/dev/null | grep -qi "healthy\|yes\|good"; then
    echo -e "${YELLOW}Warning: Daemon may not be responding. Continuing anyway...${NC}"
fi

# Run the benchmark
echo ""
echo -e "${CYAN}[2/4]${NC} Running Snow Leopard benchmark (${MODE} mode)..."
echo "      This may take several minutes..."
echo ""

if [ "$MODE" = "quick" ]; then
    RESULT=$($ANNACTL "run a quick snow leopard benchmark" 2>&1) || true
else
    RESULT=$($ANNACTL "run the full snow leopard benchmark" 2>&1) || true
fi

echo "$RESULT"
echo ""

# Wait a moment for results to be saved
sleep 2

# Show the benchmark results
echo -e "${CYAN}[3/4]${NC} Fetching benchmark results..."
echo ""

# Try to show the delta if we have previous runs
$ANNACTL "compare benchmarks" 2>/dev/null || echo "  (No previous benchmark to compare)"
echo ""

# Show full status
echo -e "${CYAN}[4/4]${NC} Full status report..."
echo ""
$ANNACTL status 2>/dev/null || echo "  (Could not fetch status)"

echo ""
echo -e "${GREEN}  ✓  Snow Leopard cycle complete!${NC}"
echo ""
