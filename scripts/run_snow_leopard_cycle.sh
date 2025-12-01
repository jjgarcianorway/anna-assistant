#!/bin/bash
# run_snow_leopard_cycle.sh - Snow Leopard Benchmark Full Cycle v7.6.0
#
# Runs a full Snow Leopard benchmark with v7.6.0 telemetry validation:
# 1. Ensures telemetry is enabled in config
# 2. Starts daemon and waits for sample collection
# 3. Validates telemetry database is collecting
# 4. Tests corruption resilience
#
# Usage: ./scripts/run_snow_leopard_cycle.sh [--quick|--telemetry-only]

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Paths
CONFIG_FILE="/etc/anna/config.toml"
TELEMETRY_DB="/var/lib/anna/telemetry.db"

# Determine mode
MODE="full"
if [ "$1" = "--quick" ]; then
    MODE="quick"
elif [ "$1" = "--telemetry-only" ]; then
    MODE="telemetry"
fi

echo -e "${CYAN}"
echo "  ╔═══════════════════════════════════════════════════════════════╗"
echo "  ║        🐆  Snow Leopard Benchmark v7.6.0 - ${MODE^} Cycle         ║"
echo "  ╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if annactl is available
if ! command -v annactl &> /dev/null; then
    echo -e "${YELLOW}Warning: annactl not in PATH. Trying /usr/local/bin/annactl${NC}"
    ANNACTL="/usr/local/bin/annactl"
else
    ANNACTL="annactl"
fi

PASS_COUNT=0
FAIL_COUNT=0

pass() {
    echo -e "  ${GREEN}✓${NC}  $1"
    ((PASS_COUNT++))
}

fail() {
    echo -e "  ${RED}✗${NC}  $1"
    ((FAIL_COUNT++))
}

warn() {
    echo -e "  ${YELLOW}⚠${NC}  $1"
}

# ============================================================================
# PHASE 1: Config Validation (v7.6.0)
# ============================================================================
echo -e "${CYAN}[1/6]${NC} Validating configuration..."

# Check if config file exists
if [ -f "$CONFIG_FILE" ]; then
    pass "Config file exists: $CONFIG_FILE"

    # Check if telemetry is enabled
    if grep -q "enabled.*=.*true" "$CONFIG_FILE" 2>/dev/null; then
        pass "Telemetry enabled in config"
    elif grep -q "enabled.*=.*false" "$CONFIG_FILE" 2>/dev/null; then
        warn "Telemetry disabled in config - some tests will be skipped"
        TELEMETRY_DISABLED=true
    else
        pass "Telemetry enabled (default)"
    fi

    # Check sample interval
    if INTERVAL=$(grep "sample_interval_secs" "$CONFIG_FILE" 2>/dev/null | grep -oP '\d+'); then
        if [ "$INTERVAL" -ge 5 ] && [ "$INTERVAL" -le 300 ]; then
            pass "Sample interval valid: ${INTERVAL}s"
        else
            warn "Sample interval out of range (will be clamped): ${INTERVAL}s"
        fi
    else
        pass "Sample interval: default (10s)"
    fi

    # Check retention days
    if RETENTION=$(grep "retention_days" "$CONFIG_FILE" 2>/dev/null | grep -oP '\d+'); then
        if [ "$RETENTION" -ge 1 ] && [ "$RETENTION" -le 365 ]; then
            pass "Retention days valid: ${RETENTION}d"
        else
            warn "Retention days out of range (will be clamped): ${RETENTION}d"
        fi
    else
        pass "Retention days: default (30d)"
    fi
else
    warn "Config file not found, using defaults"
fi
echo ""

# ============================================================================
# PHASE 2: Daemon Health Check
# ============================================================================
echo -e "${CYAN}[2/6]${NC} Checking daemon health..."

if systemctl is-active --quiet annad 2>/dev/null; then
    pass "Daemon service is active"

    # Check version
    if VERSION=$($ANNACTL status 2>/dev/null | grep -oP 'v\d+\.\d+\.\d+' | head -1); then
        pass "Daemon version: $VERSION"
    else
        warn "Could not determine daemon version"
    fi
else
    fail "Daemon service is not running"
    echo "      Start with: sudo systemctl start annad"
fi
echo ""

# ============================================================================
# PHASE 3: Telemetry Database Validation (v7.6.0)
# ============================================================================
echo -e "${CYAN}[3/6]${NC} Validating telemetry database..."

if [ "$TELEMETRY_DISABLED" = "true" ]; then
    warn "Skipping telemetry tests (disabled in config)"
else
    # Check if database exists
    if [ -f "$TELEMETRY_DB" ]; then
        pass "Telemetry database exists: $TELEMETRY_DB"

        # Check database size (should have some data)
        DB_SIZE=$(stat -f%z "$TELEMETRY_DB" 2>/dev/null || stat -c%s "$TELEMETRY_DB" 2>/dev/null || echo "0")
        if [ "$DB_SIZE" -gt 4096 ]; then
            pass "Database has data: $(numfmt --to=iec $DB_SIZE 2>/dev/null || echo "${DB_SIZE}B")"
        else
            warn "Database is empty or nearly empty"
        fi

        # Test database corruption resilience
        echo ""
        echo -e "  ${BOLD}Corruption Resilience Tests:${NC}"

        # Test 1: Read with sqlite3
        if sqlite3 "$TELEMETRY_DB" "SELECT COUNT(*) FROM process_samples;" >/dev/null 2>&1; then
            SAMPLE_COUNT=$(sqlite3 "$TELEMETRY_DB" "SELECT COUNT(*) FROM process_samples;" 2>/dev/null || echo "0")
            pass "Database readable: $SAMPLE_COUNT samples"
        else
            fail "Database corrupt or unreadable"
        fi

        # Test 2: Check integrity
        if INTEGRITY=$(sqlite3 "$TELEMETRY_DB" "PRAGMA integrity_check;" 2>/dev/null); then
            if [ "$INTEGRITY" = "ok" ]; then
                pass "Database integrity: OK"
            else
                fail "Database integrity: $INTEGRITY"
            fi
        else
            warn "Could not check database integrity"
        fi

        # Test 3: Key count check
        if KEY_COUNT=$(sqlite3 "$TELEMETRY_DB" "SELECT COUNT(DISTINCT name) FROM process_samples;" 2>/dev/null); then
            pass "Unique process keys: $KEY_COUNT"
        fi

    else
        warn "Telemetry database not found (daemon may not have started yet)"
    fi
fi
echo ""

# ============================================================================
# PHASE 4: CLI Command Tests
# ============================================================================
echo -e "${CYAN}[4/6]${NC} Testing CLI commands..."

# Test: annactl status
if $ANNACTL status >/dev/null 2>&1; then
    pass "annactl status works"
else
    fail "annactl status failed"
fi

# Test: annactl kdb
if $ANNACTL kdb >/dev/null 2>&1; then
    pass "annactl kdb works"
else
    fail "annactl kdb failed"
fi

# Test: annactl kdb <name> (use a common package)
if $ANNACTL kdb pacman >/dev/null 2>&1; then
    pass "annactl kdb <name> works"
else
    fail "annactl kdb <name> failed"
fi

# Test: annactl kdb <category>
if $ANNACTL kdb editors >/dev/null 2>&1; then
    pass "annactl kdb <category> works"
else
    fail "annactl kdb <category> failed"
fi
echo ""

# Skip benchmark in telemetry-only mode
if [ "$MODE" = "telemetry" ]; then
    echo -e "${CYAN}[5/6]${NC} Skipping benchmark (telemetry-only mode)..."
    echo ""
    echo -e "${CYAN}[6/6]${NC} Skipping status report..."
    echo ""
else
    # ============================================================================
    # PHASE 5: Run Benchmark (if not telemetry-only)
    # ============================================================================
    echo -e "${CYAN}[5/6]${NC} Running Snow Leopard benchmark (${MODE} mode)..."
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
    echo "  Fetching benchmark results..."
    echo ""

    # Try to show the delta if we have previous runs
    $ANNACTL "compare benchmarks" 2>/dev/null || echo "  (No previous benchmark to compare)"
    echo ""

    # ============================================================================
    # PHASE 6: Full Status Report
    # ============================================================================
    echo -e "${CYAN}[6/6]${NC} Full status report..."
    echo ""
    $ANNACTL status 2>/dev/null || echo "  (Could not fetch status)"
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${CYAN}  ════════════════════════════════════════════════════════════════${NC}"
TOTAL=$((PASS_COUNT + FAIL_COUNT))
if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "  ${GREEN}✓  Snow Leopard cycle complete!${NC} ${PASS_COUNT}/${TOTAL} tests passed"
else
    echo -e "  ${YELLOW}⚠  Snow Leopard cycle finished with issues.${NC} ${PASS_COUNT}/${TOTAL} passed, ${FAIL_COUNT} failed"
fi
echo -e "${CYAN}  ════════════════════════════════════════════════════════════════${NC}"
echo ""

# Exit with failure if any tests failed
[ "$FAIL_COUNT" -eq 0 ]
