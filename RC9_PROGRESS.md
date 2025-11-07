# RC.9 Progress Report

**Date:** 2025-11-07
**Status:** Work in Progress

---

## ✅ COMPLETED FIXES

### 1. Doctor Command - Complete Rewrite
**Status:** ✅ DONE (Committed: 33c7795)
**Lines Changed:** 464 lines (replaced old 392 lines)

**What Changed:**
- Removed ALL system health checks (pacman, disk, network, firewall, journal)
- Now focuses EXCLUSIVELY on Anna Assistant diagnostics
- Added CRITICAL feature: **Actual RPC connection test**, not just `systemctl`
- Can now detect when service reports "active" but socket is broken/unresponsive

**New Checks:**
1. **Binaries** - annad/annactl exist and executable
2. **Dependencies** - curl, jq, systemctl present
3. **Service** - systemd service state and configuration
4. **Socket Connectivity** - REAL RPC ping test (critical!)
5. **Directories** - /run/anna/ writable, config paths valid

**Output Example:**
```
╭────────────────────╮
│  Anna Health Check │
╰────────────────────╯

→ 🔧 Binaries
  ✓ annad binary exists and is executable
  ✓ annactl binary exists and is executable

→ 🔌 Daemon Service
  ✓ Service is loaded
  ✗ Service is not running

→ 🔗 Socket Connectivity
  ✗ Cannot connect to daemon
    Fix: sudo systemctl start annad
    Check logs: journalctl -u annad -n 50

→ 📊 Health Score: 60/100
```

---

### 2. Update Command - Release Notes Fix
**Status:** ✅ DONE (Committed: cd98b01)

**Problem:**
```
→ What's in This Version
                          ← Empty!
```

**Root Cause:**
- `fetch_release_notes()` called with "1.0.0-rc.8.3"
- GitHub tags are "v1.0.0-rc.8.3" (with "v")
- URL lookup failed silently

**Fix:**
- Auto-add "v" prefix when building GitHub API URL
- Now shows actual release notes

---

## 🚨 CRITICAL ISSUES DOCUMENTED

### Issue #0: Bundles Ignore Telemetry (NEW - CRITICAL!)
**File:** `1.0_CRITICAL_ISSUES.md` (updated)

**Problem:**
```
Bundle installing:
  1. Install nano text editor    ← USER ALREADY HAS VIM (40% CPU time)!
```

**Impact:**
- Defeats entire purpose of intelligent recommendations
- Bundles are static, not aware of user's actual usage
- Wastes time installing unwanted software

**Required Fix:**
- Bundles must query telemetry BEFORE generating packages
- Check existing software (vim? don't install nano)
- Check actual usage (CPU time, command history)
- Offer complementary packages, not replacements
- Allow customization before applying

**Implementation Needed:**
- Major refactor of bundle system
- Pass `SystemFacts` to bundle builders
- Add telemetry queries to bundle generation
- Smart filtering based on installed packages

---

## 🔴 REMAINING CRITICAL FIXES (Not Started)

### 3. Status Command Slowness
**Problem:** Takes ~10 seconds to show status
**Priority:** HIGH
**Status:** 🔴 NOT STARTED

**Fix Required:**
- Optimize RPC connection timeout
- Add fast-fail for unresponsive daemon
- Show immediate "checking..." feedback
- Cache status with short TTL

---

### 4. Category Duplicates & Capitalization
**Problem:**
```
• 10 suggestions about Security & Privacy
• 7 suggestions about Security & Privacy    ← DUPLICATE
• 5 suggestions about Development Tools
• 5 suggestions about Development Tools     ← DUPLICATE
• 5 suggestions about configuration        ← lowercase
• 3 suggestions about applications         ← lowercase
```

**Priority:** HIGH
**Status:** 🔴 NOT STARTED

**Fix Required:**
- Normalize ALL categories to Title Case
- Find and merge duplicate category entries
- Search `recommender.rs` for lowercase categories
- Possibly need to fix category assignment in Advice struct

---

### 5. Too Many Recommendations (142!)
**Problem:** 142 recommendations is overwhelming
**Priority:** HIGH
**Status:** 🔴 NOT STARTED

**Fix Required:**
- Better telemetry-based filtering
- Priority weighting based on user behavior
- Reduce to max 30-40 recommendations
- Focus on what user actually needs

---

## 🟡 MEDIUM PRIORITY (Not Started)

### 6. Report vs Advise Separation
**Status:** 🟡 NOT STARTED

**Current:** Both commands show similar info
**Required:**

**`annactl report`** - System Status:
- Plain English hardware detection
- Missing firmware/drivers
- Package analysis
- Performance metrics
- Usage telemetry ("You spend 40% CPU time on vim")

**`annactl advise`** - Improvements:
- Suggestions only
- Based on telemetry
- Prioritized by usage
- Alternative software
- Package improvements

---

### 7. Auto-Install Completions
**Status:** 🟡 NOT STARTED

**Current:** Completions are user option
**Required:** Auto-install during install/update if missing

---

### 8. Bundle Telemetry Integration
**Status:** 🟡 NOT STARTED (Major Refactor)

**Required:**
- Refactor bundle system architecture
- Pass SystemFacts + telemetry to bundle builders
- Smart package selection based on existing software
- Usage-based recommendations

---

## 📊 SUMMARY

**Completed:** 2/8 critical fixes (25%)
**Remaining:** 6 critical fixes + major refactor

**Next Steps (Priority Order):**
1. Fix category duplicates/capitalization (simpler fix)
2. Fix status slowness (performance)
3. Reduce recommendations to <50 (filtering)
4. Bundle telemetry integration (major refactor)
5. Report vs advise separation (new feature)
6. Auto-install completions (simple)

**Estimated Work:**
- Quick fixes (1-3): 2-4 hours
- Major refactors (4-5): 8-12 hours
- Total: ~10-16 hours of development

**For 1.0 Release:**
- MUST fix: #1-5 (critical bugs)
- SHOULD fix: #6-8 (UX improvements)
- Bundle telemetry is fundamental to Anna's value proposition

---

## 📝 DOCUMENTATION NEEDS

**User mentioned:** "Tones of documents are not updated in github, and some old ones need to be cleanup"

**TODO:**
- [ ] Update README.md with current RC status
- [ ] Update ROADMAP.md with RC.9 progress
- [ ] Clean up old Beta documentation
- [ ] Update TESTING.md checklist
- [ ] Document new doctor command behavior
- [ ] Add bundle customization guide
- [ ] Update troubleshooting docs

---

## 🎯 VISION FOR 1.0

**User Quote:** "this version 1.0 must be insanely good"

**Critical Success Factors:**
1. **Intelligence** - Bundles and advice based on actual usage
2. **Reliability** - Daemon stable, commands fast, no crashes
3. **Clarity** - Report vs advise clearly separated
4. **Trust** - Doctor detects real problems accurately
5. **Polish** - <50 focused recommendations, not 142 random ones

**The Big Picture:**
Anna must feel like it **understands your system**, not just throwing packages at you blindly. Telemetry-aware recommendations are THE killer feature.
