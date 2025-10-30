# Sprint 5 Phase 3: Quick Start Guide

**Use this for the next Claude session**

---

## Session Start Prompt

```
Project: Anna Assistant
Current Version: v0.9.4-alpha
Target Version: v0.9.4-beta
Last Commit: e0b6ca9 (Sprint 5 Phase 2B – Telemetry RPC/CLI Integration)

Objective: Begin Sprint 5 Phase 3 – Beautiful & Intelligent Installer

Summary of baseline:
• Telemetry system complete (collection + SQLite + RPC + CLI)
• Doctor subsystem stable
• Autonomy levels operational (low/high)
• Version management functional (semantic compare, upgrade/skip)
• Installer currently functional but not aesthetically refined

Phase 3 Goals → transform the installer into a beautiful, intelligent,
self-healing experience.

Reference: docs/SPRINT-5-PHASE-3-HANDOFF.md for complete implementation details.
```

---

## Critical First Steps

### 1. Version Bump (MUST DO FIRST)

```bash
# Update these files to v0.9.4-beta:
- Cargo.toml (workspace.package.version)
- scripts/install.sh (BUNDLE_VERSION)
- tests/runtime_validation.sh (VERSION)

# This ensures upgrade path works correctly
```

### 2. Implementation Order

1. **Formatting Infrastructure** (2 hours)
   - Terminal detection
   - Box/border functions
   - Color palette
   - Symbol definitions

2. **Phase Rewrite** (3 hours)
   - detect_installation()
   - prepare_installation()
   - install_system()
   - verify_installation()

3. **Self-Healing** (1 hour)
   - Doctor integration
   - Dependency checking
   - Telemetry verification

4. **Install Telemetry** (1 hour)
   - JSON schema
   - record_install_telemetry()
   - Phase timing

5. **Final Summary** (30 min)
   - print_final_summary()
   - Next steps display

6. **Testing** (1 hour)
   - Fresh install test
   - Upgrade test
   - Non-TTY test

---

## Key Files to Modify

```
scripts/install.sh              Main installer (~400 lines added)
Cargo.toml                      Version bump to 0.9.4-beta
tests/runtime_validation.sh     Version bump + 5 new tests
CHANGELOG.md                    Add v0.9.4-beta entry
```

---

## Quick Reference: Visual Elements

### Box Borders
```bash
╭────────────────────────────────────────────────╮
│  Title centered here                           │
╰────────────────────────────────────────────────╯
```

### Phase Borders
```bash
┌─ Phase Name
│  Content indented
│  → Sub-items with arrows
│  ✓ Completed items
└─ ✅ Summary
```

### Symbols
```bash
✓  Success (or [OK] for ASCII)
✗  Failure (or [FAIL])
⚠  Warning (or [WARN])
→  Info (or [INFO])
⏳  Waiting (or [WAIT])
🤖 Robot (or [ANNA])
```

### Spinner Frames
```bash
⣾ ⣽ ⣻ ⢿ ⡿ ⣟ ⣯ ⣷  (rotating)
```

---

## install_history.json Schema

```json
{
  "installs": [
    {
      "timestamp": "2025-10-30T14:09:23Z",
      "mode": "upgrade",
      "old_version": "0.9.4-alpha",
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

---

## Acceptance Checklist

```
[ ] Version bumped to 0.9.4-beta in all files
[ ] Installer detects upgrade from 0.9.4-alpha
[ ] Visual formatting uses rounded boxes and tree borders
[ ] Colors adapt to terminal (light/dark detection)
[ ] Spinner works in TTY, dots in non-TTY
[ ] Dependencies auto-installed (polkit, sqlite3)
[ ] Doctor repair runs automatically
[ ] install_history.json created and populated
[ ] Final summary is beautiful and informative
[ ] All 5 validation tests pass
[ ] CHANGELOG.md updated
[ ] Commit created with full message
```

---

## Expected Output Preview

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
│  → Found v0.9.4-alpha
│  → Upgrade recommended
│
│  Upgrade now? [Y/n] y
│  ✓ Confirmed by lhoqvso
│
└─ ✅ Ready to upgrade

┌─ Preparation Phase
│
│  Building binaries... ⣾ (12.3s)
│  ✓ annad compiled (release)
│  ✓ annactl compiled (release)
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
│
└─ ✅ 4/4 subsystems ready

┌─ Self-Healing Phase
│
│  Running doctor repair... ⣾ (2.1s)
│  ✓ All checks passed
│
│  Verifying telemetry...
│  ✓ Database created
│
└─ ✅ System healthy

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
╰────────────────────────────────────────────────╯
```

---

## Troubleshooting

### Upgrade Not Detected
- Check `/etc/anna/version` contains "0.9.4-alpha"
- Verify version comparison logic in detect_installation()

### Visual Formatting Broken
- Test with: `TERM=dumb sudo ./scripts/install.sh`
- Check has_unicode_support() function
- Verify ASCII fallbacks work

### Doctor Repair Fails
- Run manually: `sudo annactl doctor check`
- Check logs: `/var/log/anna/doctor.log`
- Verify all 9 checks pass

### Telemetry JSON Not Created
- Check directory exists: `/var/log/anna/`
- Verify jq is installed
- Check permissions: `0640 root:anna`

---

## Commit Message Template

```
Sprint 5 Phase 3 - Beautiful Installer (v0.9.4-beta)

PHASE 3 COMPLETE: Ceremonial Installer UX with Self-Healing

=== IMPLEMENTATION SUMMARY ===

Transform installer from functional script to beautiful, intelligent,
self-healing ceremony with 4-phase structure and visual polish.

=== VISUAL ENHANCEMENTS ===

1. Rounded Box Borders (╭─╮ ╰─╯)
2. Tree-Style Phase Sections (┌─ └─)
3. Adaptive Color Palette (light/dark detection)
4. Unicode Symbols with ASCII Fallbacks
5. Animated Spinners (TTY) and Dots (non-TTY)

=== FOUR-PHASE STRUCTURE ===

Phase 1: Detection
- Version detection and upgrade prompts
- Dependency checking
- User confirmation

Phase 2: Preparation
- Binary compilation with timing
- Backup creation for upgrades
- Progress indicators

Phase 3: Installation
- Binary installation
- System configuration
- Service setup

Phase 4: Self-Healing & Verification
- Automatic doctor repair
- Telemetry verification
- System health check

=== INTELLIGENCE FEATURES ===

1. Auto-Dependency Installation
   - Detects missing: polkit, sqlite3
   - Auto-installs on Arch Linux
   - Graceful warnings on other distros

2. Self-Healing Integration
   - Runs doctor check automatically
   - Runs doctor repair if needed
   - Tracks repair count

3. Install Telemetry
   - Persistent JSON history
   - Phase durations recorded
   - Component status tracked

=== FINAL SUMMARY ===

Beautiful one-page summary showing:
- Version and duration
- Autonomy mode
- System status
- Clear next steps
- Log file locations

=== FILES MODIFIED ===

1. Cargo.toml - Version bump to 0.9.4-beta
2. scripts/install.sh (+400 lines) - Complete rewrite
3. tests/runtime_validation.sh (+100 lines) - 5 new tests
4. CHANGELOG.md (+80 lines) - v0.9.4-beta entry

=== VALIDATION ===

✅ Fresh install test
✅ Upgrade from 0.9.4-alpha test
✅ Non-TTY environment test
✅ Narrow terminal test
✅ Missing dependencies test

=== NEXT: SPRINT 5 PHASE 4 ===

Policy Action Execution:
- Define action types (log, alert, execute)
- Add executor to policy engine
- Trigger based on telemetry thresholds
- Track outcomes for learning

---

Sprint 5 Phase 3: Complete ✅
Anna now introduces herself beautifully!

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## Time Estimate

- **Implementation:** 6-8 hours
- **Testing:** 1-2 hours
- **Documentation:** 1 hour
- **Total:** 8-11 hours

---

## Success Metrics

1. **Visual Quality:** 10/10 (beautiful, balanced, clear)
2. **Clarity:** 10/10 (every line has purpose)
3. **Personality:** 10/10 (Anna's voice present)
4. **Intelligence:** 10/10 (self-healing, auto-deps)
5. **Telemetry:** 10/10 (complete JSON history)

---

**Ready to start Phase 3!** 🚀

Reference the full handoff document for detailed implementation:
`docs/SPRINT-5-PHASE-3-HANDOFF.md`
