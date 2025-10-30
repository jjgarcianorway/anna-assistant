# Sprint 5 Phase 3: Beautiful & Intelligent Installer - Implementation Guide

**Baseline Commit:** e0b6ca9 (Phase 2B complete: Telemetry RPC/CLI Integration)
**Target Version:** 0.9.4-beta
**Focus:** UX Polish, Self-Verification, Ceremonial Experience

---

## Context: Why This Matters

The installer is Anna's first impression. Currently it works but feels like a "script dump" with rigid ASCII blocks, inconsistent formatting, and redundant output. Phase 3 transforms it into a **guided ceremony** that:

1. **Inspires confidence** - Clear progress, beautiful formatting
2. **Self-heals proactively** - Runs doctor repair automatically
3. **Tells a story** - Conversational but concise
4. **Records everything** - Persistent install telemetry
5. **Guarantees readiness** - Post-install verification

---

## Objectives

### 1. Visual Beauty & Clarity

**Current Problems:**
```bash
[INFO] Installing Anna Assistant v0.9.4-alpha...
[INFO] Checking for existing installation...
[INFO] Creating directories...
[OK] /etc/anna created
[OK] /var/lib/anna created
[INFO] Copying binaries...
# ... 50 more lines of noise
```

**Target Experience:**
```bash
╭────────────────────────────────────────────────╮
│                                                │
│  🤖 Anna Assistant Installer v0.9.4-beta       │
│     Self-Healing • Autonomous • Intelligent    │
│                                                │
╰────────────────────────────────────────────────╯

Mode: Low Risk (Anna may repair herself)
User: lhoqvso
Time: 2025-10-30 14:09 UTC

┌─ Detection Phase
│
│  Checking installation...
│  → Found v0.9.3-alpha
│  → Upgrade recommended
│
│  Upgrade now? [Y/n] y
│  ✓ Confirmed by lhoqvso
│
└─ ✅ Ready to upgrade (backup will be created)

┌─ Preparation Phase
│
│  Building binaries... ⣾ (12.3s)
│  ✓ annad compiled (release)
│  ✓ annactl compiled (release)
│
│  Creating backup... ⣾ (0.8s)
│  ✓ Backup: /var/lib/anna/backups/upgrade-20251030-140923/
│
└─ ✅ 2/2 tasks complete

┌─ Installation Phase
│
│  Installing binaries...
│  ✓ annad → /usr/local/bin/
│  ✓ annactl → /usr/local/bin/
│
│  Configuring system...
│  ✓ Directories (5 created/verified)
│  ✓ Permissions (0750 root:anna)
│  ✓ Policies (3 loaded)
│  ✓ Service (enabled & started)
│
└─ ✅ 4/4 subsystems ready

┌─ Self-Healing Phase
│
│  Running doctor repair... ⣾ (2.1s)
│  ✓ All checks passed
│  ✓ No repairs needed
│
│  Verifying telemetry...
│  ✓ Database created
│  ✓ Collector initialized
│  ⏳ First sample in ~60s
│
└─ ✅ System healthy

╭────────────────────────────────────────────────╮
│                                                │
│  ✅ Installation Complete                      │
│                                                │
│  Anna is ready to serve!                       │
│                                                │
│  Version:    0.9.4-beta                        │
│  Duration:   18.2s                             │
│  Mode:       Low Risk Autonomy                 │
│  Status:     Fully Operational                 │
│                                                │
│  Next Steps:                                   │
│  • annactl status                              │
│  • annactl telemetry snapshot (after 60s)     │
│  • annactl doctor check                        │
│                                                │
╰────────────────────────────────────────────────╯

Install log: /var/log/anna/install.log
History: /var/log/anna/install_history.json
```

### 2. Adaptive Formatting

**Requirements:**
- Detect terminal width, default to 50 columns for narrow TTYs
- Auto-detect light/dark terminal (use pastel colors for dark, bold for light)
- Graceful degradation for non-TTY (no colors, simple progress)
- Unicode symbols with ASCII fallbacks

**Color Palette (Dark Terminals):**
```bash
# Pastel tones for readability
CYAN='\033[38;5;87m'      # Headers, titles
GREEN='\033[38;5;120m'    # Success
YELLOW='\033[38;5;228m'   # Warnings
RED='\033[38;5;210m'      # Errors
BLUE='\033[38;5;111m'     # Info
GRAY='\033[38;5;245m'     # Secondary text
NC='\033[0m'              # Reset
```

**Symbols:**
```bash
# Unicode with ASCII fallback
SYM_CHECK="${SYM_CHECK:-✓}"      # or [OK]
SYM_CROSS="${SYM_CROSS:-✗}"      # or [FAIL]
SYM_WARN="${SYM_WARN:-⚠}"        # or [WARN]
SYM_INFO="${SYM_INFO:-→}"        # or [INFO]
SYM_WAIT="${SYM_WAIT:-⏳}"        # or [WAIT]
SYM_ROBOT="${SYM_ROBOT:-🤖}"     # or [ANNA]
```

**Progress Indicators:**
```bash
# Animated spinner for TTY
SPINNER_FRAMES=('⣾' '⣽' '⣻' '⢿' '⡿' '⣟' '⣯' '⣷')

# Non-TTY: dots
progress_dots() {
    while kill -0 $1 2>/dev/null; do
        echo -n "."
        sleep 1
    done
    echo ""
}
```

### 3. Phase-Based Structure

**Current:** Linear script with mixed concerns
**Target:** Four distinct phases with clear boundaries

#### Phase 1: Detection & Confirmation
```bash
detect_installation() {
    print_phase_header "Detection Phase"

    # Version detection
    if [[ -f /etc/anna/version ]]; then
        OLD_VERSION=$(cat /etc/anna/version)
        print_info "Found v$OLD_VERSION"
        print_info "Upgrade recommended"
        echo ""

        # Interactive confirmation
        if [[ "$AUTO_YES" != "true" ]]; then
            read -p "Upgrade now? [Y/n] " -n 1 -r
            echo ""
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_error "Upgrade cancelled by user"
                exit 0
            fi
            print_success "Confirmed by $USER"
        fi
        MODE="upgrade"
    else
        print_info "Fresh installation"
        MODE="fresh"
    fi

    print_phase_footer "Ready to ${MODE}" "success"
}
```

#### Phase 2: Preparation
```bash
prepare_installation() {
    print_phase_header "Preparation Phase"

    local tasks_complete=0
    local tasks_total=2

    # Build binaries
    print_info "Building binaries..."
    local start_time=$(date +%s)

    if cargo build --release &>/dev/null; then
        local duration=$(($(date +%s) - start_time))
        print_success "annad compiled (release) - ${duration}s"
        print_success "annactl compiled (release)"
        ((tasks_complete++))
    else
        print_error "Build failed"
        exit 1
    fi

    # Create backup if upgrading
    if [[ "$MODE" == "upgrade" ]]; then
        print_info "Creating backup..."
        start_time=$(date +%s)

        local timestamp=$(date +%Y%m%d-%H%M%S)
        local backup_dir="/var/lib/anna/backups/upgrade-$timestamp"

        create_backup "$backup_dir" &>/dev/null
        duration=$(($(date +%s) - start_time))

        print_success "Backup: $backup_dir - ${duration}s"
        ((tasks_complete++))
    fi

    print_phase_footer "$tasks_complete/$tasks_total tasks complete" "success"
}
```

#### Phase 3: Installation
```bash
install_system() {
    print_phase_header "Installation Phase"

    # Binaries
    print_info "Installing binaries..."
    install -m 755 target/release/annad /usr/local/bin/
    print_success "annad → /usr/local/bin/"

    install -m 755 target/release/annactl /usr/local/bin/
    print_success "annactl → /usr/local/bin/"

    # System configuration
    print_info "Configuring system..."

    setup_directories
    print_success "Directories (5 created/verified)"

    setup_permissions
    print_success "Permissions (0750 root:anna)"

    install_policies
    print_success "Policies (3 loaded)"

    setup_service
    print_success "Service (enabled & started)"

    print_phase_footer "4/4 subsystems ready" "success"
}
```

#### Phase 4: Self-Healing & Verification
```bash
verify_installation() {
    print_phase_header "Self-Healing Phase"

    # Run doctor repair
    print_info "Running doctor repair..."
    local start_time=$(date +%s)

    if annactl doctor repair --dry-run &>/dev/null; then
        local duration=$(($(date +%s) - start_time))
        print_success "All checks passed - ${duration}s"
        print_success "No repairs needed"
    else
        print_warn "Some repairs needed"
        annactl doctor repair
    fi

    # Verify telemetry
    print_info "Verifying telemetry..."

    if [[ -f /var/lib/anna/telemetry.db ]]; then
        print_success "Database created"
        print_success "Collector initialized"
        print_wait "First sample in ~60s"
    else
        print_warn "Telemetry DB will be created on first daemon start"
    fi

    print_phase_footer "System healthy" "success"
}
```

### 4. Install Telemetry

**Purpose:** Track installation history for diagnostics and analytics

**Schema:** `/var/log/anna/install_history.json`
```json
{
  "installs": [
    {
      "timestamp": "2025-10-30T14:09:23Z",
      "mode": "upgrade",
      "old_version": "0.9.3-alpha",
      "new_version": "0.9.4-beta",
      "user": "lhoqvso",
      "duration_seconds": 18.2,
      "phases": {
        "detection": {"duration": 2.1, "status": "success"},
        "preparation": {"duration": 13.1, "status": "success"},
        "installation": {"duration": 1.8, "status": "success"},
        "verification": {"duration": 1.2, "status": "success"}
      },
      "components": {
        "binaries": "success",
        "directories": "success",
        "permissions": "success",
        "policies": "success",
        "service": "success",
        "telemetry": "success"
      },
      "doctor_repairs": 0,
      "backup_created": "/var/lib/anna/backups/upgrade-20251030-140923",
      "autonomy_mode": "low"
    }
  ]
}
```

**Implementation:**
```bash
record_install_telemetry() {
    local history_file="/var/log/anna/install_history.json"

    # Create history file if doesn't exist
    if [[ ! -f "$history_file" ]]; then
        echo '{"installs": []}' > "$history_file"
    fi

    # Build telemetry record
    local record=$(cat <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "mode": "$MODE",
  "old_version": "${OLD_VERSION:-null}",
  "new_version": "$VERSION",
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
  "backup_created": "${BACKUP_DIR:-null}",
  "autonomy_mode": "$(get_autonomy_level)"
}
EOF
)

    # Append to history using jq
    jq ".installs += [$record]" "$history_file" > "${history_file}.tmp"
    mv "${history_file}.tmp" "$history_file"

    chmod 0640 "$history_file"
    chown root:anna "$history_file"
}
```

### 5. Dependency Auto-Installation

**Requirement:** Installer must detect and install missing dependencies automatically

**Dependencies:**
- polkit (required for privilege model)
- sqlite3 (telemetry database - bundled in rusqlite but CLI useful)
- systemd (required for service management)

**Implementation:**
```bash
check_and_install_dependencies() {
    print_info "Checking dependencies..."

    local missing=()
    local installed=()

    # Check polkit
    if ! command -v pkaction &>/dev/null; then
        missing+=("polkit")
    else
        installed+=("polkit")
    fi

    # Check systemd
    if ! command -v systemctl &>/dev/null; then
        print_error "systemd is required but not available"
        print_error "Anna requires systemd for service management"
        exit 1
    else
        installed+=("systemd")
    fi

    # Check sqlite3 (optional but recommended)
    if ! command -v sqlite3 &>/dev/null; then
        missing+=("sqlite3")
    else
        installed+=("sqlite3")
    fi

    # Report status
    if [[ ${#installed[@]} -gt 0 ]]; then
        print_success "Found: ${installed[*]}"
    fi

    if [[ ${#missing[@]} -eq 0 ]]; then
        return 0
    fi

    # Auto-install on Arch
    if [[ -f /etc/arch-release ]]; then
        print_warn "Missing: ${missing[*]}"
        print_info "Installing via pacman..."

        for dep in "${missing[@]}"; do
            if pacman -S --noconfirm "$dep" &>/dev/null; then
                print_success "Installed: $dep"
            else
                print_warn "Could not install $dep (non-fatal)"
            fi
        done
    else
        print_warn "Missing dependencies: ${missing[*]}"
        print_warn "Please install manually for full functionality"
    fi
}
```

### 6. Final Summary

**Requirements:**
- Single-page summary with all key information
- Clear next steps
- Log file locations
- Duration and timing info

**Implementation:**
```bash
print_final_summary() {
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time_global))

    echo ""
    print_box_header "Installation Complete"
    echo ""
    print_centered "$SYM_ROBOT Anna is ready to serve!"
    echo ""
    print_keyvalue "Version" "$VERSION"
    print_keyvalue "Duration" "${total_duration}s"
    print_keyvalue "Mode" "$(get_autonomy_level | tr '[:lower:]' '[:upper:]') Risk Autonomy"
    print_keyvalue "Status" "Fully Operational"
    echo ""
    print_section_header "Next Steps"
    print_bullet "annactl status"
    print_bullet "annactl telemetry snapshot (after 60s)"
    print_bullet "annactl doctor check"
    echo ""
    print_box_footer
    echo ""
    print_info "Install log: /var/log/anna/install.log"
    print_info "History: /var/log/anna/install_history.json"
    echo ""
}
```

---

## Implementation Checklist

### Phase 1: Formatting Infrastructure
- [ ] Add adaptive terminal detection (width, colors, unicode support)
- [ ] Implement print_box_header/footer with rounded corners
- [ ] Implement print_phase_header/footer with tree borders
- [ ] Add animated spinner for TTY, dots for non-TTY
- [ ] Define color palette with light/dark detection
- [ ] Define symbol set with ASCII fallbacks

### Phase 2: Phase-Based Rewrite
- [ ] Extract detect_installation() function
- [ ] Extract prepare_installation() function
- [ ] Extract install_system() function
- [ ] Extract verify_installation() function
- [ ] Add phase timing tracking
- [ ] Add phase status tracking

### Phase 3: Self-Healing Integration
- [ ] Add annactl doctor repair call to verification phase
- [ ] Add telemetry DB verification
- [ ] Add policy loading verification
- [ ] Track repair count for telemetry

### Phase 4: Dependency Management
- [ ] Implement check_and_install_dependencies()
- [ ] Add Arch Linux auto-install support
- [ ] Add graceful fallback for other distros
- [ ] Verify polkit installation

### Phase 5: Install Telemetry
- [ ] Create install_history.json schema
- [ ] Implement record_install_telemetry()
- [ ] Track phase durations
- [ ] Track component status
- [ ] Record user and timestamp

### Phase 6: Final Polish
- [ ] Implement print_final_summary()
- [ ] Add conversational upgrade prompts
- [ ] Test in TTY and non-TTY environments
- [ ] Test on light and dark terminals
- [ ] Test fresh install vs upgrade paths

---

## Acceptance Criteria

1. ✅ **Visual Beauty**
   - Rounded box borders (╭─╮ ╰─╯)
   - Tree-style phase headers (┌─ └─)
   - Aligned symbols and text
   - Consistent spacing and indentation

2. ✅ **Clarity**
   - Each line conveys progress or intent
   - No redundant "Installing..." followed by "[OK] Installed"
   - Section summaries (e.g., "4/4 subsystems ready")
   - Clear phase boundaries

3. ✅ **Personality**
   - "Anna is ready to serve!" (not "Installation complete")
   - "Found v0.9.3-alpha" (not "Detected existing installation")
   - "Upgrade now?" (not "Do you want to continue?")
   - Conversational but professional tone

4. ✅ **Intelligence**
   - Automatic dependency installation
   - Automatic doctor repair
   - Automatic backup creation
   - Post-install verification

5. ✅ **Telemetry**
   - install_history.json created and populated
   - Phase durations recorded
   - Component status tracked
   - User and timestamp logged

6. ✅ **Self-Healing**
   - Doctor repair runs automatically
   - Repairs tracked and reported
   - System verified before completion
   - Clear status indicators

---

## Testing Plan

### Test 1: Fresh Installation
```bash
# Clean system
sudo rm -rf /etc/anna /var/lib/anna /usr/local/bin/anna*

# Run installer
sudo ./scripts/install.sh

# Verify
[[ -f /var/log/anna/install_history.json ]]
jq '.installs[-1].mode' /var/log/anna/install_history.json  # Should be "fresh"
```

### Test 2: Upgrade Installation
```bash
# Fake old installation
echo "0.9.3-alpha" | sudo tee /etc/anna/version

# Run installer
sudo ./scripts/install.sh

# Verify
jq '.installs[-1].mode' /var/log/anna/install_history.json  # Should be "upgrade"
[[ -d /var/lib/anna/backups/upgrade-* ]]
```

### Test 3: Non-TTY Environment
```bash
# Redirect to file (no TTY)
sudo ./scripts/install.sh < /dev/null &> install.log

# Verify no escape codes, no spinners
! grep -q '\033\[' install.log
! grep -q '⣾' install.log
```

### Test 4: Narrow Terminal
```bash
# Force narrow width
COLUMNS=40 sudo ./scripts/install.sh

# Verify graceful formatting
```

### Test 5: Missing Dependencies
```bash
# Hide polkit temporarily
sudo mv /usr/bin/pkaction /usr/bin/pkaction.bak

# Run installer
sudo ./scripts/install.sh

# Should auto-install or warn gracefully
sudo mv /usr/bin/pkaction.bak /usr/bin/pkaction
```

---

## Code Structure

### New Helper Functions

```bash
# Formatting
print_box_header()        # Top border with title
print_box_footer()        # Bottom border
print_phase_header()      # Phase section start
print_phase_footer()      # Phase section end with summary
print_centered()          # Center text in box
print_keyvalue()          # Key: Value formatting
print_bullet()            # Bullet point list item
print_section_header()    # Sub-section header

# Progress
show_spinner()            # Animated spinner (TTY)
show_progress_dots()      # Static dots (non-TTY)
with_spinner()            # Run command with spinner

# Detection
detect_terminal_width()   # Get terminal columns
detect_terminal_colors()  # Detect light/dark mode
has_unicode_support()     # Check for UTF-8

# Telemetry
record_install_telemetry() # Write to history JSON
get_autonomy_level()       # Read from autonomy.conf

# Dependencies
check_and_install_dependencies()  # Auto-install missing deps
```

### File Structure

```
scripts/install.sh              # Main installer
scripts/installer-lib.sh        # Helper functions (new)
/var/log/anna/install.log       # Text log (existing)
/var/log/anna/install_history.json  # Structured telemetry (new)
```

---

## Files to Modify

1. **scripts/install.sh** (~500 lines → ~800 lines)
   - Complete rewrite with phase structure
   - Add all formatting functions
   - Add telemetry recording
   - Add dependency checking

2. **scripts/installer-lib.sh** (NEW, ~300 lines)
   - Extract formatting functions
   - Extract detection functions
   - Extract progress functions
   - Keep install.sh focused on logic

---

## Example Output Walkthrough

```bash
$ sudo ./scripts/install.sh

╭────────────────────────────────────────────────╮
│                                                │
│  🤖 Anna Assistant Installer v0.9.4-beta       │
│     Self-Healing • Autonomous • Intelligent    │
│                                                │
╰────────────────────────────────────────────────╯

Mode: Low Risk (Anna may repair herself)
User: lhoqvso
Time: 2025-10-30 14:09 UTC

┌─ Detection Phase
│
│  Checking installation...
│  → Found v0.9.3-alpha
│  → Upgrade recommended
│
│  Upgrade now? [Y/n] y
│  ✓ Confirmed by lhoqvso
│
│  Checking dependencies...
│  ✓ Found: polkit systemd sqlite3
│
└─ ✅ Ready to upgrade (backup will be created)

┌─ Preparation Phase
│
│  Building binaries... ⣾
│  ✓ annad compiled (release) - 10.2s
│  ✓ annactl compiled (release)
│
│  Creating backup... ⣾
│  ✓ Backup: /var/lib/anna/backups/upgrade-20251030-140923/ - 0.6s
│
└─ ✅ 2/2 tasks complete (10.8s)

┌─ Installation Phase
│
│  Installing binaries...
│  ✓ annad → /usr/local/bin/
│  ✓ annactl → /usr/local/bin/
│
│  Configuring system...
│  ✓ Directories (5 created/verified)
│  ✓ Permissions (0750 root:anna)
│  ✓ Policies (3 loaded)
│  ✓ Service (enabled & started)
│
│  Writing version file...
│  ✓ /etc/anna/version → 0.9.4-beta
│
└─ ✅ 5/5 subsystems ready (2.1s)

┌─ Self-Healing Phase
│
│  Running doctor check... ⣾
│  ✓ Directories present
│  ✓ Ownership correct (root:anna)
│  ✓ Permissions correct
│  ✓ Dependencies installed
│  ✓ Service running
│  ✓ Socket accessible
│  ✓ Policies loaded (3 rules)
│  ✓ Events functional
│  ✓ Telemetry database exists
│
│  Running doctor repair... ⣾
│  ✓ All checks passed - 1.9s
│  ✓ No repairs needed
│
│  Verifying telemetry...
│  ✓ Database created
│  ✓ Collector initialized
│  ⏳ First sample in ~60s
│
└─ ✅ System healthy (4.2s)

╭────────────────────────────────────────────────╮
│                                                │
│  ✅ Installation Complete                      │
│                                                │
│           Anna is ready to serve!              │
│                                                │
│  Version:    0.9.4-beta                        │
│  Duration:   17.1s                             │
│  Mode:       LOW RISK AUTONOMY                 │
│  Status:     Fully Operational                 │
│                                                │
│  Next Steps:                                   │
│  • annactl status                              │
│  • annactl telemetry snapshot (after 60s)     │
│  • annactl doctor check                        │
│                                                │
╰────────────────────────────────────────────────╯

Install log: /var/log/anna/install.log
History: /var/log/anna/install_history.json
```

---

## Sprint 5 Phase 3 Complete Criteria

1. ✅ Installer feels ceremonial, not mechanical
2. ✅ Every line has purpose and clarity
3. ✅ Visual formatting adapts to terminal
4. ✅ Dependencies installed automatically
5. ✅ Doctor repair runs automatically
6. ✅ Install telemetry persisted to JSON
7. ✅ Final summary is a one-page masterpiece
8. ✅ Tests pass in TTY and non-TTY modes
9. ✅ Works on light and dark terminals
10. ✅ User feels confident and informed

---

## Next Session Prompt

```
Begin Sprint 5 Phase 3 – Beautiful & Intelligent Installer

Baseline: commit e0b6ca9 (Phase 2B complete: Telemetry RPC/CLI)

Reference: docs/SPRINT-5-PHASE-3-HANDOFF.md for complete implementation details.

Objectives:
1. Rewrite installer with 4-phase structure (detect, prepare, install, verify)
2. Add beautiful adaptive formatting (rounded boxes, tree borders, spinners)
3. Add install telemetry (install_history.json)
4. Auto-install dependencies (polkit, sqlite3)
5. Auto-run doctor repair and verification
6. Create ceremonial final summary

Target: v0.9.4-beta (installer UX complete)

Focus on visual beauty, conversational tone, and self-healing intelligence.
Make the installer feel like a guided ceremony, not a script dump.
```

---

**Generated with:** Claude Code
**Sprint:** 5 - Phase 3 Handoff
**Version:** 0.9.4-alpha → 0.9.4-beta
