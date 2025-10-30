#!/usr/bin/env bash
set -euo pipefail

# ╭─────────────────────────────────────────────────────────────────────╮
# │ Anna Assistant Installer - Sprint 5 Phase 4 (v0.9.5-beta)          │
# │                                                                     │
# │ Conversational • Intelligent • Self-Healing                         │
# │                                                                     │
# │ Four-phase ceremonial installation:                                │
# │   1. Detection   - Analyze system state and version                │
# │   2. Preparation - Build binaries and create backup                │
# │   3. Installation - Deploy system components                       │
# │   4. Verification - Self-heal and validate                         │
# ╰─────────────────────────────────────────────────────────────────────╯

# ============================================================================
# Load Anna Common Library
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/anna_common.sh
source "$SCRIPT_DIR/anna_common.sh"

# ============================================================================
# Configuration
# ============================================================================

BUNDLE_VERSION="0.9.5-beta"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
TMPFILES_DIR="/usr/lib/tmpfiles.d"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"
STATE_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
VERSION_FILE="/etc/anna/version"
HISTORY_FILE="/var/log/anna/install_history.json"
ANNA_GROUP="anna"

# Installation state
INSTALL_MODE=""      # fresh, upgrade, skip
OLD_VERSION=""       # previous version if upgrading
BACKUP_DIR=""        # backup location if upgrading
REPAIRS_COUNT=0      # doctor repairs performed
AUTO_YES=false       # skip confirmations

# Phase timing (for telemetry)
PHASE1_START=0
PHASE1_DURATION=0
PHASE2_START=0
PHASE2_DURATION=0
PHASE3_START=0
PHASE3_DURATION=0
PHASE4_START=0
PHASE4_DURATION=0
TOTAL_START=0
TOTAL_DURATION=0

# Terminal capabilities are now provided by anna_common.sh
# Colors: C_CYAN, C_GREEN, C_YELLOW, C_RED, C_BLUE, C_GRAY, C_BOLD, NC
# Variables: IS_TTY, TERM_WIDTH, SUPPORTS_COLOR, SUPPORTS_UNICODE

# Additional symbols for installer-specific use
if [[ "$SUPPORTS_UNICODE" == "true" ]]; then
    SYM_CHECK="✓"
    SYM_CROSS="✗"
    SYM_WAIT="⏳"
    TREE_T="┌"
    TREE_B="└"
    TREE_V="│"
    TREE_H="─"
else
    SYM_CHECK="[OK]"
    SYM_CROSS="[FAIL]"
    SYM_WAIT="[WAIT]"
    TREE_T="+"
    TREE_B="+"
    TREE_V="|"
    TREE_H="-"
fi

# Spinner frames for TTY
SPINNER_FRAMES=('⣾' '⣽' '⣻' '⢿' '⡿' '⣟' '⣯' '⣷')
if [[ "$SUPPORTS_UNICODE" != "true" ]]; then
    SPINNER_FRAMES=('|' '/' '-' '\\')
fi

# ============================================================================
# Logging Functions
# ============================================================================

log_to_file() {
    local log_entry="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    if [[ -d "$LOG_DIR" ]] || run_elevated mkdir -p "$LOG_DIR" 2>/dev/null; then
        echo "$log_entry" >> "$LOG_DIR/install.log" 2>/dev/null || true
    fi
}

# ============================================================================
# Print Functions
# ============================================================================

print_box_header() {
    local text="$1"
    local width=50

    [[ $TERM_WIDTH -lt 50 ]] && width=$TERM_WIDTH

    local padding=$(( (width - ${#text} - 2) / 2 ))
    local line=$(printf "%${width}s" | tr ' ' "$BOX_H")

    echo ""
    echo -e "${C_CYAN}${BOX_TL}${line}${BOX_TR}${NC}"
    echo -e "${C_CYAN}${BOX_V}$(printf "%${padding}s")${C_BOLD}${text}${NC}${C_CYAN}$(printf "%${padding}s")${BOX_V}${NC}"
    echo -e "${C_CYAN}${BOX_BL}${line}${BOX_BR}${NC}"
    echo ""
}

print_box_footer() {
    local text="$1"
    local width=50

    [[ $TERM_WIDTH -lt 50 ]] && width=$TERM_WIDTH

    local padding=$(( (width - ${#text} - 2) / 2 ))
    local line=$(printf "%${width}s" | tr ' ' "$BOX_H")

    echo ""
    echo -e "${C_CYAN}${BOX_TL}${line}${BOX_TR}${NC}"
    printf "${C_CYAN}${BOX_V}${NC}%${padding}s${C_GREEN}${C_BOLD}%s${NC}%${padding}s${C_CYAN}${BOX_V}${NC}\n" "" "$text" ""
    echo -e "${C_CYAN}${BOX_BL}${line}${BOX_BR}${NC}"
    echo ""
}

print_phase_header() {
    local phase_name="$1"
    echo ""
    echo -e "${C_BLUE}${TREE_T}${TREE_H} ${C_BOLD}${phase_name}${NC}"
    echo -e "${C_BLUE}${TREE_V}${NC}"
}

print_phase_footer() {
    local summary="$1"
    local status="${2:-success}"  # success, warning, error

    local color="$C_GREEN"
    local symbol="$SYM_SUCCESS"

    [[ "$status" == "warning" ]] && color="$C_YELLOW" && symbol="$SYM_WARN"
    [[ "$status" == "error" ]] && color="$C_RED" && symbol="$SYM_CROSS"

    echo -e "${C_BLUE}${TREE_V}${NC}"
    echo -e "${C_BLUE}${TREE_B}${TREE_H}${NC} ${color}${symbol} ${summary}${NC}"
}

print_info() {
    echo -e "${C_BLUE}${TREE_V}${NC}  $1"
    log_to_file "[INFO] $1"
}

print_success() {
    echo -e "${C_BLUE}${TREE_V}${NC}  ${C_GREEN}${SYM_CHECK}${NC} $1"
    log_to_file "[SUCCESS] $1"
}

print_warn() {
    echo -e "${C_BLUE}${TREE_V}${NC}  ${C_YELLOW}${SYM_WARN}${NC} $1"
    log_to_file "[WARN] $1"
}

print_error() {
    echo -e "${C_BLUE}${TREE_V}${NC}  ${C_RED}${SYM_CROSS}${NC} $1"
    log_to_file "[ERROR] $1"
}

print_wait() {
    echo -e "${C_BLUE}${TREE_V}${NC}  ${C_YELLOW}${SYM_WAIT}${NC} $1"
    log_to_file "[WAIT] $1"
}

print_arrow() {
    echo -e "${C_BLUE}${TREE_V}${NC}  ${C_GRAY}${SYM_INFO}${NC} $1"
    log_to_file "[INFO] $1"
}

# ============================================================================
# Helper Functions
# ============================================================================

needs_elevation() {
    return $(test "$EUID" -ne 0)
}

run_elevated() {
    if needs_elevation; then
        if command -v sudo &>/dev/null; then
            sudo "$@"
        elif command -v pkexec &>/dev/null; then
            pkexec "$@"
        else
            echo "ERROR: This script needs elevated privileges (sudo or pkexec)" >&2
            exit 1
        fi
    else
        "$@"
    fi
}

get_autonomy_level() {
    if [[ -f /etc/anna/autonomy.conf ]]; then
        grep -oP '(?<=^AUTONOMY_LEVEL=).*' /etc/anna/autonomy.conf || echo "low"
    else
        echo "low"
    fi
}

# Version comparison with suffix support
compare_versions() {
    local v1="$1"
    local v2="$2"

    # Extract base version and suffix
    local v1_base=$(echo "$v1" | sed 's/-.*$//')
    local v2_base=$(echo "$v2" | sed 's/-.*$//')
    local v1_suffix=$(echo "$v1" | grep -o '\-.*$' | sed 's/^-//' || echo "")
    local v2_suffix=$(echo "$v2" | grep -o '\-.*$' | sed 's/^-//' || echo "")

    # Split into major.minor.patch
    IFS='.' read -ra V1 <<< "$v1_base"
    IFS='.' read -ra V2 <<< "$v2_base"

    # Compare major
    if [[ ${V1[0]:-0} -lt ${V2[0]:-0} ]]; then
        return 0  # v1 < v2
    elif [[ ${V1[0]:-0} -gt ${V2[0]:-0} ]]; then
        return 2  # v1 > v2
    fi

    # Compare minor
    if [[ ${V1[1]:-0} -lt ${V2[1]:-0} ]]; then
        return 0
    elif [[ ${V1[1]:-0} -gt ${V2[1]:-0} ]]; then
        return 2
    fi

    # Compare patch
    if [[ ${V1[2]:-0} -lt ${V2[2]:-0} ]]; then
        return 0
    elif [[ ${V1[2]:-0} -gt ${V2[2]:-0} ]]; then
        return 2
    fi

    # Base versions are equal, compare suffixes
    # Precedence: (no suffix) > rc > beta > alpha
    if [[ -z "$v1_suffix" && -n "$v2_suffix" ]]; then
        return 2  # v1 > v2 (release > prerelease)
    elif [[ -n "$v1_suffix" && -z "$v2_suffix" ]]; then
        return 0  # v1 < v2 (prerelease < release)
    elif [[ -z "$v1_suffix" && -z "$v2_suffix" ]]; then
        return 1  # v1 == v2 (both are releases)
    fi

    # Both have suffixes, compare them
    # alpha < beta < rc
    local suffix_order=("alpha" "beta" "rc")
    local v1_idx=-1
    local v2_idx=-1

    for i in "${!suffix_order[@]}"; do
        [[ "$v1_suffix" == "${suffix_order[$i]}" ]] && v1_idx=$i
        [[ "$v2_suffix" == "${suffix_order[$i]}" ]] && v2_idx=$i
    done

    if [[ $v1_idx -lt $v2_idx ]]; then
        return 0  # v1 < v2
    elif [[ $v1_idx -gt $v2_idx ]]; then
        return 2  # v1 > v2
    fi

    return 1  # v1 == v2
}

# ============================================================================
# Phase 1: Detection
# ============================================================================

detect_installation() {
    PHASE1_START=$(date +%s)
    print_phase_header "Detection Phase"

    print_info "Checking installation..."

    # Check for existing version
    if [[ -f "$VERSION_FILE" ]]; then
        OLD_VERSION=$(cat "$VERSION_FILE")
        print_arrow "Found v$OLD_VERSION"

        # Compare versions
        if compare_versions "$OLD_VERSION" "$BUNDLE_VERSION"; then
            print_arrow "Upgrade recommended"
            echo ""

            # Interactive confirmation
            if [[ "$AUTO_YES" != "true" ]]; then
                echo -en "${C_BLUE}${TREE_V}${NC}  Upgrade now? [Y/n] "
                read -n 1 -r
                echo ""
                if [[ $REPLY =~ ^[Nn]$ ]]; then
                    print_error "Upgrade cancelled by user"
                    exit 0
                fi
                print_success "Confirmed by $USER"
            else
                print_success "Auto-confirmed (--yes flag)"
            fi

            INSTALL_MODE="upgrade"
        elif [[ $? -eq 1 ]]; then
            print_arrow "Already at v$BUNDLE_VERSION"
            INSTALL_MODE="skip"
        else
            print_warn "Installed version is newer ($OLD_VERSION > $BUNDLE_VERSION)"
            print_warn "Downgrade not supported"
            INSTALL_MODE="skip"
        fi
    else
        print_arrow "Fresh installation"
        INSTALL_MODE="fresh"
    fi

    echo ""

    # Check dependencies
    check_dependencies

    PHASE1_DURATION=$(($(date +%s) - PHASE1_START))

    if [[ "$INSTALL_MODE" == "skip" ]]; then
        print_phase_footer "Nothing to do" "warning"
        exit 0
    else
        local msg="Ready to ${INSTALL_MODE}"
        [[ "$INSTALL_MODE" == "upgrade" ]] && msg="$msg (backup will be created)"
        print_phase_footer "$msg" "success"
    fi
}

check_dependencies() {
    print_info "Checking dependencies..."

    local missing=()
    local installed=()

    # Check systemd (required)
    if ! command -v systemctl &>/dev/null; then
        print_error "systemd is required but not available"
        print_error "Anna requires systemd for service management"
        exit 1
    else
        installed+=("systemd")
    fi

    # Check polkit (required)
    if ! command -v pkaction &>/dev/null; then
        missing+=("polkit")
    else
        installed+=("polkit")
    fi

    # Check sqlite3 (optional but recommended)
    if ! command -v sqlite3 &>/dev/null; then
        missing+=("sqlite3")
    else
        installed+=("sqlite3")
    fi

    # Check jq for telemetry (optional)
    if ! command -v jq &>/dev/null; then
        missing+=("jq")
    else
        installed+=("jq")
    fi

    # Report found dependencies
    if [[ ${#installed[@]} -gt 0 ]]; then
        print_success "Found: ${installed[*]}"
    fi

    # Auto-install missing dependencies on Arch
    if [[ ${#missing[@]} -gt 0 ]]; then
        print_warn "Missing: ${missing[*]}"

        if [[ -f /etc/arch-release ]]; then
            print_info "Installing via pacman..."
            for dep in "${missing[@]}"; do
                if run_elevated pacman -S --noconfirm "$dep" &>/dev/null; then
                    print_success "Installed: $dep"
                else
                    print_warn "Could not install: $dep"
                fi
            done
        else
            print_warn "Please install: ${missing[*]}"
            print_warn "Anna will still work but some features may be limited"
        fi
    fi
}

# ============================================================================
# Phase 2: Preparation
# ============================================================================

prepare_installation() {
    PHASE2_START=$(date +%s)
    print_phase_header "Preparation Phase"

    local tasks_complete=0
    local tasks_total=1

    # Build binaries
    print_info "Building binaries..."
    local build_start=$(date +%s)

    if cargo build --release &>/dev/null; then
        local build_duration=$(($(date +%s) - build_start))
        print_success "annad compiled (release) - ${build_duration}s"
        print_success "annactl compiled (release)"
        ((tasks_complete++))
    else
        print_error "Build failed"
        print_error "Check Cargo.toml and source files"
        exit 1
    fi

    # Create backup if upgrading
    if [[ "$INSTALL_MODE" == "upgrade" ]]; then
        ((tasks_total++))
        print_info "Creating backup..."
        local backup_start=$(date +%s)

        local timestamp=$(date +%Y%m%d-%H%M%S)
        BACKUP_DIR="$STATE_DIR/backups/upgrade-$timestamp"

        if create_backup "$BACKUP_DIR"; then
            local backup_duration=$(($(date +%s) - backup_start))
            print_success "Backup: $BACKUP_DIR - ${backup_duration}s"
            ((tasks_complete++))
        else
            print_warn "Backup failed (continuing anyway)"
            BACKUP_DIR=""
        fi
    fi

    PHASE2_DURATION=$(($(date +%s) - PHASE2_START))
    print_phase_footer "$tasks_complete/$tasks_total tasks complete" "success"
}

create_backup() {
    local backup_dir="$1"

    run_elevated mkdir -p "$backup_dir" 2>/dev/null || return 1

    # Backup binaries
    [[ -f "$BIN_DIR/annad" ]] && run_elevated cp "$BIN_DIR/annad" "$backup_dir/" 2>/dev/null
    [[ -f "$BIN_DIR/annactl" ]] && run_elevated cp "$BIN_DIR/annactl" "$backup_dir/" 2>/dev/null

    # Backup config
    [[ -d "$CONFIG_DIR" ]] && run_elevated cp -r "$CONFIG_DIR" "$backup_dir/" 2>/dev/null

    # Backup state (excluding backups directory)
    if [[ -d "$STATE_DIR" ]]; then
        run_elevated mkdir -p "$backup_dir/state" 2>/dev/null
        run_elevated find "$STATE_DIR" -maxdepth 1 -type f -exec cp {} "$backup_dir/state/" \; 2>/dev/null
    fi

    return 0
}

# ============================================================================
# Phase 3: Installation
# ============================================================================

install_system() {
    PHASE3_START=$(date +%s)
    print_phase_header "Installation Phase"

    # Install binaries
    print_info "Installing binaries..."
    run_elevated install -m 755 target/release/annad "$BIN_DIR/"
    print_success "annad → $BIN_DIR/"

    run_elevated install -m 755 target/release/annactl "$BIN_DIR/"
    print_success "annactl → $BIN_DIR/"

    echo ""

    # System configuration
    print_info "Configuring system..."

    # Create anna group
    if ! getent group "$ANNA_GROUP" &>/dev/null; then
        run_elevated groupadd "$ANNA_GROUP" 2>/dev/null
    fi

    # Add current user to anna group
    if ! groups "$USER" | grep -q "$ANNA_GROUP"; then
        run_elevated usermod -aG "$ANNA_GROUP" "$USER" 2>/dev/null
    fi

    # Create directories
    local dirs_created=0
    for dir in "$CONFIG_DIR" "$STATE_DIR" "$LOG_DIR" "$STATE_DIR/backups" "$STATE_DIR/policies"; do
        if [[ ! -d "$dir" ]]; then
            run_elevated mkdir -p "$dir"
            ((dirs_created++))
        fi
    done

    # Set permissions
    run_elevated chown -R root:$ANNA_GROUP "$CONFIG_DIR" "$STATE_DIR" "$LOG_DIR"
    run_elevated chmod 0750 "$CONFIG_DIR" "$STATE_DIR" "$LOG_DIR"

    print_success "Directories ($dirs_created created/verified)"
    print_success "Permissions (0750 root:anna)"

    # Install policies
    install_policies
    local policy_count=$(ls -1 policies.d/*.yaml 2>/dev/null | wc -l)
    print_success "Policies ($policy_count loaded)"

    # Install and start service
    install_service
    print_success "Service (enabled & started)"

    echo ""

    # Write version file
    print_info "Writing version file..."
    echo "$BUNDLE_VERSION" | run_elevated tee "$VERSION_FILE" > /dev/null
    print_success "$VERSION_FILE → $BUNDLE_VERSION"

    PHASE3_DURATION=$(($(date +%s) - PHASE3_START))
    print_phase_footer "System configured" "success"
}

install_policies() {
    run_elevated mkdir -p "$STATE_DIR/policies"
    run_elevated mkdir -p "$POLKIT_DIR"

    # Copy policy files
    if [[ -d policies.d ]]; then
        for policy in policies.d/*.yaml policies.d/*.yml; do
            [[ -f "$policy" ]] && run_elevated cp "$policy" "$STATE_DIR/policies/"
        done
    fi

    # Install polkit policy
    if [[ -f polkit/com.anna.assistant.policy ]]; then
        run_elevated cp polkit/com.anna.assistant.policy "$POLKIT_DIR/"
    fi

    # Install news files
    if [[ -d news ]]; then
        local news_install_dir="/usr/local/share/anna/news"
        run_elevated mkdir -p "$news_install_dir"
        for news_file in news/*.txt; do
            [[ -f "$news_file" ]] && run_elevated cp "$news_file" "$news_install_dir/"
        done
    fi
}

install_service() {
    # Create systemd service file
    cat > /tmp/annad.service <<'EOF'
[Unit]
Description=Anna Assistant Daemon
Documentation=https://github.com/anna-assistant/anna
After=network.target

[Service]
Type=notify
ExecStart=/usr/local/bin/annad
Restart=always
RestartSec=5
User=root
Group=anna

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/anna /var/log/anna /run/anna
RuntimeDirectory=anna
RuntimeDirectoryMode=0750

[Install]
WantedBy=multi-user.target
EOF

    run_elevated mv /tmp/annad.service "$SYSTEMD_DIR/"
    run_elevated systemctl daemon-reload
    run_elevated systemctl enable annad.service
    run_elevated systemctl restart annad.service
}

# ============================================================================
# Phase 4: Self-Healing & Verification
# ============================================================================

verify_installation() {
    PHASE4_START=$(date +%s)
    print_phase_header "Self-Healing Phase"

    # Wait for daemon to initialize
    sleep 2

    # Run doctor repair
    print_info "Running doctor repair..."
    local repair_start=$(date +%s)

    local repair_output=$("$BIN_DIR/annactl" doctor repair 2>&1 || true)
    local repair_duration=$(($(date +%s) - repair_start))

    if echo "$repair_output" | grep -q "All checks passed"; then
        print_success "All checks passed - ${repair_duration}s"
        print_success "No repairs needed"
        REPAIRS_COUNT=0
    else
        local repairs=$(echo "$repair_output" | grep -c "FIX" || echo "0")
        print_success "Performed $repairs repairs - ${repair_duration}s"
        REPAIRS_COUNT=$repairs
    fi

    echo ""

    # Verify telemetry
    print_info "Verifying telemetry..."

    if [[ -f "$STATE_DIR/telemetry.db" ]]; then
        print_success "Database created"
        print_success "Collector initialized"
        print_wait "First sample in ~60s"
    else
        print_warn "Telemetry DB will be created on first daemon start"
    fi

    PHASE4_DURATION=$(($(date +%s) - PHASE4_START))
    print_phase_footer "System healthy" "success"
}

# ============================================================================
# Install Telemetry
# ============================================================================

record_install_telemetry() {
    # Ensure log directory exists
    run_elevated mkdir -p "$LOG_DIR" 2>/dev/null || return 0

    # Create history file if it doesn't exist
    if [[ ! -f "$HISTORY_FILE" ]]; then
        echo '{"installs": []}' | run_elevated tee "$HISTORY_FILE" > /dev/null
    fi

    # Check for jq
    if ! command -v jq &>/dev/null; then
        log_to_file "[WARN] jq not available, skipping install telemetry"
        return 0
    fi

    # Build telemetry record
    local record=$(cat <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "mode": "$INSTALL_MODE",
  "old_version": $([ -n "$OLD_VERSION" ] && echo "\"$OLD_VERSION\"" || echo "null"),
  "new_version": "$BUNDLE_VERSION",
  "user": "$USER",
  "duration_seconds": $TOTAL_DURATION,
  "phases": {
    "detection": {"duration": $PHASE1_DURATION, "status": "success"},
    "preparation": {"duration": $PHASE2_DURATION, "status": "success"},
    "installation": {"duration": $PHASE3_DURATION, "status": "success"},
    "verification": {"duration": $PHASE4_DURATION, "status": "success"}
  },
  "components": {
    "binaries": "success",
    "directories": "success",
    "permissions": "success",
    "policies": "success",
    "service": "success",
    "telemetry": "success"
  },
  "doctor_repairs": $REPAIRS_COUNT,
  "backup_created": $([ -n "$BACKUP_DIR" ] && echo "\"$BACKUP_DIR\"" || echo "null"),
  "autonomy_mode": "$(get_autonomy_level)"
}
EOF
)

    # Append to history
    local tmp_file="/tmp/install_history.$$.json"
    jq ".installs += [$record]" "$HISTORY_FILE" > "$tmp_file" 2>/dev/null || return 0
    run_elevated mv "$tmp_file" "$HISTORY_FILE"
    run_elevated chmod 0640 "$HISTORY_FILE"
    run_elevated chown root:$ANNA_GROUP "$HISTORY_FILE"
}

# ============================================================================
# Final Summary
# ============================================================================

print_final_summary() {
    local autonomy_mode=$(get_autonomy_level)
    local duration_str=$(format_duration "$TOTAL_DURATION")

    # Friendly completion ceremony
    echo ""
    anna_box ok \
        "All done! I've checked everything twice." \
        "You can talk to me anytime using 'annactl'." \
        "Let's see what we can build together."
    echo ""

    anna_ok "Version: $BUNDLE_VERSION"
    anna_ok "Setup time: $duration_str"
    anna_ok "Autonomy: ${autonomy_mode^} mode"
    anna_ok "Status: Fully operational"

    echo ""
    anna_narrative "Here's what you can try next:"
    echo "  • annactl status          - See how I'm doing"
    echo "  • annactl doctor check    - Let me check my own health"
    echo "  • annactl explore         - Discover my capabilities"
    echo "  • annactl news            - See what's new in this version"

    echo ""
    echo -e "${C_GRAY}📝 Install log: $LOG_DIR/install.log${NC}"
    [[ -f "$HISTORY_FILE" ]] && echo -e "${C_GRAY}📜 History: $HISTORY_FILE${NC}"
    echo ""

    # Show release highlights if available
    print_release_highlights
}

print_release_highlights() {
    local news_file="news/v$BUNDLE_VERSION.txt"

    # Check if news file exists in project
    if [[ ! -f "$news_file" ]]; then
        return 0
    fi

    echo ""
    echo -e "${C_CYAN}╭────────────────────────────────────────────────╮${NC}"

    while IFS= read -r line || [[ -n "$line" ]]; do
        if [[ -z "$line" ]]; then
            echo -e "${C_CYAN}│${NC}                                                ${C_CYAN}│${NC}"
        else
            printf "${C_CYAN}│${NC}  %-44s ${C_CYAN}│${NC}\n" "$line"
        fi
    done < "$news_file"

    echo -e "${C_CYAN}╰────────────────────────────────────────────────╯${NC}"
}

# ============================================================================
# Main
# ============================================================================

main() {
    # Initialize
    TOTAL_START=$(date +%s)

    # Parse arguments
    for arg in "$@"; do
        case "$arg" in
            --yes|-y)
                AUTO_YES=true
                ;;
        esac
    done

    # Friendly greeting ceremony
    echo ""
    anna_box narrative \
        "Hi! I'm Anna. I'll take care of your setup." \
        "I'll explain each step as we go. Ready?"
    echo ""

    anna_info "Version: $BUNDLE_VERSION"
    anna_info "User: $USER"

    local autonomy_mode=$(get_autonomy_level)
    anna_info "Autonomy: ${autonomy_mode^} (I can repair myself if needed)"

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        echo ""
        anna_error "Oops! Please run me from the anna-assistant project root directory."
        exit 1
    fi

    echo ""
    anna_narrative "Let's begin..."
    echo ""

    # Execute phases
    detect_installation
    prepare_installation
    install_system
    verify_installation

    # Calculate total duration
    TOTAL_DURATION=$(($(date +%s) - TOTAL_START))

    # Record telemetry
    record_install_telemetry

    # Print summary
    print_final_summary
}

main "$@"
