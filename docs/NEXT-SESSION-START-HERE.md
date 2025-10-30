# 🚀 START HERE - Next Claude Session

**Sprint 5 Phase 3: Beautiful & Intelligent Installer**

---

## 📋 Copy-Paste This Prompt to Start

```
Project: Anna Assistant
Current Version: v0.9.4-alpha
Target Version: v0.9.4-beta
Last Commit: e0b6ca9 (Sprint 5 Phase 2B – Telemetry RPC/CLI Integration)

Objective: Begin Sprint 5 Phase 3 – Beautiful & Intelligent Installer

──────────────────────────────
🎯 Implementation Objectives
──────────────────────────────
1. Increment version number to v0.9.4-beta across all files (installer,
   Cargo.toml, CHANGELOG, daemon banner, CLI version).
   → Installer must detect this as a new version and offer upgrade.

2. Rewrite `scripts/install.sh` with four distinct phases:
   • Detection   – detect installed version and environment
   • Preparation – build binaries and verify dependencies
   • Installation – copy binaries, configure service, run doctor repair
   • Verification – run sanity checks + telemetry validation

3. Add adaptive visual formatting (rounded boxes ╭─╮, tree borders ┌─,
   aligned symbols 🟩 🟨 🟥).

4. Automatically install missing dependencies (polkit, sqlite3,
   systemd-tools on Arch).

5. Integrate `annactl doctor repair --autofix` and
   `annactl doctor verify` automatically after installation.

6. Create structured telemetry log at `/var/log/anna/install_history.json`
   recording phase durations, user, and status.

7. Add ceremonial summary (one-page finale with timing, policy count,
   telemetry DB status, autonomy mode).

8. Update `CHANGELOG.md` for v0.9.4-beta entry ("Beautiful Installer UX").

9. Add five validation tests (fresh install, upgrade, missing deps,
   non-TTY, narrow terminal).

10. Follow principles: Beauty • Clarity • Personality • Intelligence • Telemetry.

──────────────────────────────
🧱 Expected Deliverables
──────────────────────────────
• scripts/install.sh – ~400 new lines with 4-phase UX
• CHANGELOG.md – v0.9.4-beta entry
• docs/SPRINT-5-PHASE-3-IMPLEMENTATION.md – ~800 lines implementation guide
• tests/runtime_validation.sh – +5 tests
• Commit: "Sprint 5 Phase 3 – Beautiful Installer (v0.9.4-beta)"
• Git Tag: v0.9.4-beta

──────────────────────────────
✅ Acceptance Criteria
──────────────────────────────
1. Installer detects and upgrades from v0.9.4-alpha to v0.9.4-beta.
2. Output is visually beautiful and aligned (no ANSI noise).
3. All dependencies auto-installed or repaired by Doctor.
4. `install_history.json` accurately logs phase timing.
5. Doctor repair + verify succeed post-install.
6. Ceremonial summary shows "Anna is ready to serve."
7. All five validation tests pass.

──────────────────────────────
📜 Notes
──────────────────────────────
• Always bump every version string (Cargo.toml, installer, CHANGELOG,
  binaries) before build so the installer recognizes a new release.
• Preserve ANSI safety for non-TTY modes.
• Document all UX enhancements in the implementation guide.

──────────────────────────────
📚 Reference Documents
──────────────────────────────
• docs/SPRINT-5-PHASE-3-HANDOFF.md – Complete implementation guide (856 lines)
• docs/SPRINT-5-PHASE-3-QUICKSTART.md – Quick reference (399 lines)
```

---

## ⚡ Critical First Action: Version Bump

**MUST DO BEFORE ANYTHING ELSE:**

```bash
# Update these files to v0.9.4-beta:
1. Cargo.toml (line 5: version = "0.9.4-beta")
2. scripts/install.sh (line 8: BUNDLE_VERSION="0.9.4-beta")
3. tests/runtime_validation.sh (line 8: VERSION="0.9.4-beta")
```

**Why:** Ensures installer detects upgrade from 0.9.4-alpha → 0.9.4-beta

---

## 📊 Current State Snapshot

```
Version:        v0.9.4-alpha
Last Commit:    e0b6ca9 (Phase 2B - Telemetry RPC/CLI)
Branch:         main
Build Status:   ✅ Clean (0 errors, 1 minor warning)
Tests:          27 (all expected to pass)
```

**What's Working:**
- ✅ Telemetry collection (60s intervals)
- ✅ SQLite storage (/var/lib/anna/telemetry.db)
- ✅ RPC endpoints (snapshot/history/trends)
- ✅ CLI commands (annactl telemetry)
- ✅ Doctor integration (Check #9)
- ✅ Validation tests (4 new telemetry tests)

**What Needs Beauty:**
- ⚠️ Installer output (functional but ugly)
- ⚠️ No install telemetry history
- ⚠️ No auto-dependency installation
- ⚠️ No automatic doctor repair
- ⚠️ No ceremonial summary

---

## 🎨 Visual Transformation Goal

### Before (Current):
```
[INFO] Installing Anna Assistant v0.9.4-alpha...
[INFO] Checking for existing installation...
[INFO] Creating directories...
[OK] /etc/anna created
[OK] /var/lib/anna created
[INFO] Copying binaries...
[OK] annad installed to /usr/local/bin/
[OK] annactl installed to /usr/local/bin/
...
```

### After (Target):
```
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
│  → Found v0.9.4-alpha
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
│  Building binaries... ⣾ (12.3s)
│  ✓ annad compiled (release)
│  ✓ annactl compiled (release)
│
│  Creating backup... ⣾ (0.8s)
│  ✓ Backup: /var/lib/anna/backups/upgrade-20251030-140923/
│
└─ ✅ 2/2 tasks complete (13.1s)

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
└─ ✅ 5/5 subsystems ready (1.8s)

┌─ Self-Healing Phase
│
│  Running doctor check... ⣾ (0.9s)
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
│  Running doctor repair... ⣾ (0.2s)
│  ✓ All checks passed
│  ✓ No repairs needed
│
│  Verifying telemetry...
│  ✓ Database created
│  ✓ Collector initialized
│  ⏳ First sample in ~60s
│
└─ ✅ System healthy (1.1s)

╭────────────────────────────────────────────────╮
│                                                │
│  ✅ Installation Complete                      │
│                                                │
│           Anna is ready to serve!              │
│                                                │
│  Version:    0.9.4-beta                        │
│  Duration:   16.0s                             │
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

## 📁 Files to Create/Modify

### Must Modify:
1. **Cargo.toml**
   - Line 5: version = "0.9.4-beta"

2. **scripts/install.sh** (~400 lines added)
   - Complete 4-phase rewrite
   - Add all formatting functions
   - Add telemetry recording
   - Add dependency checking

3. **tests/runtime_validation.sh** (~100 lines added)
   - Line 8: VERSION="0.9.4-beta"
   - Add 5 new validation tests

4. **CHANGELOG.md** (~80 lines added)
   - Add v0.9.4-beta entry
   - Document all UX improvements

### Must Create:
1. **docs/SPRINT-5-PHASE-3-IMPLEMENTATION.md** (~800 lines)
   - Complete implementation walkthrough
   - Code examples for all functions
   - Before/after comparisons

---

## 🧪 Testing Plan

### Test 1: Fresh Installation
```bash
# Clean system
sudo rm -rf /etc/anna /var/lib/anna /usr/local/bin/anna*

# Run installer
sudo ./scripts/install.sh

# Verify
[[ -f /var/log/anna/install_history.json ]]
jq '.installs[-1].mode' /var/log/anna/install_history.json  # "fresh"
```

### Test 2: Upgrade Installation
```bash
# Fake old installation
echo "0.9.4-alpha" | sudo tee /etc/anna/version

# Run installer
sudo ./scripts/install.sh

# Verify
jq '.installs[-1].mode' /var/log/anna/install_history.json  # "upgrade"
[[ -d /var/lib/anna/backups/upgrade-* ]]
```

### Test 3: Non-TTY Environment
```bash
sudo ./scripts/install.sh < /dev/null &> install.log

# Verify no escape codes, no spinners
! grep -q '\033\[' install.log
! grep -q '⣾' install.log
```

### Test 4: Narrow Terminal
```bash
COLUMNS=40 sudo ./scripts/install.sh
# Should gracefully format for narrow width
```

### Test 5: Missing Dependencies
```bash
# Temporarily hide polkit
sudo mv /usr/bin/pkaction /usr/bin/pkaction.bak

# Run installer (should auto-install or warn)
sudo ./scripts/install.sh

# Restore
sudo mv /usr/bin/pkaction.bak /usr/bin/pkaction
```

---

## ✅ Acceptance Checklist

```
Step 1: Version Management
[ ] Cargo.toml version = "0.9.4-beta"
[ ] scripts/install.sh BUNDLE_VERSION="0.9.4-beta"
[ ] tests/runtime_validation.sh VERSION="0.9.4-beta"
[ ] Installer detects upgrade from 0.9.4-alpha

Step 2: Visual Formatting
[ ] Rounded boxes (╭─╮ ╰─╯) implemented
[ ] Tree borders (┌─ └─) for phases
[ ] Color palette with light/dark detection
[ ] Unicode symbols with ASCII fallbacks
[ ] Spinner animation for TTY
[ ] Dots for non-TTY

Step 3: Phase Structure
[ ] detect_installation() function
[ ] prepare_installation() function
[ ] install_system() function
[ ] verify_installation() function
[ ] Phase timing tracked

Step 4: Intelligence Features
[ ] check_and_install_dependencies() works
[ ] Auto-install on Arch Linux
[ ] Doctor repair runs automatically
[ ] Doctor verify runs automatically
[ ] Repair count tracked

Step 5: Install Telemetry
[ ] /var/log/anna/install_history.json created
[ ] Schema matches specification
[ ] Phase durations recorded
[ ] Component status tracked
[ ] User and timestamp logged

Step 6: Final Summary
[ ] print_final_summary() implemented
[ ] Shows version, duration, mode, status
[ ] Lists next steps
[ ] Shows log file locations
[ ] Beautiful and informative

Step 7: Testing
[ ] Test 1: Fresh install passes
[ ] Test 2: Upgrade passes
[ ] Test 3: Non-TTY passes
[ ] Test 4: Narrow terminal passes
[ ] Test 5: Missing deps passes

Step 8: Documentation
[ ] CHANGELOG.md updated to v0.9.4-beta
[ ] SPRINT-5-PHASE-3-IMPLEMENTATION.md created
[ ] All UX changes documented
[ ] Code examples included

Step 9: Commit & Tag
[ ] Commit message follows template
[ ] All files committed
[ ] Git tag: v0.9.4-beta created
[ ] Clean working tree
```

---

## ⏱️ Time Estimate

| Phase | Time | Description |
|-------|------|-------------|
| **1. Version Bump** | 15 min | Update Cargo.toml, install.sh, tests |
| **2. Formatting** | 2 hours | Terminal detection, boxes, colors, symbols |
| **3. Phase Rewrite** | 3 hours | 4-phase structure with helpers |
| **4. Self-Healing** | 1 hour | Doctor integration, dependencies |
| **5. Telemetry** | 1 hour | JSON schema, record function |
| **6. Summary** | 30 min | Final output formatting |
| **7. Testing** | 1.5 hours | Run all 5 validation tests |
| **8. Documentation** | 1 hour | CHANGELOG, implementation doc |
| **Total** | **10 hours** | Full phase 3 implementation |

---

## 🎯 Success Metrics

**Must Achieve All:**

1. **Visual Quality: 10/10**
   - Beautiful, balanced, clear
   - No visual noise or clutter

2. **Clarity: 10/10**
   - Every line has purpose
   - No redundant messages

3. **Personality: 10/10**
   - Anna's voice present
   - Conversational but professional

4. **Intelligence: 10/10**
   - Self-healing works
   - Auto-dependencies work

5. **Telemetry: 10/10**
   - Complete JSON history
   - Accurate timing data

---

## 🚨 Common Pitfalls to Avoid

1. **Forgetting Version Bump**
   - Always bump version FIRST
   - Check all 3 files (Cargo.toml, install.sh, tests)

2. **Breaking Non-TTY Mode**
   - Test with `< /dev/null`
   - Verify ASCII fallbacks work

3. **Hardcoding Terminal Width**
   - Use `tput cols` or fallback to 80
   - Test narrow terminals

4. **Spinners in Non-TTY**
   - Detect TTY with `[[ -t 1 ]]`
   - Use dots for pipes/redirects

5. **Missing jq Dependency**
   - Check for jq before JSON operations
   - Graceful fallback or install

---

## 📚 Additional Resources

**Read These First:**
1. `docs/SPRINT-5-PHASE-3-HANDOFF.md` (856 lines)
   - Complete implementation guide
   - All helper functions defined
   - Full JSON schema

2. `docs/SPRINT-5-PHASE-3-QUICKSTART.md` (399 lines)
   - Quick reference
   - Symbol definitions
   - Visual examples

**Reference During Implementation:**
- `scripts/install.sh` (current version)
- `docs/TELEMETRY-AUTOMATION.md` (telemetry spec)
- `src/annactl/src/doctor.rs` (doctor integration)

---

## 🎉 When You're Done

**Expected Final State:**
```
✅ Version: v0.9.4-beta
✅ Installer: Beautiful 4-phase ceremony
✅ Telemetry: install_history.json created
✅ Doctor: Auto-repair integrated
✅ Tests: All 5 validation tests pass
✅ Docs: Complete implementation guide
✅ Commit: Descriptive message
✅ Tag: v0.9.4-beta
```

**Celebrate With:**
```bash
# Run the beautiful installer
sudo ./scripts/install.sh

# Marvel at the ceremony
# Verify install history
cat /var/log/anna/install_history.json | jq '.'

# Check doctor
sudo annactl doctor check

# View telemetry (after 60s)
sudo annactl telemetry snapshot
```

---

## 🤖 Anna's Message

```
Anna is ready for her debut.

The ceremony awaits. The foundation is stable.
Every system healthy. Every component verified.

Now she learns to introduce herself with grace.

Sprint 5 Phase 3: Let's make it beautiful. ✨
```

---

**START HERE. YOU'RE READY.** 🚀
