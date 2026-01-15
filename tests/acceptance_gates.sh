#!/bin/bash
# Anna Acceptance Gates
# This script is run by CI to verify all acceptance criteria.
# Exit code 0 = all gates pass, non-zero = failure

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0

pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "======================================"
echo "  ACCEPTANCE GATES"
echo "  $(date -u)"
echo "======================================"
echo

# ================================================
# GATE 0: Code Quality Gates (400-line limit, etc)
# ================================================
echo "======================================"
echo "  GATE 0: Code Quality Gates"
echo "======================================"
echo

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -x "$SCRIPT_DIR/gates.sh" ]; then
    info "Running code quality gates..."
    if "$SCRIPT_DIR/gates.sh" --line-limit-only; then
        pass "Code quality gates passed"
    else
        fail "Code quality gates failed"
    fi
else
    warn "gates.sh not found, skipping code quality checks"
fi

echo

# ================================================
# GATE A: No Home Writes in Codebase
# ================================================
echo "======================================"
echo "  GATE A: No Home Writes in Codebase"
echo "======================================"
echo

info "Searching for forbidden dirs:: patterns in production code..."

# Search for dirs:: usage - use temp file to avoid pipe issues
TMPFILE=$(mktemp)
grep -rn "dirs::" --include="*.rs" crates/ 2>/dev/null > "$TMPFILE" || true

# Filter out legitimate uses
FORBIDDEN=$(cat "$TMPFILE" | \
    grep -v "_test" | \
    grep -v "migration" | \
    grep -v "migrate" | \
    grep -v "legacy" | \
    grep -v "shell.rs" | \
    grep -v "vim.rs" | \
    grep -v "changes.rs" | \
    grep -v "paths.rs.*//!" | \
    grep -v "safe_ops.rs" || true)

rm -f "$TMPFILE"

# Check if remaining matches are only in legitimate migration/test code
if [ -z "$FORBIDDEN" ]; then
    pass "No forbidden dirs:: patterns in production code"
else
    # Double check - these should only be in migration functions or tests
    TRULY_FORBIDDEN=$(echo "$FORBIDDEN" | grep -v "fn migrate_\|#\[cfg(test)\]" || true)
    if [ -z "$TRULY_FORBIDDEN" ]; then
        pass "dirs:: only in migration/test code (legitimate)"
    else
        fail "Found forbidden dirs:: patterns:"
        echo "$TRULY_FORBIDDEN"
    fi
fi

info "Verifying paths.rs defines system paths..."
if grep -q 'PathBuf::from("/var/lib/anna")' crates/anna-shared/src/paths.rs && \
   grep -q 'PathBuf::from("/etc/anna")' crates/anna-shared/src/paths.rs && \
   grep -q 'PathBuf::from("/run/anna")' crates/anna-shared/src/paths.rs; then
    pass "paths.rs defines correct system paths"
else
    fail "paths.rs missing system path definitions"
fi

info "Verifying state accessors use paths()..."
STATS_USES_PATHS=$(grep -rn "paths().stats_file()" --include="*.rs" crates/ | grep -v "fn stats_file" | grep -v test | wc -l)
TICKETS_USES_PATHS=$(grep -rn "paths().tickets_file()" --include="*.rs" crates/ | grep -v "fn tickets_file" | grep -v test | wc -l)
LEDGER_USES_PATHS=$(grep -rn "paths().update_ledger_file()" --include="*.rs" crates/ | grep -v "fn update_ledger" | grep -v test | wc -l)

if [ "$STATS_USES_PATHS" -gt 0 ] && [ "$TICKETS_USES_PATHS" -gt 0 ] && [ "$LEDGER_USES_PATHS" -gt 0 ]; then
    pass "State accessors use paths() (stats:$STATS_USES_PATHS, tickets:$TICKETS_USES_PATHS, ledger:$LEDGER_USES_PATHS)"
else
    fail "State accessors not using paths()"
fi

echo

# ================================================
# GATE B: Permissions Model
# ================================================
echo "======================================"
echo "  GATE B: Permissions Model"
echo "======================================"
echo

info "Checking directory permissions..."

check_dir_perms() {
    local dir=$1
    local expected_mode=$2
    local expected_group=$3

    if [ ! -d "$dir" ]; then
        warn "$dir does not exist (may be created at runtime)"
        return
    fi

    local actual_mode=$(stat -c "%a" "$dir")
    local actual_group=$(stat -c "%G" "$dir")

    if [ "$actual_mode" = "$expected_mode" ]; then
        pass "$dir mode is $actual_mode"
    else
        fail "$dir mode is $actual_mode, expected $expected_mode"
    fi

    if [ "$actual_group" = "$expected_group" ]; then
        pass "$dir group is $actual_group"
    else
        fail "$dir group is $actual_group, expected $expected_group"
    fi
}

check_dir_perms "/var/lib/anna" "750" "anna"
check_dir_perms "/run/anna" "750" "anna"
check_dir_perms "/var/log/anna" "750" "anna"

info "Checking for world-writable paths..."
WORLD_WRITE=$(find /var/lib/anna /run/anna -perm -o+w 2>/dev/null | head -5 || true)
if [ -z "$WORLD_WRITE" ]; then
    pass "No world-writable paths in /var/lib/anna or /run/anna"
else
    fail "Found world-writable paths: $WORLD_WRITE"
fi

info "Verifying socket permissions in source..."
if grep -q "0o660" crates/annad/src/server/mod.rs; then
    pass "Socket permissions set to 0660 in source"
else
    fail "Socket permissions not set to 0660 in source"
fi

info "Verifying installer uses 750 permissions..."
if grep -q "chmod 750" scripts/install.sh && grep -q "chmod 750" install.sh; then
    pass "Installers use 750 permissions"
else
    fail "Installers not using 750 permissions"
fi

echo

# ================================================
# GATE C: Socket Access Model (Source Verification)
# ================================================
echo "======================================"
echo "  GATE C: Socket Access Model"
echo "======================================"
echo

info "Verifying socket path uses system directory..."
if grep -q "/run/anna" crates/anna-shared/src/paths.rs; then
    pass "Socket path uses /run/anna"
else
    fail "Socket path not using /run/anna"
fi

info "Verifying RuntimeDirectory in systemd service..."
if grep -q "RuntimeDirectory=anna" scripts/install.sh && grep -q "RuntimeDirectoryMode=0750" scripts/install.sh; then
    pass "SystemD service uses RuntimeDirectory with 0750"
else
    fail "SystemD service missing RuntimeDirectory config"
fi

info "Verifying tmpfiles.d config..."
if grep -q "d /run/anna 0750 root anna" scripts/install.sh; then
    pass "tmpfiles.d creates /run/anna with 0750"
else
    fail "tmpfiles.d not configured correctly"
fi

echo

# ================================================
# GATE D: Migration Idempotency (Source Verification)
# ================================================
echo "======================================"
echo "  GATE D: Migration Idempotency"
echo "======================================"
echo

info "Verifying migration tombstone mechanism..."
if grep -q "is_migrated" crates/anna-shared/src/migration.rs && \
   grep -q "write_tombstone" crates/anna-shared/src/migration.rs; then
    pass "Migration uses tombstone for idempotency"
else
    fail "Migration missing tombstone mechanism"
fi

info "Verifying migration merge functions exist..."
MERGE_FUNCS=$(grep -c "fn merge_" crates/anna-shared/src/migration.rs || echo "0")
if [ "$MERGE_FUNCS" -ge 4 ]; then
    pass "Migration has $MERGE_FUNCS merge functions"
else
    fail "Migration missing merge functions (found $MERGE_FUNCS, expected >= 4)"
fi

info "Verifying legacy path detection..."
if grep -q "detect_legacy_paths" crates/anna-shared/src/paths.rs; then
    pass "Legacy path detection exists"
else
    fail "Legacy path detection missing"
fi

echo

# ================================================
# GATE E: Update Mechanism
# ================================================
echo "======================================"
echo "  GATE E: Update Mechanism"
echo "======================================"
echo

info "Verifying update ledger uses system path..."
if grep -q "paths().update_ledger_file()" crates/anna-shared/src/update_ledger.rs; then
    pass "Update ledger uses paths().update_ledger_file()"
else
    fail "Update ledger not using system path"
fi

info "Verifying no user-local ledger references..."
USER_LEDGER=$(grep -rn "home.*update_ledger\|\.anna.*ledger" --include="*.rs" crates/ 2>/dev/null | grep -v test | grep -v migration || true)
if [ -z "$USER_LEDGER" ]; then
    pass "No user-local ledger references in production code"
else
    fail "Found user-local ledger references: $USER_LEDGER"
fi

info "Checking for auto-update in installer notes..."
if grep -qi "auto-update\|auto update" install.sh scripts/install.sh CHANGELOG.md 2>/dev/null; then
    pass "Auto-update mentioned in project files"
else
    warn "Consider adding auto-update documentation"
fi

echo

# ================================================
# GATE F: No Manual Deployment Instructions
# ================================================
echo "======================================"
echo "  GATE F: No Manual sudo cp Deployment"
echo "======================================"
echo

info "Checking for forbidden deployment patterns in docs..."
SUDO_CP_DOCS=$(grep -rn "sudo cp.*annad\|sudo cp.*annactl" *.md docs/ 2>/dev/null | grep -v "CI\|workflow\|test\|acceptance" || true)
if [ -z "$SUDO_CP_DOCS" ]; then
    pass "No sudo cp deployment instructions in docs"
else
    fail "Found manual deployment instructions in docs:"
    echo "$SUDO_CP_DOCS"
fi

echo

# ================================================
# GATE G: UX Golden Tests
# ================================================
echo "======================================"
echo "  GATE G: UX Golden Tests"
echo "======================================"
echo

SCRIPT_DIR_G="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -x "$SCRIPT_DIR_G/ux_golden.sh" ]; then
    info "Running UX golden tests..."
    if "$SCRIPT_DIR_G/ux_golden.sh" >/dev/null 2>&1; then
        pass "UX golden tests passed"
    else
        fail "UX golden tests failed"
    fi
else
    warn "ux_golden.sh not found"
fi

echo

# ================================================
# Summary
# ================================================
echo "======================================"
echo "  SUMMARY"
echo "======================================"
echo
echo -e "Passed: ${GREEN}$PASS_COUNT${NC}"
echo -e "Failed: ${RED}$FAIL_COUNT${NC}"
echo

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}ACCEPTANCE GATES FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL ACCEPTANCE GATES PASSED${NC}"
    exit 0
fi
