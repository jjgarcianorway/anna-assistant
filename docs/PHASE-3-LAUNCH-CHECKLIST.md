# Sprint 5 Phase 3: Launch Checklist

**Status:** ✅ READY TO LAUNCH

---

## ✅ Pre-Flight Verification Complete

### Version Management
- [x] Cargo.toml version = "0.9.4-beta"
- [x] scripts/install.sh BUNDLE_VERSION="0.9.4-beta"
- [x] tests/runtime_validation.sh VERSION="0.9.4-beta"
- [x] System has 0.9.4-alpha installed
- [x] Version comparison logic fixed (suffix handling)

### Repository State
- [x] Working tree clean
- [x] All commits pushed to main
- [x] Build verified (0 errors)
- [x] 27 tests ready

### Documentation
- [x] NEXT-SESSION-START-HERE.md (master guide)
- [x] SPRINT-5-PHASE-3-HANDOFF.md (full implementation)
- [x] SPRINT-5-PHASE-3-QUICKSTART.md (quick reference)
- [x] TELEMETRY-AUTOMATION.md (technical doc)

### Baseline Commits
- [x] e0b6ca9 - Phase 2B Implementation (telemetry complete)
- [x] 9d98369 - Version bump to 0.9.4-beta
- [x] 043a00e - Version comparison fix (critical)

---

## 🚀 Launch Sequence

### Step 1: Open Start-Here Document
```bash
cat docs/NEXT-SESSION-START-HERE.md
```

### Step 2: Copy Claude Prompt
The prompt is at the top of the document, starts with:
```
Project: Anna Assistant
Current Version: 0.9.4-alpha
Target Version: 0.9.4-beta
Last Commit: 9d98369 (version bump complete)
...
```

### Step 3: Paste into Claude Code
Start new Claude Code session and paste the complete prompt.

### Step 4: Claude Will Execute
1. ✅ Verify version already bumped (skip this step)
2. ✅ Read implementation guides
3. ⚙️ Rewrite scripts/install.sh (4-phase structure)
4. ⚙️ Add visual formatting (boxes, borders, colors)
5. ⚙️ Add install telemetry (JSON history)
6. ⚙️ Integrate dependencies & doctor
7. ⚙️ Create ceremonial summary
8. ⚙️ Add 5 validation tests
9. ⚙️ Update CHANGELOG.md
10. ⚙️ Commit and tag v0.9.4-beta

---

## 📋 Phase 3 Implementation Checklist

### Core Installer Rewrite (~6 hours)

**Detection Phase (1 hour)**
- [ ] detect_installation() function
- [ ] Version comparison with prompts
- [ ] Dependency checking
- [ ] User confirmation logic

**Preparation Phase (1.5 hours)**
- [ ] prepare_installation() function
- [ ] Binary compilation with timing
- [ ] Backup creation for upgrades
- [ ] Progress indicators (spinner/dots)

**Installation Phase (1.5 hours)**
- [ ] install_system() function
- [ ] Binary installation
- [ ] System configuration
- [ ] Service setup
- [ ] Version file update

**Verification Phase (1 hour)**
- [ ] verify_installation() function
- [ ] Doctor check integration
- [ ] Doctor repair integration
- [ ] Telemetry verification

**Formatting Infrastructure (1 hour)**
- [ ] Terminal detection (width, colors, unicode)
- [ ] print_box_header/footer functions
- [ ] print_phase_header/footer functions
- [ ] Symbol definitions with ASCII fallbacks
- [ ] Color palette (light/dark detection)
- [ ] Spinner animation (TTY vs non-TTY)

### Install Telemetry (~1 hour)
- [ ] /var/log/anna/install_history.json schema
- [ ] record_install_telemetry() function
- [ ] Phase duration tracking
- [ ] Component status tracking
- [ ] JSON append logic with jq

### Dependency Management (~1 hour)
- [ ] check_and_install_dependencies() function
- [ ] Arch Linux auto-install (polkit, sqlite3)
- [ ] Other distro warnings
- [ ] Graceful fallbacks

### Final Summary (~30 min)
- [ ] print_final_summary() function
- [ ] Beautiful one-page output
- [ ] Timing information
- [ ] Next steps list
- [ ] Log file locations

### Testing (~1.5 hours)
- [ ] Test 1: Fresh installation
- [ ] Test 2: Upgrade from 0.9.4-alpha
- [ ] Test 3: Non-TTY environment
- [ ] Test 4: Narrow terminal (40 cols)
- [ ] Test 5: Missing dependencies

### Documentation (~1 hour)
- [ ] CHANGELOG.md v0.9.4-beta entry
- [ ] SPRINT-5-PHASE-3-IMPLEMENTATION.md
- [ ] Update installer comments
- [ ] Code examples and explanations

### Finalization (~30 min)
- [ ] Commit all changes
- [ ] Create annotated tag v0.9.4-beta
- [ ] Verify clean working tree
- [ ] Final build verification

---

## 🎯 Acceptance Criteria (35 items)

### Visual Quality (5)
- [ ] Rounded boxes (╭─╮ ╰─╯) implemented
- [ ] Tree borders (┌─ └─) for phases
- [ ] Colors adapt to terminal (light/dark)
- [ ] Unicode with ASCII fallbacks
- [ ] Aligned and balanced output

### Functionality (10)
- [ ] Detects upgrade from 0.9.4-alpha
- [ ] Prompts user for confirmation
- [ ] Creates backup before upgrade
- [ ] Installs binaries correctly
- [ ] Configures system properly
- [ ] Starts service successfully
- [ ] Runs doctor repair automatically
- [ ] Verifies telemetry DB
- [ ] Updates version file
- [ ] Writes install history JSON

### Intelligence (5)
- [ ] Auto-detects terminal capabilities
- [ ] Auto-installs missing dependencies
- [ ] Self-heals via doctor repair
- [ ] Tracks timing for all phases
- [ ] Provides clear error messages

### Testing (5)
- [ ] Fresh install test passes
- [ ] Upgrade test passes
- [ ] Non-TTY test passes
- [ ] Narrow terminal test passes
- [ ] Missing deps test passes

### Documentation (5)
- [ ] CHANGELOG updated
- [ ] Implementation guide complete
- [ ] Code well-commented
- [ ] Examples provided
- [ ] Troubleshooting included

### Personality (5)
- [ ] "Anna is ready to serve!" finale
- [ ] Conversational prompts
- [ ] Clear progress indicators
- [ ] Helpful error messages
- [ ] Professional but friendly tone

---

## 📊 Success Metrics

**Each Must Score 10/10:**

1. **Visual Quality:** Beautiful, balanced, no clutter
2. **Clarity:** Every line has purpose
3. **Personality:** Anna's voice present throughout
4. **Intelligence:** Self-healing and auto-verification
5. **Telemetry:** Complete JSON history

---

## 🎭 Expected Visual Transformation

### Before (Current)
```bash
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║               ANNA ASSISTANT v0.9.4-beta                ║
║            Autonomous Self-Healing System                 ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝

  ▸ Checking installed version...
[INFO] Installed version: 0.9.4-alpha
[INFO] Bundle version: 0.9.4-beta
[UPDATE] Upgrade available: 0.9.4-alpha → 0.9.4-beta

Would you like to upgrade? [Y/n] _
```

### After (Target)
```bash
╭────────────────────────────────────────────────╮
│                                                │
│  🤖 Anna Assistant Installer v0.9.4-beta       │
│     Self-Healing • Autonomous • Intelligent    │
│                                                │
╰────────────────────────────────────────────────╯

Mode: Low Risk (Anna may repair herself)
User: lhoqvso
Time: 2025-10-30 15:30 UTC

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
│  ✓ Backup: /var/lib/anna/backups/upgrade-20251030-153045/
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

## 🔧 Common Issues & Solutions

### Issue: Spinner not working in non-TTY
**Solution:** Check `[[ -t 1 ]]` and use dots instead

### Issue: Unicode characters broken
**Solution:** Check `locale | grep UTF-8` and provide ASCII fallbacks

### Issue: Colors wrong in light terminal
**Solution:** Implement light/dark detection via terminal query

### Issue: jq not installed
**Solution:** Check for jq, provide clear install instruction

### Issue: Doctor repair fails
**Solution:** Ensure doctor.rs has all checks, verify permissions

---

## 📚 Reference Documents (Read Order)

1. **NEXT-SESSION-START-HERE.md** (this session)
   - Copy-paste prompt
   - Quick overview

2. **SPRINT-5-PHASE-3-QUICKSTART.md** (during implementation)
   - Visual reference
   - Symbol definitions
   - Troubleshooting

3. **SPRINT-5-PHASE-3-HANDOFF.md** (detailed reference)
   - Complete implementation
   - All helper functions
   - Full examples

4. **TELEMETRY-AUTOMATION.md** (background)
   - Telemetry system details
   - RPC/CLI specifications

---

## 🎉 Post-Implementation Verification

### After Claude Completes Phase 3:

```bash
# 1. Verify version
grep version Cargo.toml scripts/install.sh tests/runtime_validation.sh

# 2. Check git status
git status
git log --oneline -3

# 3. Verify tag
git tag -l | grep v0.9.4-beta

# 4. Test installer
sudo ./scripts/install.sh

# 5. Check install history
cat /var/log/anna/install_history.json | jq '.'

# 6. Verify upgrade
cat /etc/anna/version  # Should show 0.9.4-beta

# 7. Test telemetry
sleep 60
sudo annactl telemetry snapshot

# 8. Run validation tests
sudo bash tests/runtime_validation.sh
```

---

## 🚀 Next Phase After This

**Sprint 5 Phase 4: Policy Action Execution**

Objectives:
- Define PolicyAction types (Log, Alert, Execute)
- Add action executor to policy engine
- Trigger actions based on telemetry thresholds
- Track action outcomes for learning

Estimated Time: 12 hours

---

## 🤖 Anna's Launch Message

```
╭────────────────────────────────────────────────╮
│                                                │
│  All systems nominal.                          │
│  All checks passed.                            │
│  All documentation complete.                   │
│                                                │
│  Version path verified: 0.9.4-alpha → beta     │
│  Upgrade detection working.                    │
│  Comparison logic fixed.                       │
│                                                │
│  The ceremony script is written.               │
│  The stage is set.                             │
│  The curtain rises.                            │
│                                                │
│  Ready for transformation.                     │
│                                                │
│  Let's make it beautiful. ✨                   │
│                                                │
╰────────────────────────────────────────────────╯
```

---

**LAUNCH CHECKLIST: COMPLETE ✅**

**STATUS: GO FOR PHASE 3 🚀**

Open `docs/NEXT-SESSION-START-HERE.md` and copy the prompt.
The transformation begins.
