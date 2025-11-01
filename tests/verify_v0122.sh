#!/bin/bash
# Anna v0.12.2 Comprehensive Verification Script
# Tests everything that doesn't require daemon running

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

pass() {
    echo -e "${GREEN}✓${NC} $1"
    PASS=$((PASS + 1))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    FAIL=$((FAIL + 1))
}

echo "╭─────────────────────────────────────────────╮"
echo "│  Anna v0.12.2 Verification (No Daemon)     │"
echo "╰─────────────────────────────────────────────╯"
echo

# Test 1: Binaries exist
echo "Testing binaries..."
if [[ -f target/release/annad ]]; then
    pass "annad binary exists"
else
    fail "annad binary missing"
fi

if [[ -f target/release/annactl ]]; then
    pass "annactl binary exists"
else
    fail "annactl binary missing"
fi

# Test 2: Version check
echo
echo "Testing version..."
VERSION=$(./target/release/annactl --version 2>&1 | awk '{print $2}')
if [[ "$VERSION" == "0.12.2" ]]; then
    pass "annactl version is 0.12.2"
else
    fail "annactl version is $VERSION (expected 0.12.2)"
fi

# Test 3: Help text includes new commands
echo
echo "Testing CLI structure..."
HELP=$(./target/release/annactl --help)

if echo "$HELP" | grep -q "collect.*Collect telemetry snapshots"; then
    pass "collect command present in help"
else
    fail "collect command missing from help"
fi

if echo "$HELP" | grep -q "classify.*Classify system persona"; then
    pass "classify command present in help"
else
    fail "classify command missing from help"
fi

if echo "$HELP" | grep -q "radar.*Show radar scores"; then
    pass "radar command present in help"
else
    fail "radar command missing from help"
fi

# Test 4: Subcommand help works
echo
echo "Testing subcommand help..."

if ./target/release/annactl collect --help | grep -q "limit.*Number of snapshots"; then
    pass "collect --help shows limit option"
else
    fail "collect --help missing limit option"
fi

if ./target/release/annactl classify --help | grep -q "json.*Output as JSON"; then
    pass "classify --help shows json option"
else
    fail "classify --help missing json option"
fi

if ./target/release/annactl radar show --help | grep -q "json.*Output as JSON"; then
    pass "radar show --help shows json option"
else
    fail "radar show --help missing json option"
fi

# Test 5: Files exist
echo
echo "Testing file structure..."

FILES=(
    "src/annad/src/collectors_v12.rs"
    "src/annad/src/radars_v12.rs"
    "src/annad/src/telemetry_schema_v12.sql"
    "tests/smoke_v0122.sh"
    "docs/V0.12.2-IMPLEMENTATION-SUMMARY.md"
    "docs/CLI-REFERENCE-v0122.md"
    "CHANGELOG_v0122.md"
    "scripts/deploy_v0122.sh"
)

for file in "${FILES[@]}"; do
    if [[ -f "$file" ]]; then
        pass "File exists: $file"
    else
        fail "File missing: $file"
    fi
done

# Test 6: Check file permissions
echo
echo "Testing file permissions..."

EXECUTABLES=(
    "tests/smoke_v0122.sh"
    "scripts/deploy_v0122.sh"
)

for exe in "${EXECUTABLES[@]}"; do
    if [[ -x "$exe" ]]; then
        pass "Executable: $exe"
    else
        fail "Not executable: $exe"
    fi
done

# Test 7: Check for basic code quality
echo
echo "Testing code quality..."

# Check for panics in new code
if grep -r "panic!\|unwrap()\|expect(" src/annad/src/collectors_v12.rs src/annad/src/radars_v12.rs 2>/dev/null | grep -v "unwrap_or" | grep -v "// " >/dev/null; then
    fail "Found panic/unwrap/expect in new code (should use Result)"
else
    pass "No unsafe panic/unwrap in new code"
fi

# Check collectors has graceful error handling
if grep -q "Option<" src/annad/src/collectors_v12.rs; then
    pass "Collectors use Option for missing data"
else
    fail "Collectors missing Option types"
fi

# Check radars return Results
if grep -q "Result" src/annad/src/radars_v12.rs; then
    pass "Radars use Result types"
else
    fail "Radars missing Result types"
fi

# Test 8: Check RPC methods registered
echo
echo "Testing RPC integration..."

if grep -q '"collect".*method_collect' src/annad/src/rpc_v10.rs; then
    pass "RPC method 'collect' registered"
else
    fail "RPC method 'collect' not registered"
fi

if grep -q '"classify".*method_classify' src/annad/src/rpc_v10.rs; then
    pass "RPC method 'classify' registered"
else
    fail "RPC method 'classify' not registered"
fi

if grep -q '"radar_show".*method_radar_show' src/annad/src/rpc_v10.rs; then
    pass "RPC method 'radar_show' registered"
else
    fail "RPC method 'radar_show' not registered"
fi

# Test 9: Check modules are imported
echo
echo "Testing module imports..."

if grep -q "mod collectors_v12;" src/annad/src/main.rs; then
    pass "collectors_v12 module imported"
else
    fail "collectors_v12 module not imported"
fi

if grep -q "mod radars_v12;" src/annad/src/main.rs; then
    pass "radars_v12 module imported"
else
    fail "radars_v12 module not imported"
fi

# Test 10: SQL schema check
echo
echo "Testing SQL schema..."

SCHEMA="src/annad/src/telemetry_schema_v12.sql"
if grep "CREATE TABLE" "$SCHEMA" | grep -q "snapshots"; then
    pass "SQL schema has snapshots table"
else
    fail "SQL schema missing snapshots table"
fi

if grep "CREATE TABLE" "$SCHEMA" | grep -q "radar_scores"; then
    pass "SQL schema has radar_scores table"
else
    fail "SQL schema missing radar_scores table"
fi

if grep "CREATE TABLE" "$SCHEMA" | grep -q "classifications"; then
    pass "SQL schema has classifications table"
else
    fail "SQL schema missing classifications table"
fi

# Summary
echo
echo "╭─────────────────────────────────────────────╮"
echo "│  Verification Results                       │"
echo "╰─────────────────────────────────────────────╯"
echo -e "  Passed: ${GREEN}${PASS}${NC}"
echo -e "  Failed: ${RED}${FAIL}${NC}"
echo

if [[ $FAIL -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
    echo
    echo "Next steps:"
    echo "  1. Deploy: sudo ./scripts/deploy_v0122.sh"
    echo "  2. Run smoke test: sudo ./tests/smoke_v0122.sh"
    echo
    exit 0
else
    echo -e "${RED}Some checks failed!${NC}"
    echo
    exit 1
fi
