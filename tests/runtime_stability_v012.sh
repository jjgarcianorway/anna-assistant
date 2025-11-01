#!/usr/bin/env bash
# Anna v0.12.1 - Runtime Stability Validation Suite
# Tests socket persistence, latency, and hang prevention across 20 restarts

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
NUM_RESTARTS=20
SOCKET_PATH="/run/anna/annad.sock"
LATENCY_THRESHOLD_MS=500
TIMEOUT_THRESHOLD=0  # Zero timeouts expected

# Metrics
declare -a latencies=()
socket_failures=0
timeout_failures=0
rpc_failures=0
successful_restarts=0

echo "╭────────────────────────────────────────────────────────╮"
echo "│  Anna v0.12.1 - Runtime Stability Validation Suite    │"
echo "│  Testing: $NUM_RESTARTS restarts with latency measurement      │"
echo "╰────────────────────────────────────────────────────────╯"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}✗${NC} This test requires sudo/root privileges"
   echo "  Run: sudo ./tests/runtime_stability_v012.sh"
   exit 1
fi

# Function to measure command execution time in milliseconds
measure_latency() {
    local start=$(date +%s%3N)  # milliseconds
    if "$@" >/dev/null 2>&1; then
        local end=$(date +%s%3N)
        local latency=$((end - start))
        echo "$latency"
        return 0
    else
        local exit_code=$?
        echo "-1"  # Error indicator
        return $exit_code
    fi
}

# Pre-test: Verify initial state
echo "Pre-test verification:"
echo -n "  ⏳ Checking annactl is installed ... "
if command -v annactl &>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC} annactl not found"
    exit 1
fi

echo -n "  ⏳ Checking systemd service exists ... "
if systemctl cat annad >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC} annad.service not found"
    exit 1
fi

echo ""
echo "Starting stability test loop (this will take ~$(( NUM_RESTARTS * 3 )) seconds)..."
echo ""

# Main test loop
for i in $(seq 1 $NUM_RESTARTS); do
    printf "${BLUE}[%2d/%2d]${NC} " "$i" "$NUM_RESTARTS"

    # Step 1: Restart daemon
    printf "Restarting ... "
    if systemctl restart annad 2>/dev/null; then
        printf "${GREEN}✓${NC} "
    else
        printf "${RED}✗${NC} "
        ((rpc_failures++))
        echo ""
        continue
    fi

    # Step 2: Wait for startup
    sleep 1

    # Step 3: Check socket exists
    printf "Socket ... "
    if [[ -S "$SOCKET_PATH" ]]; then
        printf "${GREEN}✓${NC} "
    else
        printf "${RED}✗${NC} "
        ((socket_failures++))
        echo ""
        continue
    fi

    # Step 4: Measure RPC latency
    printf "RPC ... "
    latency=$(measure_latency timeout 3s annactl status)
    exit_code=$?

    if [[ $exit_code -eq 7 ]]; then
        # Timeout detected
        printf "${RED}TIMEOUT${NC} "
        ((timeout_failures++))
        latencies+=(-1)
    elif [[ $exit_code -eq 0 ]] && [[ $latency -ge 0 ]]; then
        # Success
        if [[ $latency -le $LATENCY_THRESHOLD_MS ]]; then
            printf "${GREEN}%4dms${NC} " "$latency"
        else
            printf "${YELLOW}%4dms${NC} " "$latency"
        fi
        latencies+=("$latency")
        ((successful_restarts++))
    else
        # Other error
        printf "${RED}FAIL${NC} "
        ((rpc_failures++))
        latencies+=(-1)
    fi

    # Step 5: Check journalctl for socket ready message
    printf "Log ... "
    if journalctl -u annad --since "3 seconds ago" --no-pager 2>/dev/null | grep -q "RPC socket ready"; then
        printf "${GREEN}✓${NC}"
    else
        printf "${YELLOW}⚠${NC}"
    fi

    echo ""

    # Brief pause between restarts
    sleep 0.5
done

echo ""
echo "╭────────────────────────────────────────────────────────╮"
echo "│  Test Results                                          │"
echo "╰────────────────────────────────────────────────────────╯"
echo ""

# Calculate statistics
total_valid_latencies=0
sum_latencies=0
min_latency=999999
max_latency=0

for lat in "${latencies[@]}"; do
    if [[ $lat -ge 0 ]]; then
        ((total_valid_latencies++))
        ((sum_latencies += lat))
        if [[ $lat -lt $min_latency ]]; then
            min_latency=$lat
        fi
        if [[ $lat -gt $max_latency ]]; then
            max_latency=$lat
        fi
    fi
done

if [[ $total_valid_latencies -gt 0 ]]; then
    avg_latency=$((sum_latencies / total_valid_latencies))
else
    avg_latency=0
fi

# Display metrics
echo "Restart Success:"
echo "  ✓ Successful:      $successful_restarts/$NUM_RESTARTS"
echo "  ✗ Socket failures: $socket_failures"
echo "  ✗ RPC failures:    $rpc_failures"
echo "  ✗ Timeouts:        $timeout_failures"
echo ""

if [[ $total_valid_latencies -gt 0 ]]; then
    echo "Latency Statistics (valid responses only):"
    echo "  Average:  ${avg_latency}ms"
    echo "  Min:      ${min_latency}ms"
    echo "  Max:      ${max_latency}ms"
    echo "  Samples:  $total_valid_latencies"
    echo ""
fi

# Determine pass/fail
pass=true
reason=""

if [[ $successful_restarts -lt $NUM_RESTARTS ]]; then
    pass=false
    failed=$((NUM_RESTARTS - successful_restarts))
    reason="$failed restarts failed"
fi

if [[ $timeout_failures -gt $TIMEOUT_THRESHOLD ]]; then
    pass=false
    if [[ -n "$reason" ]]; then
        reason="$reason, "
    fi
    reason="${reason}$timeout_failures timeouts (threshold: $TIMEOUT_THRESHOLD)"
fi

if [[ $total_valid_latencies -gt 0 ]] && [[ $avg_latency -gt $LATENCY_THRESHOLD_MS ]]; then
    pass=false
    if [[ -n "$reason" ]]; then
        reason="$reason, "
    fi
    reason="${reason}avg latency ${avg_latency}ms > ${LATENCY_THRESHOLD_MS}ms"
fi

echo "╭────────────────────────────────────────────────────────╮"
if [[ "$pass" == true ]]; then
    echo -e "│  ${GREEN}✓ PASS${NC} - Runtime stability validated!               │"
    echo "│    All restarts successful, no hangs detected         │"
    echo "╰────────────────────────────────────────────────────────╯"
    exit 0
else
    echo -e "│  ${RED}✗ FAIL${NC} - Stability issues detected                   │"
    echo "│    Reason: $reason"
    echo "╰────────────────────────────────────────────────────────╯"
    exit 1
fi
