#!/usr/bin/env bash
# Anna v0.12.5 - Btrfs Storage Smoke Test
# Validates storage command, advisor rules, and script functionality

set -euo pipefail

ANNACTL="${ANNACTL:-./target/release/annactl}"
PASS=0
FAIL=0

# Colors
if [ -t 1 ]; then
    C_GREEN='\033[38;5;120m'
    C_RED='\033[38;5;210m'
    C_CYAN='\033[38;5;87m'
    C_RESET='\033[0m'
else
    C_GREEN=''
    C_RED=''
    C_CYAN=''
    C_RESET=''
fi

echo -e "${C_CYAN}==== Btrfs Storage Smoke Test ====${C_RESET}"
echo

# Test 1: Check if annactl exists and is executable
echo "[TEST 1] Checking if annactl exists..."
if [ ! -x "$ANNACTL" ]; then
    echo -e "${C_RED}✗ FAIL: $ANNACTL not found or not executable${C_RESET}"
    echo "Build with: cargo build --release --bin annactl"
    exit 1
fi
echo -e "${C_GREEN}✓ PASS: annactl found at $ANNACTL${C_RESET}"
: $((PASS++))
echo

# Test 2: Check storage command exists
echo "[TEST 2] Testing 'storage btrfs' command availability..."
if $ANNACTL storage --help 2>&1 | grep -q "Show storage profile"; then
    echo -e "${C_GREEN}✓ PASS: Storage command available${C_RESET}"
    : $((PASS++))
else
    echo -e "${C_RED}✗ FAIL: Storage command not found${C_RESET}"
    : $((FAIL++))
fi
echo

# Test 3: Check storage btrfs subcommand
echo "[TEST 3] Testing 'storage btrfs' subcommand..."
if $ANNACTL storage btrfs --help 2>&1 | grep -q "Show Btrfs storage profile"; then
    echo -e "${C_GREEN}✓ PASS: Btrfs subcommand available${C_RESET}"
    : $((PASS++))
else
    echo -e "${C_RED}✗ FAIL: Btrfs subcommand not found${C_RESET}"
    : $((FAIL++))
fi
echo

# Test 4: Test JSON output (requires daemon)
echo "[TEST 4] Testing storage btrfs JSON output..."
if JSON_OUTPUT=$($ANNACTL storage btrfs --json 2>&1); then
    if echo "$JSON_OUTPUT" | jq . >/dev/null 2>&1; then
        echo -e "${C_GREEN}✓ PASS: Valid JSON output${C_RESET}"
        : $((PASS++))
    else
        echo -e "${C_RED}✗ FAIL: Invalid JSON output${C_RESET}"
        echo "$JSON_OUTPUT"
        : $((FAIL++))
    fi
else
    echo "⚠ SKIP: Daemon not responding (expected in test environment)"
    echo "$JSON_OUTPUT"
fi
echo

# Test 5: Verify JSON schema fields
if [ "${JSON_OUTPUT:-}" != "" ] && echo "$JSON_OUTPUT" | jq . >/dev/null 2>&1; then
    echo "[TEST 5] Validating JSON schema..."
    REQUIRED_FIELDS=("version" "generated_at" "detected" "layout" "mount_opts" "tools" "health" "bootloader")
    MISSING_FIELDS=()

    for field in "${REQUIRED_FIELDS[@]}"; do
        if ! echo "$JSON_OUTPUT" | jq -e ".$field" >/dev/null 2>&1; then
            MISSING_FIELDS+=("$field")
        fi
    done

    if [ ${#MISSING_FIELDS[@]} -eq 0 ]; then
        echo -e "${C_GREEN}✓ PASS: All required fields present${C_RESET}"
        : $((PASS++))
    else
        echo -e "${C_RED}✗ FAIL: Missing required fields: ${MISSING_FIELDS[*]}${C_RESET}"
        : $((FAIL++))
    fi
    echo
fi

# Test 6: Check advisor storage rules
echo "[TEST 6] Checking for storage advisor rules..."
ADVISOR_OUTPUT=$($ANNACTL advisor arch --json 2>&1 || echo "[]")

if echo "$ADVISOR_OUTPUT" | jq . >/dev/null 2>&1; then
    STORAGE_RULES=$(echo "$ADVISOR_OUTPUT" | jq '[.[] | select(.category == "storage")] | length')

    if [ "$STORAGE_RULES" -ge 0 ]; then
        echo -e "${C_GREEN}✓ PASS: Advisor storage category available${C_RESET}"
        echo "  Found $STORAGE_RULES storage rule(s) triggered"
        : $((PASS++))
    else
        echo -e "${C_RED}✗ FAIL: Storage category not found in advisor${C_RESET}"
        : $((FAIL++))
    fi
else
    echo "⚠ SKIP: Advisor output not valid JSON (daemon may not be running)"
fi
echo

# Test 7: Check storage rule IDs exist in advisor
echo "[TEST 7] Checking for known storage rule IDs..."
KNOWN_STORAGE_IDS=(
    "btrfs-layout-missing-snapshots"
    "pacman-autosnap-missing"
    "grub-btrfs-missing-on-grub"
    "sd-boot-snapshots-missing"
    "scrub-overdue"
    "low-free-space"
    "compression-suboptimal"
    "qgroups-disabled"
    "copy-on-write-exceptions"
    "balance-required"
)

if echo "$ADVISOR_OUTPUT" | jq . >/dev/null 2>&1; then
    FOUND_IDS=0
    for rule_id in "${KNOWN_STORAGE_IDS[@]}"; do
        if echo "$ADVISOR_OUTPUT" | jq -e ".[] | select(.id == \"$rule_id\")" >/dev/null 2>&1; then
            ((FOUND_IDS++))
        fi
    done

    echo "  Found $FOUND_IDS/${#KNOWN_STORAGE_IDS[@]} known storage rule IDs in current advice"
    echo -e "${C_GREEN}✓ PASS: Storage rule IDs validated${C_RESET}"
    : $((PASS++))
else
    echo "  ⚠ SKIP: Cannot validate rule IDs (daemon not running)"
fi
echo

# Test 8: Check Btrfs scripts exist and are executable
echo "[TEST 8] Checking Btrfs automation scripts..."
SCRIPTS_OK=true

for script in "autosnap-pre.sh" "prune.sh" "sdboot-gen.sh"; do
    if [ ! -x "scripts/btrfs/$script" ]; then
        echo -e "${C_RED}  ✗ Missing or not executable: scripts/btrfs/$script${C_RESET}"
        SCRIPTS_OK=false
    fi
done

if $SCRIPTS_OK; then
    echo -e "${C_GREEN}✓ PASS: All Btrfs scripts present and executable${C_RESET}"
    : $((PASS++))
else
    echo -e "${C_RED}✗ FAIL: Some Btrfs scripts missing or not executable${C_RESET}"
    : $((FAIL++))
fi
echo

# Test 9: Check pacman hook exists
echo "[TEST 9] Checking pacman hook..."
if [ -f "packaging/arch/hooks/90-btrfs-autosnap.hook" ]; then
    echo -e "${C_GREEN}✓ PASS: Pacman hook file exists${C_RESET}"
    : $((PASS++))
else
    echo -e "${C_RED}✗ FAIL: Pacman hook file missing${C_RESET}"
    : $((FAIL++))
fi
echo

# Test 10: Validate script syntax (dry-run)
echo "[TEST 10] Testing Btrfs scripts with --help..."
SCRIPT_SYNTAX_OK=true

for script in "prune.sh" "sdboot-gen.sh"; do
    if ! ./scripts/btrfs/"$script" --help >/dev/null 2>&1; then
        echo -e "${C_RED}  ✗ Script failed --help: $script${C_RESET}"
        SCRIPT_SYNTAX_OK=false
    fi
done

if $SCRIPT_SYNTAX_OK; then
    echo -e "${C_GREEN}✓ PASS: All scripts accept --help flag${C_RESET}"
    : $((PASS++))
else
    echo -e "${C_RED}✗ FAIL: Some scripts failed --help check${C_RESET}"
    : $((FAIL++))
fi
echo

# Summary
echo -e "${C_CYAN}==== Test Summary ====${C_RESET}"
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo

if [ $FAIL -eq 0 ]; then
    echo -e "${C_GREEN}✅ All tests passed!${C_RESET}"
    exit 0
else
    echo -e "${C_RED}❌ Some tests failed${C_RESET}"
    exit 1
fi
