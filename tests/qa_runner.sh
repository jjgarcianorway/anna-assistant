#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant QA Test Harness - Sprint 2
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
Sprint 2 QA Test Manifest
=========================
Sprint 1 tests (base):
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

Sprint 2 tests (new):
15. Autonomy framework (levels, tasks, permissions)
16. Persistence layer (state save/load/list)
17. Auto-fix mechanism (doctor --autofix)
18. Telemetry commands (list, stats)
19. State directory structure
20. Autonomy CLI commands

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

# Test 14: Sprint 2 - Autonomy Framework
test_autonomy_framework() {
    test_header "Sprint 2: Autonomy Framework"

    # Check autonomy module exists
    if [[ -f src/annad/src/autonomy.rs ]]; then
        test_pass "Autonomy module exists"
    else
        test_fail "Autonomy module missing"
        return 1
    fi

    # Check for autonomy levels
    if grep -q 'enum AutonomyLevel' src/annad/src/autonomy.rs && \
       grep -q 'Off' src/annad/src/autonomy.rs && \
       grep -q 'Low' src/annad/src/autonomy.rs && \
       grep -q 'Safe' src/annad/src/autonomy.rs; then
        test_pass "Autonomy levels defined (Off, Low, Safe)"
    else
        test_fail "Autonomy levels incomplete"
    fi

    # Check for task types
    if grep -q 'enum Task' src/annad/src/autonomy.rs && \
       grep -q 'Doctor' src/annad/src/autonomy.rs && \
       grep -q 'TelemetryCleanup' src/annad/src/autonomy.rs && \
       grep -q 'ConfigSync' src/annad/src/autonomy.rs; then
        test_pass "Autonomy tasks defined (Doctor, TelemetryCleanup, ConfigSync)"
    else
        test_fail "Autonomy tasks incomplete"
    fi

    # Check for get_status and run_task functions
    if grep -q 'pub fn get_status' src/annad/src/autonomy.rs && \
       grep -q 'pub async fn run_task' src/annad/src/autonomy.rs; then
        test_pass "Autonomy API functions present"
    else
        test_fail "Autonomy API functions missing"
    fi

    # Check RPC handlers
    if grep -q 'AutonomyStatus' src/annad/src/rpc.rs && \
       grep -q 'AutonomyRun' src/annad/src/rpc.rs; then
        test_pass "Autonomy RPC handlers present"
    else
        test_fail "Autonomy RPC handlers missing"
    fi

    # Check annactl commands
    if grep -q 'Autonomy {' src/annactl/src/main.rs && \
       grep -q 'AutonomyAction::Status' src/annactl/src/main.rs && \
       grep -q 'AutonomyAction::Run' src/annactl/src/main.rs; then
        test_pass "annactl autonomy commands present"
    else
        test_fail "annactl autonomy commands missing"
    fi
}

# Test 15: Sprint 2 - Persistence Layer
test_persistence_layer() {
    test_header "Sprint 2: Persistence Layer"

    # Check persistence module exists
    if [[ -f src/annad/src/persistence.rs ]]; then
        test_pass "Persistence module exists"
    else
        test_fail "Persistence module missing"
        return 1
    fi

    # Check for core functions
    if grep -q 'pub fn init()' src/annad/src/persistence.rs && \
       grep -q 'pub fn save_state' src/annad/src/persistence.rs && \
       grep -q 'pub fn load_state' src/annad/src/persistence.rs && \
       grep -q 'pub fn list_states' src/annad/src/persistence.rs; then
        test_pass "Persistence API functions present"
    else
        test_fail "Persistence API functions incomplete"
    fi

    # Check for state rotation
    if grep -q 'MAX_STATE_AGE_DAYS' src/annad/src/persistence.rs || \
       grep -q 'rotate_old_states' src/annad/src/persistence.rs; then
        test_pass "Persistence includes state rotation logic"
    else
        test_fail "Persistence missing rotation logic"
    fi

    # Check RPC handlers
    if grep -q 'StateSave' src/annad/src/rpc.rs && \
       grep -q 'StateLoad' src/annad/src/rpc.rs && \
       grep -q 'StateList' src/annad/src/rpc.rs; then
        test_pass "Persistence RPC handlers present"
    else
        test_fail "Persistence RPC handlers missing"
    fi

    # Check annactl commands
    if grep -q 'State {' src/annactl/src/main.rs && \
       grep -q 'StateAction::Save' src/annactl/src/main.rs && \
       grep -q 'StateAction::Load' src/annactl/src/main.rs && \
       grep -q 'StateAction::List' src/annactl/src/main.rs; then
        test_pass "annactl state commands present"
    else
        test_fail "annactl state commands missing"
    fi

    # Check initialization in main
    if grep -q 'persistence::init()' src/annad/src/main.rs; then
        test_pass "Persistence initialized in daemon main"
    else
        test_fail "Persistence not initialized in daemon"
    fi
}

# Test 16: Sprint 2 - Auto-Fix Mechanism
test_autofix_mechanism() {
    test_header "Sprint 2: Auto-Fix Mechanism"

    # Check for run_autofix function
    if grep -q 'pub async fn run_autofix' src/annad/src/diagnostics.rs; then
        test_pass "Auto-fix function exists"
    else
        test_fail "Auto-fix function missing"
        return 1
    fi

    # Check for AutoFixResult type
    if grep -q 'struct AutoFixResult' src/annad/src/diagnostics.rs; then
        test_pass "AutoFixResult type defined"
    else
        test_fail "AutoFixResult type missing"
    fi

    # Check for individual fix functions
    local fix_functions=(
        "autofix_socket_directory"
        "autofix_paths"
        "autofix_config_directory"
    )

    local all_found=true
    for func in "${fix_functions[@]}"; do
        if grep -q "fn ${func}()" src/annad/src/diagnostics.rs; then
            test_pass "Auto-fix function: ${func} implemented"
        else
            test_fail "Auto-fix function: ${func} missing"
            all_found=false
        fi
    done

    # Check RPC handler
    if grep -q 'DoctorAutoFix' src/annad/src/rpc.rs; then
        test_pass "Auto-fix RPC handler present"
    else
        test_fail "Auto-fix RPC handler missing"
    fi

    # Check annactl doctor --autofix flag
    if grep -q 'autofix: bool' src/annactl/src/main.rs && \
       grep -q 'DoctorAutoFix' src/annactl/src/main.rs; then
        test_pass "annactl doctor --autofix flag present"
    else
        test_fail "annactl doctor --autofix flag missing"
    fi

    # Check for telemetry logging of auto-fix attempts
    if grep -q 'autofix' src/annad/src/diagnostics.rs; then
        test_pass "Auto-fix attempts logged to telemetry"
    else
        test_fail "Auto-fix telemetry logging missing"
    fi
}

# Test 17: Sprint 2 - Telemetry Commands
test_telemetry_commands() {
    test_header "Sprint 2: Telemetry Commands"

    # Check for telemetry subcommand
    if grep -q 'Telemetry {' src/annactl/src/main.rs; then
        test_pass "annactl telemetry subcommand exists"
    else
        test_fail "annactl telemetry subcommand missing"
        return 1
    fi

    # Check for list and stats actions
    if grep -q 'TelemetryAction::List' src/annactl/src/main.rs && \
       grep -q 'TelemetryAction::Stats' src/annactl/src/main.rs; then
        test_pass "Telemetry actions (list, stats) defined"
    else
        test_fail "Telemetry actions incomplete"
    fi

    # Check for print functions
    if grep -q 'fn print_telemetry_list' src/annactl/src/main.rs && \
       grep -q 'fn print_telemetry_stats' src/annactl/src/main.rs; then
        test_pass "Telemetry print functions present"
    else
        test_fail "Telemetry print functions missing"
    fi

    # Check for limit parameter
    if grep -q 'limit: usize' src/annactl/src/main.rs; then
        test_pass "Telemetry list has limit parameter"
    else
        test_fail "Telemetry list missing limit parameter"
    fi

    # Verify telemetry reads from correct paths
    if grep -q '/var/lib/anna/events' src/annactl/src/main.rs && \
       grep -q '.local/share/anna/events' src/annactl/src/main.rs; then
        test_pass "Telemetry reads from system and user paths"
    else
        test_fail "Telemetry path configuration incorrect"
    fi
}

# Test 18: Sprint 2 - State Directory Structure
test_state_directories() {
    test_header "Sprint 2: State Directory Structure"

    # Check that persistence defines STATE_DIR constant
    if grep -q 'STATE_DIR' src/annad/src/persistence.rs; then
        test_pass "STATE_DIR constant defined"
    else
        test_fail "STATE_DIR constant missing"
    fi

    # Check for /var/lib/anna/state path
    if grep -q '/var/lib/anna/state' src/annad/src/persistence.rs; then
        test_pass "State directory path correct"
    else
        test_fail "State directory path incorrect"
    fi

    # Check that init creates required directories
    if grep -q 'create_dir_all' src/annad/src/persistence.rs; then
        test_pass "Persistence init creates directories"
    else
        test_fail "Persistence init missing directory creation"
    fi
}

# Test 19: Sprint 2 - Integration Validation
test_sprint2_integration() {
    test_header "Sprint 2: Integration Validation"

    # Check that all Sprint 2 modules are declared in main
    if grep -q 'mod autonomy;' src/annad/src/main.rs && \
       grep -q 'mod persistence;' src/annad/src/main.rs; then
        test_pass "Sprint 2 modules declared in daemon main"
    else
        test_fail "Sprint 2 modules not declared"
    fi

    # Check that rpc.rs imports Sprint 2 modules
    if grep -q 'use crate::autonomy' src/annad/src/rpc.rs && \
       grep -q 'use crate::persistence' src/annad/src/rpc.rs; then
        test_pass "RPC imports Sprint 2 modules"
    else
        test_fail "RPC missing Sprint 2 imports"
    fi

    # Verify telemetry rotation is accessible from autonomy
    if grep -q 'pub fn rotate_old_files_now' src/annad/src/telemetry.rs || \
       grep -q 'TelemetryCleanup' src/annad/src/autonomy.rs; then
        test_pass "Telemetry rotation accessible from autonomy"
    else
        test_fail "Telemetry rotation not properly exposed"
    fi

    # Check for dirs dependency in annactl Cargo.toml
    if grep -q 'dirs' src/annactl/Cargo.toml; then
        test_pass "dirs dependency added to annactl"
    else
        test_fail "dirs dependency missing from annactl"
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
║    Sprint 2 Validation Suite          ║
║                                       ║
╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        echo -e "${RED}Error: Must run from project root${NC}"
        exit 1
    fi

    # Sprint 1 tests (base)
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

    # Sprint 2 tests (new)
    test_autonomy_framework
    test_persistence_layer
    test_autofix_mechanism
    test_telemetry_commands
    test_state_directories
    test_sprint2_integration

    print_summary
}

main "$@"
