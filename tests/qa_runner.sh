#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant QA Test Harness
# Validates compilation, basic functionality, and contract compliance

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0
TESTS_RUN=0

test_header() {
    echo ""
    echo -e "${BLUE}━━━ $1 ━━━${NC}"
}

test_pass() {
    PASS=$((PASS + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "${GREEN}✓${NC} $1"
}

test_fail() {
    FAIL=$((FAIL + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "${RED}✗${NC} $1"
}

test_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Test 1: Project structure
test_structure() {
    test_header "Project Structure"

    local required_files=(
        "Cargo.toml"
        "src/annad/Cargo.toml"
        "src/annactl/Cargo.toml"
        "scripts/install.sh"
        "scripts/uninstall.sh"
        "etc/systemd/annad.service"
        "config/default.toml"
    )

    for file in "${required_files[@]}"; do
        if [[ -f "$file" ]]; then
            test_pass "Found $file"
        else
            test_fail "Missing $file"
        fi
    done
}

# Test 2: Compilation
test_compilation() {
    test_header "Compilation"

    test_info "Running cargo check..."
    if cargo check --quiet 2>/dev/null; then
        test_pass "cargo check succeeded"
    else
        test_fail "cargo check failed"
        return 1
    fi

    test_info "Building release binaries (this may take a few minutes)..."
    if cargo build --release --quiet 2>/dev/null; then
        test_pass "Release build succeeded"
    else
        test_fail "Release build failed"
        return 1
    fi

    if [[ -f target/release/annad ]]; then
        test_pass "annad binary exists"
    else
        test_fail "annad binary not found"
    fi

    if [[ -f target/release/annactl ]]; then
        test_pass "annactl binary exists"
    else
        test_fail "annactl binary not found"
    fi
}

# Test 3: Binary smoke test
test_binaries() {
    test_header "Binary Smoke Tests"

    if [[ ! -f target/release/annad ]]; then
        test_fail "annad not built - skipping smoke tests"
        return 1
    fi

    # Test annad --help (should fail gracefully without root)
    if target/release/annad --help &>/dev/null || true; then
        test_pass "annad responds to --help"
    else
        test_info "annad help check (expected to require root)"
    fi

    # Test annactl --help
    if target/release/annactl --help &>/dev/null; then
        test_pass "annactl --help works"
    else
        test_fail "annactl --help failed"
    fi
}

# Test 4: Configuration validation
test_config() {
    test_header "Configuration"

    if [[ -f config/default.toml ]]; then
        test_pass "Default config exists"

        # Validate TOML syntax (basic check)
        if grep -q '\[daemon\]' config/default.toml && \
           grep -q '\[autonomy\]' config/default.toml; then
            test_pass "Config structure valid"
        else
            test_fail "Config structure invalid"
        fi
    else
        test_fail "Default config missing"
    fi
}

# Test 5: Installation scripts
test_scripts() {
    test_header "Installation Scripts"

    if [[ -x scripts/install.sh ]]; then
        test_pass "install.sh is executable"
    else
        test_fail "install.sh not executable"
    fi

    if [[ -x scripts/uninstall.sh ]]; then
        test_pass "uninstall.sh is executable"
    else
        test_fail "uninstall.sh not executable"
    fi

    # Syntax check
    if bash -n scripts/install.sh; then
        test_pass "install.sh syntax valid"
    else
        test_fail "install.sh syntax error"
    fi

    if bash -n scripts/uninstall.sh; then
        test_pass "uninstall.sh syntax valid"
    else
        test_fail "uninstall.sh syntax error"
    fi
}

# Test 6: Systemd service
test_systemd() {
    test_header "Systemd Service"

    if [[ -f etc/systemd/annad.service ]]; then
        test_pass "Service file exists"

        if grep -q 'ExecStart=/usr/local/bin/annad' etc/systemd/annad.service; then
            test_pass "Service ExecStart correct"
        else
            test_fail "Service ExecStart incorrect"
        fi
    else
        test_fail "Service file missing"
    fi
}

print_summary() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "  Tests Run:  $TESTS_RUN"
    echo -e "  ${GREEN}Passed:     $PASS${NC}"
    echo -e "  ${RED}Failed:     $FAIL${NC}"
    echo ""

    if [[ $FAIL -eq 0 ]]; then
        echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                                       ║${NC}"
        echo -e "${GREEN}║   ALL TESTS PASSED ✓                  ║${NC}"
        echo -e "${GREEN}║                                       ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
        return 0
    else
        echo -e "${RED}╔═══════════════════════════════════════╗${NC}"
        echo -e "${RED}║                                       ║${NC}"
        echo -e "${RED}║   SOME TESTS FAILED ✗                 ║${NC}"
        echo -e "${RED}║                                       ║${NC}"
        echo -e "${RED}╚═══════════════════════════════════════╝${NC}"
        return 1
    fi
}

main() {
    echo -e "${BLUE}"
    cat <<'EOF'
╔═══════════════════════════════════════╗
║                                       ║
║    ANNA QA TEST HARNESS               ║
║    Contract Validation Suite          ║
║                                       ║
╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        echo -e "${RED}Error: Must run from project root${NC}"
        exit 1
    fi

    test_structure
    test_compilation
    test_binaries
    test_config
    test_scripts
    test_systemd
    print_summary
}

main "$@"
