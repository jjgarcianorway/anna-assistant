# Phase 4.3 Implementation Status

**Version:** v0.9.6-alpha (in progress)
**Date:** 2025-10-30
**Status:** 🟡 Foundation Complete, Full Implementation Pending

---

## 🎯 What We Set Out To Build

Phase 4.3 aims to give Anna deep system understanding and smart, explainable actions through:

1. **Deep System Profiling** - Hardware, graphics, audio, network, boot, software inventory
2. **Smart Playbooks** - Safe, reversible system improvements with dry-run
3. **Natural "Ask" Interface** - Route user intents to appropriate actions
4. **Privacy & Consent** - Explicit gates, transparent collection, easy opt-out
5. **Config Governance** - Single source of truth via `annactl config`

---

## ✅ What's Been Completed

### 1. Version Bump (v0.9.6-alpha)
- ✅ Updated `Cargo.toml` workspace version
- ✅ Updated `scripts/install.sh` version and header
- ✅ Created `news/v0.9.6-alpha.txt` with user-facing highlights

### 2. Foundation Infrastructure
- ✅ Created profile subsystem structure (`src/annactl/src/profile/`)
- ✅ Created playbooks structure (`src/annactl/src/playbooks/`)
- ✅ Built `ProfileCollector` with categorized data gathering:
  - Hardware (CPU, memory, kernel)
  - Graphics (GPU detection, VA-API, session type)
  - Audio (PipeWire/PulseAudio detection)
  - Network (interface listing)
  - Boot (systemd-analyze, failed units)
  - Software (package managers, AUR helpers, shells, tools)
  - Usage (consent-gated placeholder)

### 3. Existing Anna Infrastructure (from Phases 4.0-4.1)
- ✅ Unified messaging layer (`anna_common`)
- ✅ Conversational installer with ceremonies
- ✅ Friendly privilege escalation helpers
- ✅ Config/persona system foundations
- ✅ News and explore commands
- ✅ Doctor system (self-healing)

---

## 🔨 What Remains To Be Built

### Critical Path Components

#### 1. Profile Checks System (`src/annactl/src/profile/checks.rs`)
```rust
// Curated health checks with human explanations
pub struct Check {
    id: String,
    status: CheckStatus,  // OK, WARN, FAIL, INFO
    message: String,
    remediation: Option<String>,
}

// Examples needed:
// - VA-API available but not enabled in browser
// - Boot time > 30s with specific slow units identified
// - GPU driver mismatch
// - Missing codecs for common video formats
```

#### 2. Profile Rendering (`src/annactl/src/profile/render.rs`)
- Pretty, sectioned output using `anna_box` and `anna_say`
- Clear categorization (Hardware / Graphics / Audio / etc.)
- Highlight warnings/failures with remediation hints
- Support `--json` for machine-readable output

#### 3. Smart Playbooks (Minimum Set)

Each needs: `detect()`, `plan()`, `execute()`, `verify()`, `rollback()`

**Priority Order:**
1. **browser.hwaccel** - Enable VA-API/VDPAU in Firefox/Chromium
2. **boot.speedup** - Analyze and mask slow systemd units
3. **video.codecs** - Install common playback codecs
4. **hyprland.beautiful** - Themed Hyprland setup (if present)
5. **power.governor** - Optimize CPU governor
6. **shell.aliases.suggest** - Propose helpful aliases

#### 4. annactl Commands Integration

```rust
// In src/annactl/src/main.rs, add:
Commands::Profile { action } => match action {
    ProfileAction::Show { json } => profile_cmd::show(json).await,
    ProfileAction::Checks { status, category } =>
        profile_cmd::checks(status, category).await,
},

Commands::Ask { intent } => ask_cmd::route(&intent).await,

Commands::Playbooks { action } => match action {
    PlaybookAction::List => playbooks_cmd::list().await,
    PlaybookAction::Plan { name, opts } =>
        playbooks_cmd::plan(name, opts).await,
    PlaybookAction::Run { name, opts, yes } =>
        playbooks_cmd::run(name, opts, yes).await,
},
```

#### 5. SQLite Schema

Extend `telemetry.db`:

```sql
CREATE TABLE IF NOT EXISTS profile_runs (
  id INTEGER PRIMARY KEY,
  ts_utc TEXT NOT NULL,
  depth TEXT NOT NULL,
  duration_ms INTEGER,
  success INTEGER
);

CREATE TABLE IF NOT EXISTS profile_signals (
  run_id INTEGER,
  category TEXT,
  key TEXT,
  value TEXT,
  FOREIGN KEY(run_id) REFERENCES profile_runs(id)
);

CREATE TABLE IF NOT EXISTS profile_checks (
  run_id INTEGER,
  check_id TEXT,
  status TEXT,
  message TEXT,
  remediation TEXT
);

CREATE TABLE IF NOT EXISTS playbook_runs (
  id INTEGER PRIMARY KEY,
  ts_utc TEXT,
  name TEXT,
  mode TEXT,  -- dry-run|execute|verify|rollback
  result TEXT -- ok|changed|failed|rolled_back
);
```

#### 6. Config Extension

Add to `~/.config/anna/prefs.yml`:

```yaml
# ─────────────────────────────────────────────────────────
# Managed by Anna. Please use `annactl config ...` to change behavior.
# Manual edits may be overwritten. See `annactl help config`.
# ─────────────────────────────────────────────────────────

privacy:
  profiling_depth: standard  # minimal|standard|deep
  usage_metrics: off         # explicit consent required

playbooks:
  autoverify: true
  autorollback: true
  backups_dir: ~/.local/share/anna/backups
```

#### 7. Friendly Escalation Wrapper

Bash helper in `scripts/anna_common.sh`:

```bash
anna_escalate() {
    local reason="$1"
    shift
    local command=("$@")

    anna_narrative "I need administrator privileges to ${reason}."
    anna_info "I'll request temporary permission and return here as soon as it's done."

    if sudo "${command[@]}"; then
        anna_ok "Done!"
        return 0
    else
        anna_error "That didn't work out. You might need to check permissions."
        return 1
    fi
}
```

#### 8. Installer Updates

Add to final summary:

```bash
anna_box narrative \
    "Your Knobs - How to Customize Me" \
    "" \
    "Change any setting:    annactl config set <key> <value>" \
    "See all settings:      annactl config list" \
    "Profile your system:   annactl profile show" \
    "Ask for help:          annactl ask 'make X better'" \
    "" \
    "Remember: config files say 'use annactl' for a reason!"
```

#### 9. Runtime Tests

In `tests/runtime_validation.sh`, add:

```bash
test_profile_show_prints_sections() {
    # Verify profile output contains expected sections
}

test_profile_checks_json() {
    # Verify --json produces valid JSON
}

test_playbook_dry_run() {
    # Ensure dry-run makes no changes
}

test_config_governance_banner() {
    # Config file has banner and round-trips
}

test_privacy_gates() {
    # Usage collection respects consent
}
```

#### 10. Documentation

- `docs/PHASE-4.3-PROFILING-AND-PLAYBOOKS.md` (full spec)
- Update `CHANGELOG.md` with v0.9.6-alpha entry
- Update `docs/PHASE-4-SUMMARY.md` with 4.3 highlights

---

## 📊 Progress Estimate

| Component | Status | Effort Remaining |
|-----------|--------|------------------|
| Version bump | ✅ Complete | 0% |
| Profile collector | ✅ Complete | 0% |
| Profile checks | 🟡 Planned | 100% |
| Profile render | 🟡 Planned | 100% |
| Playbooks (6 total) | 🟡 Planned | 100% |
| CLI integration | 🟡 Planned | 100% |
| SQLite schema | 🟡 Planned | 100% |
| Config extension | 🟡 Planned | 100% |
| Escalation wrapper | 🟡 Planned | 100% |
| Installer updates | 🟡 Planned | 100% |
| Tests | 🟡 Planned | 100% |
| Documentation | 🟡 Planned | 100% |

**Overall:** ~8% complete (foundation only)

---

## 🚀 Next Steps (Priority Order)

1. **Complete Profile Rendering** - Make `annactl profile show` work
2. **Build Checks System** - At least 10 meaningful checks
3. **Implement One Playbook** - `browser.hwaccel` as proof of concept
4. **CLI Integration** - Wire up profile/playbooks commands
5. **Basic Tests** - Prove the foundation works
6. **Documentation** - Full specification document

---

## 🎯 Acceptance Criteria (Not Yet Met)

- [ ] `annactl profile show` produces friendly, sectioned output
- [ ] `annactl profile checks` lists health findings with remediation
- [ ] `annactl ask browser.hwaccel --dry-run` shows a clear plan
- [ ] At least 3 playbooks functional (browser.hwaccel, boot.speedup, video.codecs)
- [ ] Config governance active with banner enforcement
- [ ] Privacy toggles respected
- [ ] Installer shows "Your knobs" panel
- [ ] Tests pass
- [ ] Documentation complete

---

## 💡 Recommendations

Given the scope and current token constraints, we have two paths:

### Option A: Complete Core Features (Recommended)
Focus on making a minimal but *working* Phase 4.3:
- Working profile show/checks
- One fully functional playbook (browser.hwaccel)
- Basic tests
- Essential docs
- Tag as v0.9.6-alpha-partial or v0.9.6-preview

### Option B: Full Implementation (Requires Fresh Session)
- Complete all 6 playbooks
- Full test suite
- Comprehensive docs
- All integration points
- Tag as v0.9.6-alpha (full)

**Estimated additional tokens for Option A:** ~30-40k
**Estimated additional tokens for Option B:** ~70-90k

---

## 📝 Notes

- Foundation is solid and follows Anna's conversational patterns
- Profile collector successfully gathers system data
- Integration points are clear
- Next implementer can pick up from `ProfileCollector` structure
- All existing Phase 4.0-4.1 infrastructure is preserved

---

**Status:** Ready for continued implementation
**Next Action:** Choose Option A or B and continue
**Token Budget Remaining:** ~81k tokens

