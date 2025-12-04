# Anna Specification v0.0.83

This document is the **authoritative source of truth** for Anna's invariants.
It supersedes any conflicting information in other documentation.

---

## 1. Public CLI Surface (Strict)

The ONLY public commands are:

```
annactl                     # REPL mode (interactive)
annactl <request...>        # One-shot natural language request
annactl status              # Self-status display
annactl reset [--dry-run] [--force]     # Factory reset (root required)
annactl uninstall [--dry-run] [--force] # Complete removal (root required)
annactl --version | -V      # Show version
annactl --help | -h         # Show help
annactl --debug [command]   # Enable debug mode
```

**No other public commands.** All capabilities are accessed via natural language.

### Adding New Commands

New public commands require:
1. Explicit user approval
2. Update to this SPEC.md
3. Update to CLAUDE.md
4. CI gate verification

---

## 2. File Size Limit

**Maximum file size: 400 lines**

This applies to:
- All `.rs` files in `crates/`
- All `.sh` files in `scripts/`

Exceptions (grandfathered, must not grow):
- Files listed in TRUTH_REPORT.md as existing violations

Remediation:
- Split into modules
- Quarantine unused code to `_quarantine/` directory (do not delete)

---

## 3. Version Discipline

### Versioning Rule
Anna uses `0.xxx.yyy` format until production quality.
Every meaningful change = version bump.

### Version Locations (must all match)
1. `Cargo.toml` - `version = "X.Y.Z"`
2. `CLAUDE.md` - `**Version: X.Y.Z**`
3. `README.md` - `Anna Assistant vX.Y.Z`
4. `TODO.md` - `**Current Version: X.Y.Z**`
5. `RELEASE_NOTES.md` - `## vX.Y.Z` section
6. Git tag - `vX.Y.Z`

### Release Checklist
1. Update version in all 5 files
2. Add RELEASE_NOTES.md entry
3. Commit
4. Create annotated tag: `git tag -a vX.Y.Z -m "vX.Y.Z: description"`
5. Push commit and tag: `git push origin main --tags`
6. Verify GitHub release workflow completes
7. Verify release has assets (annad, annactl, SHA256SUMS)

---

## 4. Install/Uninstall Invariants

### Paths Created by Installer

| Path | Purpose | Owner |
|------|---------|-------|
| `/usr/local/bin/annad` | Daemon binary | root:root |
| `/usr/local/bin/annactl` | CLI binary | root:root |
| `/etc/anna/` | Configuration | root:root |
| `/etc/anna/config.toml` | Main config | root:root |
| `/var/lib/anna/` | Data directory | root:root |
| `/var/lib/anna/internal/` | Internal state | root:root |
| `/var/lib/anna/internal/version.json` | Version stamp | root:root |
| `/var/log/anna/` | Log directory | root:root |
| `/run/anna/` | Runtime directory | root:root |
| `/etc/systemd/system/annad.service` | Systemd unit | root:root |

### Uninstall Must Remove

**Every path the installer creates must be removed by uninstaller.**

No extra paths. No leftover files. No orphaned users/groups.

### Invariant Test

```bash
# Before install: snapshot filesystem
# Run install
# Run uninstall --force
# After uninstall: compare filesystem
# Difference must be empty (except logs if user chose to keep)
```

---

## 5. CI Gates (Mandatory)

### Blocking Gates (must pass to merge)

| Gate | Description |
|------|-------------|
| build | Compiles in Arch Linux (debug + release) |
| test | All unit and integration tests pass |
| smoke | CLI smoke tests (--version, --help, status, natural language) |
| hygiene | Version consistency across all files |
| version-match | Cargo.toml = CLAUDE.md = README.md = TODO.md |
| release-notes | RELEASE_NOTES.md has entry for current version |
| file-size | No source file exceeds 400 lines (new files only) |
| install-paths | Install/uninstall path consistency |

### Advisory Gates (warnings only)

| Gate | Description |
|------|-------------|
| clippy | Rust lints |
| fmt | Code formatting |
| audit | Security vulnerabilities |

---

## 6. No Invention Rule

When uncertain about implementation:

1. **Inspect first** - Read existing code before writing new code
2. **Ask if unclear** - Don't guess at requirements
3. **Preserve working code** - Don't delete without quarantining
4. **No feature creep** - Only implement what's explicitly requested

---

## 7. Debug Mode

Debug mode is enabled by:
1. `annactl --debug <command>`
2. `ANNA_DEBUG=1` environment variable
3. `ANNA_DEBUG_TRANSCRIPT=1` environment variable
4. Config: `[ui] transcript_mode = "debug"`

Debug mode shows:
- Tool names and evidence IDs
- Timing information
- Internal dialogue steps
- Parse warnings and retries

Human mode (default) shows:
- Professional IT dialogue
- Human-readable descriptions
- No internal identifiers

---

## 8. Structured Logging

### Log Levels
- `error` - Failures requiring attention
- `warn` - Degraded operation
- `info` - Normal operation events
- `debug` - Detailed debugging
- `trace` - Verbose tracing

### Log Format
```
[timestamp] [level] [component] [correlation_id] message
```

### Correlation ID
Every request gets a unique correlation ID that flows through:
- annactl → annad (via RPC)
- All internal operations
- All log entries

---

## 9. Change Control

This SPEC.md can only be modified:
1. With explicit user approval
2. With corresponding version bump
3. With CI verification that changes are reflected in code

---

## Appendix: File Headers

All source files should have a header comment indicating:
- Purpose
- Current version (matching SPEC)
- Key functionality

Example:
```rust
//! Module Name v0.0.82
//!
//! Purpose: Brief description
```
