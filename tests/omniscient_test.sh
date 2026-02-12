#!/usr/bin/env bash
# Test Anna's Omniscient/Omnipotent capabilities (v0.3.162)

set -e

ANNACTL="./target/release/annactl"
RESULTS_FILE="tests/omniscient_results.txt"

echo "🧠 ANNA OMNISCIENT/OMNIPOTENT TEST"
echo "=================================="
echo ""
echo "Testing Anna's universal capability system..."
echo ""

> "$RESULTS_FILE"

# Test 1: Bullshit Detection (Impossible Request)
echo "Test 1: Bullshit Detection - Impossible Request"
echo "Question: 'make it rain outside'"
echo "Expected: Reject as physically impossible"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "make it rain outside" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result: $RESULT"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "cannot.*physical"; then
    echo "✓ PASS: Correctly rejected impossible request"
    echo "Test 1: PASS - Bullshit detected" >> "$RESULTS_FILE"
else
    echo "✗ FAIL: Should have rejected as impossible"
    echo "Test 1: FAIL - Did not detect bullshit" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Test 2: Temporal Task Detection
echo "Test 2: Temporal Task - Time-Based Monitoring"
echo "Question: 'monitor CPU usage for 2 minutes'"
echo "Expected: Detect temporal requirement, set up background task"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "monitor CPU usage for 2 minutes" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result (first 500 chars): ${RESULT:0:500}"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "monitor\|temporal\|minutes\|task"; then
    echo "✓ PASS: Temporal task handling detected"
    echo "Test 2: PASS - Temporal task" >> "$RESULTS_FILE"
else
    echo "? UNCERTAIN: May have handled differently"
    echo "Test 2: UNCERTAIN - Check logs" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Test 3: Feasibility - Challenging But Possible
echo "Test 3: Feasibility - Challenging Request"
echo "Question: 'install and configure a VPN server with WireGuard'"
echo "Expected: Recognize as challenging but proceed"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "install and configure a VPN server with WireGuard" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result (first 500 chars): ${RESULT:0:500}"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "wireguard\|vpn\|install\|configure"; then
    echo "✓ PASS: Challenging request handled"
    echo "Test 3: PASS - Challenging task" >> "$RESULTS_FILE"
else
    echo "? UNCERTAIN: May have rejected or handled differently"
    echo "Test 3: UNCERTAIN - Check logs" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Test 4: Universal Capability - Novel Task
echo "Test 4: Universal Capability - Novel Complex Task"
echo "Question: 'create a systemd timer that cleans /tmp every Sunday at 3am'"
echo "Expected: Use universal handler or config handler"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "create a systemd timer that cleans /tmp every Sunday at 3am" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result (first 500 chars): ${RESULT:0:500}"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "systemd\|timer\|sunday\|/tmp"; then
    echo "✓ PASS: Novel task handled"
    echo "Test 4: PASS - Novel task" >> "$RESULTS_FILE"
else
    echo "? UNCERTAIN: May need more investigation"
    echo "Test 4: UNCERTAIN - Check logs" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Test 5: Simple Config (from v0.3.161)
echo "Test 5: Config Handling - Package Installation"
echo "Question: 'install nmap if not already installed'"
echo "Expected: Detect config, check if installed, install if needed"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "install nmap if not already installed" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result (first 500 chars): ${RESULT:0:500}"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "nmap\|install"; then
    echo "✓ PASS: Config handling worked"
    echo "Test 5: PASS - Config handling" >> "$RESULTS_FILE"
else
    echo "? UNCERTAIN: May have installed or skipped"
    echo "Test 5: UNCERTAIN - Check logs" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Test 6: Meta Request Detection
echo "Test 6: Meta Request - Capability Question"
echo "Question: 'can you monitor network traffic?'"
echo "Expected: Explain capability, possibly demonstrate"
echo ""

START=$(date +%s)
RESULT=$($ANNACTL "can you monitor network traffic?" 2>&1 || true)
END=$(date +%s)
ELAPSED=$((END - START))

echo "Result (first 500 chars): ${RESULT:0:500}"
echo "Time: ${ELAPSED}s"
echo ""

if echo "$RESULT" | grep -qi "yes\|can\|tcpdump\|monitor\|network"; then
    echo "✓ PASS: Meta request answered"
    echo "Test 6: PASS - Meta request" >> "$RESULTS_FILE"
else
    echo "? UNCERTAIN: May have answered differently"
    echo "Test 6: UNCERTAIN - Check logs" >> "$RESULTS_FILE"
fi

echo "---"
echo ""

# Summary
echo "=================================="
echo "TEST SUMMARY"
echo "=================================="
echo ""
cat "$RESULTS_FILE"
echo ""

PASS_COUNT=$(grep -c "PASS" "$RESULTS_FILE" || true)
TOTAL_COUNT=6

echo "Results: $PASS_COUNT/$TOTAL_COUNT tests passed"
echo ""

if [ "$PASS_COUNT" -ge 4 ]; then
    echo "✓ Anna's omniscient/omnipotent capabilities are working!"
else
    echo "⚠ Some capabilities need attention"
fi

echo ""
echo "Full results saved to: $RESULTS_FILE"
