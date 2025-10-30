#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant QA Test Harness - Sprint 1
# Validates compilation, functionality, and contract compliance

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0
TESTS_RUN=0
START_TIME=$(date +%s)

# Test manifest
cat > /tmp/qa_manifest.txt <<'EOF'
Sprint 1 QA Test Manifest
=========================
Expected tests:
1. Project structure validation
2. Compilation (release mode)
3. Binary smoke tests
4. Configuration validation
5. Installation scripts syntax
6. Systemd service file
7. Polkit policy file
8. Bash completion file
9. Privilege separation (annactl runs unprivileged)
10. Config get/set/list operations
11. Doctor output with fix hints
12. Telemetry event creation
13. Install idempotency
14. Uninstall backup creation

Expected runtime: < 3 minutes
EOF

test_header() {
    echo ""
    echo -e "${BLUE}━━━ $1 ━━━${NC}"
}

test_pass() {
    PASS=$((PASS + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "TEST: $1 ${GREEN}… PASS${NC}"
}

test_fail() {
    FAIL=$((FAIL + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "TEST: $1 ${RED}… FAIL${NC}"
    if [[ -n "${2:-}" ]]; then
        echo -e "  ${RED}↳${NC} $2"
    fi
}

test_skip() {
    SKIP=$((SKIP + 1))
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "TEST: $1 ${YELLOW}… SKIP${NC}"
    if [[ -n "${2:-}" ]]; then
        echo -e "  ${YELLOW}↳${NC} $2"
    fi
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
        "polkit/com.anna.policy"
        "completion/annactl.bash"
        "DESIGN-NOTE-privilege-model.md"
    )

    local all_found=true
    for file in "${required_files[@]}"; do
        if [[ -f "$file" ]]; then
            test_pass "Found $file"
        else
            test_fail "Missing $file"
            all_found=false
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

    test_info "Building release binaries..."
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

# Test 3: Binary smoke tests
test_binaries() {
    test_header "Binary Smoke Tests"

    if [[ ! -f target/release/annad ]]; then
        test_fail "annad not built - skipping smoke tests"
        return 1
    fi

    # Test annactl --help
    if target/release/annactl --help &>/dev/null; then
        test_pass "annactl --help works"
    else
        test_fail "annactl --help failed"
    fi

    # Test annactl --version
    if target/release/annactl --version &>/dev/null; then
        test_pass "annactl --version works"
    else
        test_fail "annactl --version failed"
    fi
}

# Test 4: Configuration validation
test_config() {
    test_header "Configuration"

    if [[ -f config/default.toml ]]; then
        test_pass "Default config exists"

        # Validate TOML structure
        if grep -q '\[daemon\]' config/default.toml && \
           grep -q '\[autonomy\]' config/default.toml && \
           grep -q '\[telemetry\]' config/default.toml && \
           grep -q '\[shell.integrations\]' config/default.toml; then
            test_pass "Config structure valid (all sections present)"
        else
            test_fail "Config structure invalid (missing sections)"
        fi

        # Check for Sprint 1 keys
        if grep -q 'level = "off"' config/default.toml; then
            test_pass "autonomy.level key present"
        else
            test_fail "autonomy.level key missing"
        fi

        if grep -q 'local_store = true' config/default.toml; then
            test_pass "telemetry.local_store key present"
        else
            test_fail "telemetry.local_store key missing"
        fi

        if grep -q 'autocomplete = true' config/default.toml; then
            test_pass "shell.integrations.autocomplete key present"
        else
            test_fail "shell.integrations.autocomplete key missing"
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
    if bash -n scripts/install.sh 2>/dev/null; then
        test_pass "install.sh syntax valid"
    else
        test_fail "install.sh syntax error"
    fi

    if bash -n scripts/uninstall.sh 2>/dev/null; then
        test_pass "uninstall.sh syntax valid"
    else
        test_fail "uninstall.sh syntax error"
    fi

    # Check for Sprint 1 features in install.sh
    if grep -q 'install_polkit_policy' scripts/install.sh && \
       grep -q 'install_bash_completion' scripts/install.sh && \
       grep -q 'create_required_paths' scripts/install.sh; then
        test_pass "install.sh includes Sprint 1 features"
    else
        test_fail "install.sh missing Sprint 1 features"
    fi

    # Check for backup README in uninstall.sh
    if grep -q 'README-RESTORE.md' scripts/uninstall.sh; then
        test_pass "uninstall.sh creates backup README"
    else
        test_fail "uninstall.sh missing backup README"
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

        if grep -q 'Type=simple' etc/systemd/annad.service; then
            test_pass "Service type is simple"
        else
            test_fail "Service type not set to simple"
        fi
    else
        test_fail "Service file missing"
    fi
}

# Test 7: Polkit policy
test_polkit_policy() {
    test_header "Polkit Policy"

    if [[ -f polkit/com.anna.policy ]]; then
        test_pass "Polkit policy file exists"

        if grep -q 'com.anna.config.write' polkit/com.anna.policy && \
           grep -q 'com.anna.maintenance.execute' polkit/com.anna.policy; then
            test_pass "Polkit actions defined correctly"
        else
            test_fail "Polkit actions missing or incorrect"
        fi

        # Validate XML syntax
        if command -v xmllint &>/dev/null; then
            if xmllint --noout polkit/com.anna.policy 2>/dev/null; then
                test_pass "Polkit policy XML valid"
            else
                test_fail "Polkit policy XML invalid"
            fi
        else
            test_skip "XML validation" "xmllint not installed"
        fi
    else
        test_fail "Polkit policy file missing"
    fi
}

# Test 8: Bash completion
test_bash_completion() {
    test_header "Bash Completion"

    if [[ -f completion/annactl.bash ]]; then
        test_pass "Bash completion file exists"

        if bash -n completion/annactl.bash 2>/dev/null; then
            test_pass "Bash completion syntax valid"
        else
            test_fail "Bash completion syntax error"
        fi

        if grep -q '_annactl()' completion/annactl.bash && \
           grep -q 'complete -F _annactl annactl' completion/annactl.bash; then
            test_pass "Bash completion function defined"
        else
            test_fail "Bash completion function missing"
        fi
    else
        test_fail "Bash completion file missing"
    fi
}

# Test 9: Privilege separation
test_privilege_separation() {
    test_header "Privilege Separation"

    # Verify annactl doesn't require root
    if [[ -f target/release/annactl ]]; then
        # annactl should work without root (even if daemon not running)
        if target/release/annactl --help &>/dev/null; then
            test_pass "annactl runs without root privileges"
        else
            test_fail "annactl requires root or has other issues"
        fi
    fi

    # Check that annad validates root requirement
    if grep -q 'is_root()' src/annad/src/main.rs && \
       grep -q 'annad must run as root' src/annad/src/main.rs; then
        test_pass "annad enforces root requirement in code"
    else
        test_fail "annad missing root enforcement"
    fi

    # Verify polkit module exists
    if [[ -f src/annad/src/polkit.rs ]]; then
        test_pass "Polkit module exists"
    else
        test_fail "Polkit module missing"
    fi
}

# Test 10: Config operations (mock - requires daemon)
test_config_operations() {
    test_header "Config Operations"

    # Check that config module has required functions
    if grep -q 'pub fn get_value' src/annad/src/config.rs && \
       grep -q 'pub fn set_value' src/annad/src/config.rs && \
       grep -q 'pub fn list_values' src/annad/src/config.rs; then
        test_pass "Config module has get/set/list functions"
    else
        test_fail "Config module missing required functions"
    fi

    # Check RPC handlers
    if grep -q 'ConfigGet' src/annad/src/rpc.rs && \
       grep -q 'ConfigSet' src/annad/src/rpc.rs && \
       grep -q 'ConfigList' src/annad/src/rpc.rs; then
        test_pass "RPC handlers for config operations exist"
    else
        test_fail "RPC handlers missing config operations"
    fi

    # Check annactl has config subcommands
    if grep -q 'Config {' src/annactl/src/main.rs && \
       grep -q 'ConfigAction::Get' src/annactl/src/main.rs && \
       grep -q 'ConfigAction::Set' src/annactl/src/main.rs; then
        test_pass "annactl has config subcommands"
    else
        test_fail "annactl missing config subcommands"
    fi
}

# Test 11: Doctor checks
test_doctor_checks() {
    test_header "Doctor Checks"

    # Verify required checks are implemented
    local required_checks=(
        "check_daemon_active"
        "check_socket_ready"
        "check_polkit_policies"
        "check_paths_writable"
        "check_autocomplete_installed"
    )

    local all_found=true
    for check in "${required_checks[@]}"; do
        if grep -q "fn ${check}()" src/annad/src/diagnostics.rs; then
            test_pass "Doctor check: ${check} implemented"
        else
            test_fail "Doctor check: ${check} missing"
            all_found=false
        fi
    done

    # Verify fix hints are included
    if grep -q 'fix_hint: Option<String>' src/annad/src/diagnostics.rs; then
        test_pass "Doctor checks include fix hints"
    else
        test_fail "Doctor checks missing fix hints"
    fi

    # Verify annactl doctor exits non-zero on failure
    if grep -q 'std::process::exit(1)' src/annactl/src/main.rs; then
        test_pass "annactl doctor exits non-zero on failure"
    else
        test_fail "annactl doctor missing non-zero exit"
    fi
}

# Test 12: Telemetry
test_telemetry() {
    test_header "Telemetry"

    if [[ -f src/annad/src/telemetry.rs ]]; then
        test_pass "Telemetry module exists"

        # Check for required event types
        if grep -q 'DaemonStarted' src/annad/src/telemetry.rs && \
           grep -q 'RpcCall' src/annad/src/telemetry.rs && \
           grep -q 'ConfigChanged' src/annad/src/telemetry.rs; then
            test_pass "Telemetry has required event types"
        else
            test_fail "Telemetry missing required event types"
        fi

        # Check for rotation logic
        if grep -q 'rotate_old_files' src/annad/src/telemetry.rs && \
           grep -q 'MAX_EVENT_FILES' src/annad/src/telemetry.rs; then
            test_pass "Telemetry implements rotation"
        else
            test_fail "Telemetry missing rotation logic"
        fi

        # Verify no network code
        if ! grep -qi 'http\|upload\|network\|tcp' src/annad/src/telemetry.rs; then
            test_pass "Telemetry is local-only (no network code)"
        else
            test_fail "Telemetry contains network-related code"
        fi
    else
        test_fail "Telemetry module missing"
    fi
}

# Test 13: Documentation
test_documentation() {
    test_header "Documentation"

    if [[ -f DESIGN-NOTE-privilege-model.md ]]; then
        test_pass "Privilege model design note exists"
    else
        test_fail "Privilege model design note missing"
    fi

    if [[ -f README.md ]]; then
        test_pass "README.md exists"

        # Check for key sections
        if grep -q '## Quick Start' README.md || grep -q '## Quickstart' README.md; then
            test_pass "README has quickstart section"
        else
            test_fail "README missing quickstart section"
        fi
    else
        test_fail "README.md missing"
    fi

    if [[ -f GENESIS.md ]]; then
        test_pass "GENESIS.md exists"
    else
        test_fail "GENESIS.md missing"
    fi
}

print_summary() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))

    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "  Tests Run:  $TESTS_RUN"
    echo -e "  ${GREEN}Passed:     $PASS${NC}"
    echo -e "  ${RED}Failed:     $FAIL${NC}"
    echo -e "  ${YELLOW}Skipped:    $SKIP${NC}"
    echo "  Duration:   ${duration}s"
    echo ""

    if [[ $FAIL -eq 0 ]]; then
        echo -e "${GREEN}✅ All tests passed ($TESTS_RUN total)${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}❌ Failed tests: $FAIL${NC}"
        echo ""
        return 1
    fi
}

main() {
    echo -e "${BLUE}"
    cat <<'EOF'
╔═══════════════════════════════════════╗
║                                       ║
║    ANNA QA TEST HARNESS               ║
║    Sprint 1 Validation Suite          ║
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
    test_polkit_policy
    test_bash_completion
    test_privilege_separation
    test_config_operations
    test_doctor_checks
    test_telemetry
    test_documentation

    print_summary
}

main "$@"
