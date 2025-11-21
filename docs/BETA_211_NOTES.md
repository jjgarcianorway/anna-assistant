# Beta.211: CLI Welcome Engine Integration

**Version**: 5.7.0-beta.211
**Date**: 2025-01-21
**Type**: Feature Release (CLI Only)

---

## Summary

Beta.211 integrates the deterministic welcome engine (from Beta.209) into the `annactl status` command, using the output normalizer (from Beta.210) for proper CLI formatting.

**Scope:**
- ✅ CLI: `annactl status` now shows "System Welcome Report"
- ❌ TUI: Unchanged (deferred to Beta.212+)
- ❌ One-shot queries: Unchanged (prelude integration deferred to Beta.212+)

---

## What Beta.211 Does

### annactl status Enhancement

The `annactl status` command now includes a **System Welcome Report** section that:

1. **Tracks system changes** between invocations:
   - Kernel updates
   - Package installations/removals
   - Disk space changes
   - Hostname/hardware changes
   - RAM/CPU modifications

2. **Uses deterministic, telemetry-driven logic**:
   - Zero LLM calls
   - Session metadata stored in `/var/lib/anna/state/last_session.json`
   - Compares last session snapshot with current telemetry
   - Falls back gracefully if metadata unavailable

3. **Applies CLI formatting via normalizer**:
   - Uses `output::normalize_for_cli()` for colored output
   - Preserves [SUMMARY]/[DETAILS] structure
   - Section headers in cyan+bold
   - Commands in green
   - Clean, readable terminal output

### First Run vs. Returning User

**First Run** (no prior session data):
```
[SUMMARY]
Welcome to Anna Assistant! This is your first session.

[DETAILS]
System Information:
- Hostname: archlinux
- Kernel: 6.17.8-arch1-1
- CPU Cores: 8
- RAM: 16 GB
- Disk Space: 500 GB available
- Packages: 1234 installed
```

**Returning User** (with changes detected):
```
[SUMMARY]
Welcome back! 2 system changes detected since last run (3 hours ago).

[DETAILS]
Recent Changes:
- Kernel updated: 6.17.8-arch1-1 → 6.18.0-arch1-1
- Packages: +6 installed

Current System:
- Hostname: archlinux
- Kernel: 6.18.0-arch1-1
...
```

**Returning User** (no changes):
```
[SUMMARY]
Welcome back! No system changes detected since last run (2 hours ago).

[DETAILS]
System Status:
- Hostname: archlinux
- Kernel: 6.18.0-arch1-1
...
```

---

## What Beta.211 Does NOT Do

### TUI (Unchanged)

- TUI home screen unchanged
- TUI startup flow unchanged
- Welcome engine NOT wired into TUI
- **Explicitly deferred to Beta.212+**

### One-Shot Queries (Unchanged)

- `annactl "<question>"` behavior unchanged
- No welcome prelude before answers
- Uses existing answer formatting
- **Prelude integration explicitly deferred to Beta.212+**

### CLI Surface (Preserved)

- Still exactly 3 commands:
  - `annactl` (TUI)
  - `annactl status` (enhanced with welcome report)
  - `annactl "<question>"` (unchanged)
- No new flags
- No new subcommands

---

## Technical Details

### Architecture

```
annactl status
    ↓
status_command.rs::execute_anna_status_command()
    ↓
[Existing health checks: daemon, LLM, permissions, logs]
    ↓
[NEW] System Welcome Report section:
    ↓
telemetry::fetch_cached() → SystemTelemetry
    ↓
welcome::load_last_session() → Option<SessionMetadata>
    ↓
welcome::create_telemetry_snapshot() → TelemetrySnapshot
    ↓
welcome::generate_welcome_report() → String (ANSWER_FORMAT)
    ↓
output::normalize_for_cli() → String (colored)
    ↓
println!()
    ↓
welcome::save_session_metadata() → writes to /var/lib/anna/state/last_session.json
```

### Files Modified

- `crates/annactl/src/status_command.rs` (lines 175-212):
  - Added welcome report generation section
  - Integrated telemetry fetching
  - Wired normalizer for CLI output
- `crates/annactl/src/main.rs` (line 49):
  - Added `mod startup` declaration

### Session Metadata Format

Stored in `/var/lib/anna/state/last_session.json`:

```json
{
  "last_run": "2025-01-21T10:30:45Z",
  "last_telemetry": {
    "cpu_count": 8,
    "total_ram_mb": 16384,
    "hostname": "archlinux",
    "kernel_version": "6.17.8-arch1-1",
    "package_count": 1234,
    "available_disk_gb": 500
  },
  "anna_version": "5.7.0-beta.211"
}
```

---

## Limitations and Known Issues

1. **Session metadata requires daemon telemetry**:
   - If `telemetry::fetch_cached()` fails, shows error message
   - Gracefully falls back to "unable to fetch telemetry" message

2. **First-run detection**:
   - If `/var/lib/anna/state/last_session.json` doesn't exist, treats as first run
   - Normal behavior for new installations

3. **Time formatting**:
   - Human-readable time since last run (e.g., "3 hours ago", "2 days ago")
   - Accurate but not locale-aware

4. **Minimal change detection**:
   - Disk space changes only reported if > 1 GB
   - Package count changes always reported
   - Kernel/hostname/hardware changes always reported

---

## Build Status

- ✅ **Compilation**: SUCCESS
- ✅ **Module integration**: startup module properly declared
- ✅ **Output formatting**: normalize_for_cli() working correctly
- ✅ **Zero regressions**: Existing tests still pass

---

## Compatibility

- **Backwards Compatible**: ✅ (new feature, no breaking changes)
- **Forward Compatible**: ✅ (session metadata format stable)
- **TUI Unaffected**: ✅ (zero TUI changes)

---

## Deferred to Beta.212+

The following items from initial Beta.211 planning are **explicitly deferred**:

1. **TUI Welcome Integration**:
   - Replace TUI home page with deterministic welcome
   - Use `normalize_for_tui()` for TUI rendering
   - Status bar enhancements

2. **One-Shot Query Prelude**:
   - Brief system changes prelude before answers in `annactl "<question>"`
   - Conditional display (only if notable changes exist)

3. **Advanced Features**:
   - Historian trend integration into welcome (partially exists in Beta.209 code)
   - Multi-locale time formatting
   - High-resolution status bar updates

---

**Release Date**: 2025-01-21
**Philosophy**: Small, focused CLI enhancement. Keep TUI and one-shot behavior unchanged.
