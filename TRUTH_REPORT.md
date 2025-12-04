# Anna v0.0.83 Truth Report

Generated: 2025-12-04

## 1. Version State

| Location | Version | Status |
|----------|---------|--------|
| Cargo.toml | 0.0.83 | ✓ |
| CLAUDE.md | 0.0.83 | ✓ |
| README.md | 0.0.83 | ✓ |
| TODO.md | 0.0.83 | ✓ |
| RELEASE_NOTES.md | 0.0.83 | ✓ |
| SPEC.md | 0.0.83 | ✓ |
| Git tag | v0.0.83 | pending |
| GitHub Release | v0.0.83 | pending |

**Status: Versions updated, release pending.**

## 2. Public CLI Surface (from main.rs)

```
annactl                  REPL mode (interactive)
annactl <request...>     one-shot natural language request
annactl status           self-status
annactl reset            factory reset (requires root)
annactl uninstall        complete removal (requires root)
annactl --version        version (also: -V)
annactl --help           help (also: -h)
annactl --debug          enable debug mode (also: ANNA_DEBUG=1)
```

**Status: CLI surface is strict. No legacy sw/hw commands exposed.**

## 3. Install/Uninstall Paths

### Installer creates:
- `/usr/local/bin/annad`
- `/usr/local/bin/annactl`
- `/etc/anna/` (config)
- `/var/lib/anna/` (data)
- `/var/log/anna/` (logs)
- `/run/anna/` (runtime)
- `/etc/systemd/system/annad.service`
- `/var/lib/anna/internal/version.json`

### Uninstaller removes:
- `/usr/local/bin/annad` ✓
- `/usr/local/bin/annactl` ✓
- `/etc/anna/` ✓
- `/var/lib/anna/` ✓
- `/var/log/anna/` ✓
- `/run/anna/` ✓
- `/etc/systemd/system/annad.service` ✓
- `/usr/share/anna` **← DRIFT: Never created, but uninstaller tries to remove**
- `anna` user **← DRIFT: Installer no longer creates (v6.0.0), uninstaller still offers removal**

## 4. File Size Violations (>400 lines)

Source files exceeding 400-line limit:

| File | Lines | Severity |
|------|-------|----------|
| crates/annactl/src/pipeline.rs | 4,661 | CRITICAL |
| crates/annactl/src/commands/hw_detail.rs | 3,500 | CRITICAL |
| crates/anna_common/src/tool_executor.rs | 3,114 | CRITICAL |
| crates/anna_common/src/graphics_doctor.rs | 2,738 | CRITICAL |
| crates/annactl/src/commands/sw_detail.rs | 2,502 | CRITICAL |
| crates/anna_common/src/boot_doctor.rs | 2,477 | CRITICAL |
| crates/anna_common/src/telemetry_db.rs | 2,376 | CRITICAL |
| crates/anna_common/src/lib.rs | 2,286 | CRITICAL |
| crates/anna_common/src/storage_doctor.rs | 2,230 | CRITICAL |
| crates/anna_common/src/audio_doctor.rs | 2,078 | CRITICAL |
| crates/annactl/src/commands/status.rs | 1,917 | HIGH |
| crates/anna_common/src/policy.rs | 1,901 | HIGH |
| crates/anna_common/src/grounded/config.rs | 1,859 | HIGH |
| crates/anna_common/src/networking_doctor.rs | 1,826 | HIGH |
| crates/anna_common/src/anomaly_engine.rs | 1,816 | HIGH |
| crates/anna_common/src/doctor_registry.rs | 1,723 | HIGH |
| scripts/anna_deep_test.sh | 1,110 | MEDIUM |
| scripts/install.sh | 838 | MEDIUM |

**Total: 18 files over 400 lines**

## 5. CI Gates (from ci.yml)

Current gates:
- ✓ Build (debug + release)
- ✓ Unit tests
- ✓ Integration tests
- ✓ Smoke tests (--version, --help, status, natural language)
- ✓ Version consistency check (Cargo.toml vs CLAUDE.md vs README.md vs TODO.md)
- ✓ RELEASE_NOTES entry check
- ✓ Legacy command check (no sw/hw in docs)
- ✓ Security tests (redaction, policy, reliability)
- ⚠ Clippy (continue-on-error: true - advisory only)
- ⚠ Rustfmt (continue-on-error: true - advisory only)
- ⚠ Cargo audit (continue-on-error: true - advisory only)

Missing gates:
- ✗ File size limit (no check for >400 lines)
- ✗ Install/uninstall path consistency
- ✗ Checksum verification
- ✗ SPEC.md/CLAUDE.md consistency

## 6. Release Workflow

- Tag triggers release.yml
- Validates: Cargo.toml version matches tag
- Validates: README.md, CLAUDE.md, RELEASE_NOTES.md, TODO.md updated
- Builds in Arch Linux container
- Creates GitHub release with assets

**Status: Release workflow is functional but not fully automated (manual version bumps required).**

## 7. Drift Summary

### Critical Drift:
1. **Uninstall removes paths installer doesn't create**: `/usr/share/anna`, `anna` user
2. **18 files exceed 400-line limit** - violates project constraint

### Documentation Drift:
1. annactl/src/main.rs header says "v0.0.62" but code is v0.0.82
2. scripts/uninstall.sh header says "v0.0.1"

### Missing from CI:
1. No file size gate
2. No install/uninstall consistency test
3. Clippy/fmt are advisory (should be strict)

## 8. Logging State

### Current Implementation:
- Uses `tracing` crate for structured logging
- Request IDs generated via `generate_request_id()` function
- Request IDs flow through pipeline and are stored in case files

### Issues Found:
- `generate_request_id()` duplicated in 3 files (mutation_executor.rs, mutation_safety.rs, transcript.rs)
- Not all log entries include correlation IDs consistently

### Future Work:
- Consolidate `generate_request_id()` to single location
- Add tracing spans with request_id for consistent correlation
- Ensure all logs include correlation ID

## 9. Completed Actions (v0.0.82 Stabilization)

### Step 1: SPEC.md Created ✓
- Defined public CLI surface (strict)
- Defined file size limit (400 lines)
- Defined release discipline
- Defined install/uninstall invariants
- Defined no-invention rule

### Step 2: Drift Fixed ✓
- Updated uninstall.sh to handle legacy paths gracefully
- Legacy `/usr/share/anna` only removed if exists
- Legacy `anna` user only prompted if exists

### Step 3: CI Gates Added ✓
- File size check (advisory - grandfathered files exist)
- Install/uninstall path consistency check
- SPEC.md existence check
- Root cruft allowlist updated

### Step 4: File Size Remediation (FUTURE)
- 18 files over 400 lines need splitting
- Quarantine unused code, don't delete
- Not in scope for this stabilization pass
