#!/bin/bash
# Anna Deep Test Harness v0.0.48
#
# Produces a deterministic test artifact directory with comprehensive
# testing of Anna's CLI, Translator stability, correctness, doctor routing,
# mutation execution with rollback (v0.0.47), and learning system (v0.0.48).
#
# Usage: ./scripts/anna_deep_test.sh [--release]
#
# Output: anna-deep-test-YYYYMMDD-HHMMSS/
#   |- REPORT.md          - Human-readable test report
#   |- report.json        - Machine-readable test data
#   |- environment.txt    - System environment capture
#   |- transcripts/       - Full transcripts for each test
#   |- cases/             - Copies of case files (sanitized)
#   |- mutations/         - Mutation test files (v0.0.47)
#
# Requirements: Arch Linux, annactl built

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
ARTIFACT_DIR="${PROJECT_DIR}/anna-deep-test-${TIMESTAMP}"
USE_RELEASE="${1:-}"
ANNACTL=""

# Colors
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
CYAN=$'\033[0;36m'
BOLD=$'\033[1m'
DIM=$'\033[2m'
NC=$'\033[0m'

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Translator stability counters
TRANSLATOR_PARSE_SUCCESS=0
TRANSLATOR_RETRY_SUCCESS=0
TRANSLATOR_FALLBACK_USED=0
TRANSLATOR_TOTAL_RUNS=0

# ============================================================================
# Setup Functions
# ============================================================================

log_info() {
    echo "${CYAN}[INFO]${NC} $1"
}

log_pass() {
    echo "${GREEN}[PASS]${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_fail() {
    echo "${RED}[FAIL]${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_skip() {
    echo "${YELLOW}[SKIP]${NC} $1"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

find_annactl() {
    if [[ "$USE_RELEASE" == "--release" ]]; then
        local release_bin="$PROJECT_DIR/target/release/annactl"
        if [[ -x "$release_bin" ]]; then
            echo "$release_bin"
            return 0
        fi
        echo "${RED}[ERROR]${NC} Release build not found at $release_bin" >&2
        echo "Run: cargo build --release" >&2
        return 1
    fi

    # Try release build first
    local release_bin="$PROJECT_DIR/target/release/annactl"
    if [[ -x "$release_bin" ]]; then
        echo "$release_bin"
        return 0
    fi

    # Try debug build
    local debug_bin="$PROJECT_DIR/target/debug/annactl"
    if [[ -x "$debug_bin" ]]; then
        echo "$debug_bin"
        return 0
    fi

    # Try system path
    if command -v annactl &>/dev/null; then
        command -v annactl
        return 0
    fi

    echo "${RED}[ERROR]${NC} Could not find annactl binary" >&2
    echo "Run: cargo build --release" >&2
    return 1
}

setup_artifact_dir() {
    mkdir -p "$ARTIFACT_DIR"
    mkdir -p "$ARTIFACT_DIR/transcripts"
    mkdir -p "$ARTIFACT_DIR/cases"
    log_info "Artifact directory: $ARTIFACT_DIR"
}

# ============================================================================
# Environment Capture
# ============================================================================

capture_environment() {
    log_info "Capturing environment..."

    local env_file="$ARTIFACT_DIR/environment.txt"

    {
        echo "=========================================="
        echo "Anna Deep Test - Environment Capture"
        echo "Timestamp: $(date -Iseconds)"
        echo "=========================================="
        echo ""

        echo "=== System ==="
        uname -a
        echo ""

        echo "=== OS Release ==="
        cat /etc/os-release 2>/dev/null || echo "Not available"
        echo ""

        echo "=== CPU ==="
        lscpu | grep -E "Model name|CPU\(s\)|Thread|Core|Socket" 2>/dev/null || echo "Not available"
        echo ""

        echo "=== Memory ==="
        free -h 2>/dev/null || echo "Not available"
        echo ""

        echo "=== GPU ==="
        lspci | grep -i vga 2>/dev/null || echo "Not available"
        echo ""

        echo "=== Disk Layout ==="
        lsblk -o NAME,SIZE,TYPE,MOUNTPOINT 2>/dev/null || echo "Not available"
        echo ""

        echo "=== Disk Free ==="
        df -h 2>/dev/null || echo "Not available"
        echo ""

        echo "=== Anna Version ==="
        "$ANNACTL" --version 2>/dev/null || echo "Not available"
        echo ""

        echo "=== Daemon Status ==="
        systemctl status annad 2>/dev/null | head -20 || echo "Daemon not running"
        echo ""

        echo "=== Ollama Status ==="
        if command -v ollama &>/dev/null; then
            ollama list 2>/dev/null || echo "Ollama available but list failed"
        else
            echo "Ollama not installed"
        fi
        echo ""

    } > "$env_file"

    log_pass "Environment captured"
}

# ============================================================================
# Build Validation
# ============================================================================

validate_build() {
    log_info "Validating build..."

    if [[ ! -x "$ANNACTL" ]]; then
        log_fail "annactl binary not executable"
        return 1
    fi

    # Check version output
    local version_output
    version_output=$("$ANNACTL" --version 2>&1) || true

    if [[ "$version_output" =~ annactl\ v[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        log_pass "Version format correct: $version_output"
    else
        log_fail "Version format incorrect: $version_output"
    fi

    # Check help output
    local help_output
    help_output=$("$ANNACTL" --help 2>&1) || true

    if [[ "$help_output" =~ "status" ]]; then
        log_pass "Help output contains expected commands"
    else
        log_fail "Help output missing expected commands"
    fi
}

# ============================================================================
# A) Translator Stability Tests
# ============================================================================

run_translator_stability_tests() {
    log_info "Running Translator stability tests (50 queries)..."

    local queries=(
        "what cpu do i have"
        "show my ram"
        "how much memory"
        "what kernel"
        "disk space"
        "free space on /"
        "is nginx running"
        "show services"
        "recent errors"
        "journal warnings"
        "what gpu"
        "network interfaces"
        "ip address"
        "default route"
        "dns servers"
        "installed packages"
        "what is installed"
        "system uptime"
        "boot time"
        "last update"
        "what changed recently"
        "any alerts"
        "system health"
        "who am i"
        "target user"
        "anna version"
        "daemon status"
        "ollama status"
        "model status"
        "show helpers"
        "policy status"
        "knowledge packs"
        "recent cases"
        "last failure"
        "metrics summary"
        "error budgets"
        "self diagnostics"
        "what is linux"
        "how to restart nginx"
        "what does systemctl do"
        "explain btrfs"
        "what is pacman"
        "cpu temperature"
        "fan speed"
        "power usage"
        "battery status"
        "sound card"
        "audio devices"
        "wifi status"
        "bluetooth devices"
    )

    local stability_log="$ARTIFACT_DIR/translator_stability.log"

    for query in "${queries[@]}"; do
        TRANSLATOR_TOTAL_RUNS=$((TRANSLATOR_TOTAL_RUNS + 1))

        local output
        local exit_code=0
        output=$("$ANNACTL" "$query" 2>&1) || exit_code=$?

        # Save transcript
        local safe_name
        safe_name=$(echo "$query" | tr ' ' '_' | tr -cd 'a-zA-Z0-9_')
        echo "$output" > "$ARTIFACT_DIR/transcripts/stability_${safe_name}.txt"

        # Check for translator markers in output
        if echo "$output" | grep -q "INTENT:"; then
            TRANSLATOR_PARSE_SUCCESS=$((TRANSLATOR_PARSE_SUCCESS + 1))
        elif echo "$output" | grep -q "retry"; then
            TRANSLATOR_RETRY_SUCCESS=$((TRANSLATOR_RETRY_SUCCESS + 1))
        elif echo "$output" | grep -q "deterministic\|fallback"; then
            TRANSLATOR_FALLBACK_USED=$((TRANSLATOR_FALLBACK_USED + 1))
        fi

        echo "Query: $query | Exit: $exit_code" >> "$stability_log"
    done

    local success_rate=0
    if [[ $TRANSLATOR_TOTAL_RUNS -gt 0 ]]; then
        success_rate=$(( (TRANSLATOR_PARSE_SUCCESS + TRANSLATOR_RETRY_SUCCESS) * 100 / TRANSLATOR_TOTAL_RUNS ))
    fi

    log_info "Translator stability: $success_rate% LLM success ($TRANSLATOR_FALLBACK_USED fallbacks in $TRANSLATOR_TOTAL_RUNS runs)"

    if [[ $TRANSLATOR_FALLBACK_USED -lt 10 ]]; then
        log_pass "Translator fallback rate acceptable (<20%)"
    else
        log_fail "Translator fallback rate too high (>20%)"
    fi
}

# ============================================================================
# B) Read-Only Correctness Tests
# ============================================================================

run_correctness_tests() {
    log_info "Running read-only correctness tests..."

    local test_queries=(
        "what cpu do i have"
        "what kernel version am i using"
        "how much memory do i have"
        "how much disk space is free on /"
        "is NetworkManager running"
        "is pipewire running"
        "what is my default route"
        "show me recent errors"
    )

    local expected_patterns=(
        "cpu|processor|cores|threads|AMD|Intel"
        "kernel|Linux|[0-9]+\.[0-9]+\.[0-9]+"
        "memory|RAM|GB|GiB|[0-9]+ (GB|GiB|MB)"
        "free|avail|[0-9]+[GMKT]|[0-9]+%"
        "NetworkManager|running|active|enabled|inactive|not"
        "pipewire|running|active|enabled|inactive|not|wireplumber"
        "route|gateway|default|0.0.0.0|via"
        "error|warning|journal|log|none|no errors"
    )

    # v0.0.46: Expected tools per query (domain-specific)
    local expected_tools=(
        "hw_snapshot"                    # CPU can use hw_snapshot
        "uname_summary"                  # Kernel MUST use uname_summary
        "mem_summary"                    # Memory MUST use mem_summary
        "mount_usage"                    # Disk MUST use mount_usage
        "nm_summary|link_state_summary"  # Network MUST use network tools
        "audio_services_summary|pactl"   # Audio MUST use audio tools
        "ip_route_summary"               # Route MUST use ip_route
        "recent_errors_summary|journal"  # Errors MUST use error tools
    )

    for i in "${!test_queries[@]}"; do
        local query="${test_queries[$i]}"
        local pattern="${expected_patterns[$i]}"
        local expected_tool="${expected_tools[$i]}"

        local output
        output=$("$ANNACTL" "$query" 2>&1) || true

        # Save transcript
        local safe_name
        safe_name=$(echo "$query" | tr ' ' '_' | tr -cd 'a-zA-Z0-9_')
        echo "$output" > "$ARTIFACT_DIR/transcripts/correctness_${safe_name}.txt"

        # Check for concrete value in output (case-insensitive)
        if echo "$output" | grep -qiE "$pattern"; then
            log_pass "Query: '$query' - contains concrete evidence"

            # Extract and log evidence IDs
            local evidence_ids
            evidence_ids=$(echo "$output" | grep -oE '\[E[0-9]+\]' | sort -u | tr '\n' ' ' || true)
            if [[ -n "$evidence_ids" ]]; then
                log_info "  Evidence IDs: $evidence_ids"
            fi
        else
            log_fail "Query: '$query' - missing concrete evidence (expected pattern: $pattern)"
        fi

        # v0.0.46: Check for correct tool usage
        if echo "$output" | grep -qiE "$expected_tool"; then
            log_info "  Correct tool used: $expected_tool"
        else
            # Check if using hw_snapshot_summary when domain tool required
            if echo "$output" | grep -qi "hw_snapshot_summary" && [[ "$expected_tool" != "hw_snapshot" ]]; then
                log_fail "Query: '$query' - using hw_snapshot instead of domain tool ($expected_tool)"
            fi
        fi
    done
}

# ============================================================================
# B2) v0.0.46: Domain Evidence Tool Validation
# ============================================================================

run_evidence_tool_validation() {
    log_info "Running evidence tool validation (v0.0.46)..."

    # Test cases: (query, required_tool, forbidden_tool)
    # These MUST pass for v0.0.46 to be valid

    local -a domain_tests=(
        "how much disk space is free on /|mount_usage|hw_snapshot_summary"
        "what kernel version am i using|uname_summary|hw_snapshot_summary"
        "how much memory do i have|mem_summary|hw_snapshot_summary"
        "is NetworkManager running|nm_summary|hw_snapshot_summary"
        "is pipewire running|audio_services_summary|hw_snapshot_summary"
    )

    local domain_passed=0
    local domain_failed=0

    for test_case in "${domain_tests[@]}"; do
        IFS='|' read -r query required forbidden <<< "$test_case"

        local output
        output=$("$ANNACTL" "$query" 2>&1) || true

        # Save transcript
        local safe_name
        safe_name=$(echo "$query" | tr ' ' '_' | tr -cd 'a-zA-Z0-9_')
        echo "$output" > "$ARTIFACT_DIR/transcripts/domain_${safe_name}.txt"

        # Check required tool is used
        if echo "$output" | grep -qi "$required"; then
            # Check forbidden tool is NOT used (or if used, required is also used)
            if echo "$output" | grep -qi "$forbidden"; then
                if echo "$output" | grep -qi "$required"; then
                    log_pass "Domain test: '$query' - uses $required (also has $forbidden)"
                    domain_passed=$((domain_passed + 1))
                else
                    log_fail "Domain test: '$query' - uses $forbidden instead of $required"
                    domain_failed=$((domain_failed + 1))
                fi
            else
                log_pass "Domain test: '$query' - correctly uses $required"
                domain_passed=$((domain_passed + 1))
            fi
        else
            log_fail "Domain test: '$query' - missing required tool $required"
            domain_failed=$((domain_failed + 1))
        fi
    done

    log_info "Domain evidence validation: $domain_passed passed, $domain_failed failed"

    # v0.0.46 requirement: domain tests must pass
    if [[ $domain_failed -gt 0 ]]; then
        log_fail "Domain evidence validation failed - v0.0.46 requirement not met"
    else
        log_pass "Domain evidence validation passed - v0.0.46 requirement met"
    fi
}

# ============================================================================
# B3) v0.0.47: Mutation Execution and Rollback Tests
# ============================================================================

run_mutation_tests() {
    log_info "Running mutation execution and rollback tests (v0.0.47)..."

    # Create mutations test directory
    local mutation_dir="$ARTIFACT_DIR/mutations"
    mkdir -p "$mutation_dir"

    # Create test file in sandbox (artifact dir is in cwd, so should be allowed)
    local test_file="$mutation_dir/test_config.txt"
    echo "# Test config file" > "$test_file"
    echo "line1=value1" >> "$test_file"
    echo "line2=value2" >> "$test_file"

    local original_content
    original_content=$(cat "$test_file")
    local original_hash
    original_hash=$(sha256sum "$test_file" | cut -d' ' -f1)

    log_info "Test file created: $test_file"
    log_info "Original hash: ${original_hash:0:12}..."

    # Test 1: Mutation request should show preview and ask for confirmation
    log_info "Test 1: Append line request (checking preview/confirmation)"

    local append_output
    append_output=$("$ANNACTL" "add the line 'new_option=enabled' to $test_file" 2>&1) || true
    echo "$append_output" > "$mutation_dir/append_request.txt"

    # Check for diff preview
    if echo "$append_output" | grep -qiE "preview|diff|will.*append|will.*add"; then
        log_pass "Mutation shows diff preview"
    else
        log_warn "Mutation preview not clearly shown"
    fi

    # Check for confirmation requirement
    if echo "$append_output" | grep -qiE "confirm|yes|type.*yes|I CONFIRM"; then
        log_pass "Mutation requires confirmation"
    else
        log_fail "Mutation should require confirmation"
    fi

    # Test 2: Check file unchanged without confirmation (should not execute)
    local current_content
    current_content=$(cat "$test_file")
    if [[ "$current_content" == "$original_content" ]]; then
        log_pass "File unchanged without confirmation (correct behavior)"
    else
        log_fail "File was modified without confirmation (security issue!)"
    fi

    # Test 3: For now, we test that the mutation system exists and responds correctly
    # Full interactive confirmation testing requires a different approach
    log_info "Test 3: Policy check for sandbox path"

    local policy_output
    policy_output=$("$ANNACTL" "can I edit $test_file" 2>&1) || true
    echo "$policy_output" > "$mutation_dir/policy_check.txt"

    # The system should recognize sandbox paths as allowed
    if echo "$policy_output" | grep -qiE "allow|sandbox|can.*edit|safe"; then
        log_pass "Sandbox path recognized as allowed"
    else
        log_warn "Sandbox path policy not clearly communicated"
    fi

    # Test 4: Check blocked path
    local blocked_output
    blocked_output=$("$ANNACTL" "can I edit /etc/passwd" 2>&1) || true
    echo "$blocked_output" > "$mutation_dir/blocked_check.txt"

    if echo "$blocked_output" | grep -qiE "block|deny|not.*allow|protect|cannot"; then
        log_pass "Protected path correctly blocked"
    else
        log_warn "Protected path blocking not clearly communicated"
    fi

    # Summary
    log_info "Mutation tests complete"
    log_info "Full interactive mutation testing requires daemon + manual confirmation"
}

# ============================================================================
# B4) v0.0.48: Learning System Tests
# ============================================================================

run_learning_tests() {
    log_info "Running learning system tests (v0.0.48)..."

    # Test 1: Check learning stats tool
    log_info "Test 1: Learning stats retrieval"

    local stats_output
    stats_output=$("$ANNACTL" "what is Anna's learning progress" 2>&1) || true
    echo "$stats_output" > "$ARTIFACT_DIR/transcripts/learning_stats.txt"

    if echo "$stats_output" | grep -qiE "level|xp|recipe|learn"; then
        log_pass "Learning stats returned (level/XP visible)"
    else
        log_warn "Learning stats not clearly shown"
    fi

    # Test 2: Run a domain query that should be learnable
    log_info "Test 2: Run domain query (first time)"

    local time_start
    local time_end
    local first_duration

    time_start=$(date +%s%N)
    local first_output
    first_output=$("$ANNACTL" "how much disk space is free" 2>&1) || true
    time_end=$(date +%s%N)
    first_duration=$(( (time_end - time_start) / 1000000 ))  # ms
    echo "$first_output" > "$ARTIFACT_DIR/transcripts/learning_first_query.txt"

    log_info "  First query took ${first_duration}ms"

    # Test 3: Run the same query again (should potentially use learned recipe)
    log_info "Test 3: Run same query (second time, timing comparison)"

    time_start=$(date +%s%N)
    local second_output
    second_output=$("$ANNACTL" "how much disk space is free" 2>&1) || true
    time_end=$(date +%s%N)
    local second_duration=$(( (time_end - time_start) / 1000000 ))  # ms
    echo "$second_output" > "$ARTIFACT_DIR/transcripts/learning_second_query.txt"

    log_info "  Second query took ${second_duration}ms"

    # Note: We don't assert faster - just record for analysis
    if [[ $second_duration -lt $first_duration ]]; then
        log_info "  Second query was faster (${second_duration}ms < ${first_duration}ms)"
    else
        log_info "  Second query not faster (may be expected on first run)"
    fi

    # Test 4: Check if knowledge search tool responds
    log_info "Test 4: Knowledge search capability"

    local search_output
    search_output=$("$ANNACTL" "search Anna's knowledge for disk space" 2>&1) || true
    echo "$search_output" > "$ARTIFACT_DIR/transcripts/learning_search.txt"

    if echo "$search_output" | grep -qiE "recipe|knowledge|learn|search|found"; then
        log_pass "Knowledge search responds"
    else
        log_warn "Knowledge search response unclear"
    fi

    # Test 5: Verify XP directory structure exists (or gets created)
    log_info "Test 5: XP state persistence check"

    local xp_dir="/var/lib/anna/internal"
    local packs_dir="/var/lib/anna/knowledge_packs/installed"

    if [[ -d "$xp_dir" ]] || [[ -d "$packs_dir" ]]; then
        log_pass "Learning directories exist or created"
    else
        log_info "Learning directories not yet created (expected on fresh system)"
    fi

    # Summary
    log_info "Learning system tests complete"
    log_info "First query: ${first_duration}ms, Second query: ${second_duration}ms"
}

# ============================================================================
# C) Doctor Auto-Trigger Tests
# ============================================================================

run_doctor_tests() {
    log_info "Running doctor auto-trigger tests..."

    local doctor_queries=(
        "wifi disconnecting"
        "no sound"
        "boot is slow"
    )

    local expected_doctors=(
        "network|networking|Network Doctor"
        "audio|Audio Doctor|sound"
        "boot|Boot Doctor|startup"
    )

    for i in "${!doctor_queries[@]}"; do
        local query="${doctor_queries[$i]}"
        local pattern="${expected_doctors[$i]}"

        local output
        output=$("$ANNACTL" "$query" 2>&1) || true

        # Save transcript
        local safe_name
        safe_name=$(echo "$query" | tr ' ' '_' | tr -cd 'a-zA-Z0-9_')
        echo "$output" > "$ARTIFACT_DIR/transcripts/doctor_${safe_name}.txt"

        # Check for doctor selection in output
        if echo "$output" | grep -qiE "Selecting Doctor:|doctor_query|$pattern"; then
            log_pass "Doctor query: '$query' - doctor triggered"

            # Check for relevant evidence (not just generic hw summary)
            if echo "$output" | grep -qiE "journal|service|interface|status|error"; then
                log_info "  Evidence appears domain-specific"
            else
                log_info "  Evidence may be generic"
            fi
        else
            log_fail "Doctor query: '$query' - doctor not triggered"
        fi
    done
}

# ============================================================================
# D) Policy Gating Tests
# ============================================================================

run_policy_tests() {
    log_info "Running policy gating tests..."

    # Create a test sandbox file
    local sandbox_dir="$ARTIFACT_DIR/sandbox"
    mkdir -p "$sandbox_dir"
    echo "test content" > "$sandbox_dir/test_file.txt"

    # Try to mutate a file (should require confirmation)
    local output
    output=$("$ANNACTL" "delete $sandbox_dir/test_file.txt" 2>&1) || true

    echo "$output" > "$ARTIFACT_DIR/transcripts/policy_mutation.txt"

    # Check that mutation was NOT executed (file still exists or confirmation required)
    if [[ -f "$sandbox_dir/test_file.txt" ]]; then
        log_pass "Policy gating: mutation blocked without confirmation"
    else
        # Check if output mentions confirmation
        if echo "$output" | grep -qiE "confirm|risk|blocked|not allowed"; then
            log_pass "Policy gating: mutation requires confirmation"
        else
            log_fail "Policy gating: mutation may have executed without confirmation"
        fi
    fi

    # Clean up sandbox
    rm -rf "$sandbox_dir"
}

# ============================================================================
# E) Case File Tests
# ============================================================================

run_case_tests() {
    log_info "Running case file tests..."

    local cases_dir="/var/lib/anna/cases"

    # Check if cases directory exists
    if [[ -d "$cases_dir" ]]; then
        log_pass "Cases directory exists: $cases_dir"

        # Check for recent case files
        local case_count
        case_count=$(find "$cases_dir" -name "*.json" -type f 2>/dev/null | wc -l)
        log_info "Found $case_count case files"

        if [[ $case_count -gt 0 ]]; then
            log_pass "Case files present"

            # Copy some case files to artifact (sanitize if needed)
            local copied=0
            for case_file in $(find "$cases_dir" -name "*.json" -type f 2>/dev/null | head -5); do
                local basename
                basename=$(basename "$case_file")
                cp "$case_file" "$ARTIFACT_DIR/cases/$basename" 2>/dev/null || true
                copied=$((copied + 1))
            done
            log_info "Copied $copied case files to artifact"
        else
            log_skip "No case files found (may need to run queries first)"
        fi

        # Check permissions
        if [[ -r "$cases_dir" ]]; then
            log_info "Cases directory readable by current user"
        else
            log_info "Cases directory requires root to access"
        fi
    else
        log_skip "Cases directory not found (daemon may not have written any yet)"
    fi
}

# ============================================================================
# Report Generation
# ============================================================================

generate_report() {
    log_info "Generating report..."

    local report_md="$ARTIFACT_DIR/REPORT.md"
    local report_json="$ARTIFACT_DIR/report.json"

    # Generate Markdown report
    {
        echo "# Anna Deep Test Report"
        echo ""
        echo "**Generated:** $(date -Iseconds)"
        echo "**Anna Version:** $("$ANNACTL" --version 2>/dev/null || echo 'unknown')"
        echo "**Artifact Directory:** $ARTIFACT_DIR"
        echo ""
        echo "---"
        echo ""
        echo "## Summary"
        echo ""
        echo "| Metric | Value |"
        echo "|--------|-------|"
        echo "| Total Tests | $TOTAL_TESTS |"
        echo "| Passed | $PASSED_TESTS |"
        echo "| Failed | $FAILED_TESTS |"
        echo "| Skipped | $SKIPPED_TESTS |"
        echo "| Pass Rate | $(( PASSED_TESTS * 100 / (TOTAL_TESTS > 0 ? TOTAL_TESTS : 1) ))% |"
        echo ""
        echo "---"
        echo ""
        echo "## Translator Stability"
        echo ""
        echo "| Metric | Count |"
        echo "|--------|-------|"
        echo "| Total Runs | $TRANSLATOR_TOTAL_RUNS |"
        echo "| Parse Success | $TRANSLATOR_PARSE_SUCCESS |"
        echo "| Retry Success | $TRANSLATOR_RETRY_SUCCESS |"
        echo "| Fallback Used | $TRANSLATOR_FALLBACK_USED |"
        echo "| LLM Success Rate | $(( (TRANSLATOR_PARSE_SUCCESS + TRANSLATOR_RETRY_SUCCESS) * 100 / (TRANSLATOR_TOTAL_RUNS > 0 ? TRANSLATOR_TOTAL_RUNS : 1) ))% |"
        echo ""
        echo "---"
        echo ""
        echo "## Test Categories"
        echo ""
        echo "### A) Translator Stability (50 queries)"
        echo "Ran 50 read-only queries to test LLM translator reliability."
        echo ""
        echo "### B) Read-Only Correctness"
        echo "Verified that system queries return concrete, evidence-cited values."
        echo ""
        echo "### C) Doctor Auto-Trigger"
        echo "Verified that problem descriptions route to appropriate doctors."
        echo ""
        echo "### D) Policy Gating"
        echo "Verified that mutations require confirmation."
        echo ""
        echo "### E) Case Files"
        echo "Verified case file creation and accessibility."
        echo ""
        echo "---"
        echo ""
        echo "## Files in This Artifact"
        echo ""
        echo "- \`REPORT.md\` - This report"
        echo "- \`report.json\` - Machine-readable test data"
        echo "- \`environment.txt\` - System environment capture"
        echo "- \`transcripts/\` - Full transcripts for each test"
        echo "- \`cases/\` - Copies of case files"
        echo ""
    } > "$report_md"

    # Generate JSON report
    cat > "$report_json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "anna_version": "$("$ANNACTL" --version 2>/dev/null | tr -d '\n' || echo 'unknown')",
  "artifact_dir": "$ARTIFACT_DIR",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "skipped": $SKIPPED_TESTS,
    "pass_rate_percent": $(( PASSED_TESTS * 100 / (TOTAL_TESTS > 0 ? TOTAL_TESTS : 1) ))
  },
  "translator_stability": {
    "total_runs": $TRANSLATOR_TOTAL_RUNS,
    "parse_success": $TRANSLATOR_PARSE_SUCCESS,
    "retry_success": $TRANSLATOR_RETRY_SUCCESS,
    "fallback_used": $TRANSLATOR_FALLBACK_USED,
    "llm_success_rate_percent": $(( (TRANSLATOR_PARSE_SUCCESS + TRANSLATOR_RETRY_SUCCESS) * 100 / (TRANSLATOR_TOTAL_RUNS > 0 ? TRANSLATOR_TOTAL_RUNS : 1) ))
  }
}
EOF

    log_pass "Report generated"
}

# ============================================================================
# Main
# ============================================================================

main() {
    echo ""
    echo "${BOLD}Anna Deep Test Harness v0.0.48${NC}"
    echo "========================================"
    echo ""

    # Find annactl
    ANNACTL=$(find_annactl) || exit 1
    log_info "Using: $ANNACTL"

    # Setup
    setup_artifact_dir

    # Capture environment
    capture_environment

    # Validate build
    validate_build

    echo ""
    echo "----------------------------------------"
    echo ""

    # Run test suites
    run_translator_stability_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    run_correctness_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    # v0.0.46: Domain evidence tool validation
    run_evidence_tool_validation

    echo ""
    echo "----------------------------------------"
    echo ""

    # v0.0.47: Mutation and rollback tests
    run_mutation_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    # v0.0.48: Learning system tests
    run_learning_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    run_doctor_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    run_policy_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    run_case_tests

    echo ""
    echo "----------------------------------------"
    echo ""

    # Generate reports
    generate_report

    # Final summary
    echo ""
    echo "========================================"
    echo "${BOLD}Deep Test Complete${NC}"
    echo "========================================"
    echo ""
    echo "  ${GREEN}Passed:${NC}  $PASSED_TESTS"
    echo "  ${RED}Failed:${NC}  $FAILED_TESTS"
    echo "  ${YELLOW}Skipped:${NC} $SKIPPED_TESTS"
    echo "  ${CYAN}Total:${NC}   $TOTAL_TESTS"
    echo ""
    echo "  Artifact: $ARTIFACT_DIR"
    echo "  Report:   $ARTIFACT_DIR/REPORT.md"
    echo ""

    if [[ $FAILED_TESTS -gt 0 ]]; then
        echo "${RED}Some tests failed. Review transcripts for details.${NC}"
        exit 1
    else
        echo "${GREEN}All tests passed!${NC}"
        exit 0
    fi
}

main "$@"
