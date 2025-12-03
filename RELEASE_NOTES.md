# Anna Assistant - Release Notes

---

## v0.0.2 - Strict CLI Surface

**Release Date:** 2024-12-03

### Summary

Enforces the strict CLI surface. All legacy commands (sw, hw, JSON flags) are removed from public dispatch and now route through natural language processing.

### Supported Entrypoints

```bash
annactl                  # REPL mode (interactive)
annactl <request>        # One-shot natural language request
annactl status           # Self-status
annactl --version        # Version (also: -V)
annactl --help           # Help (also: -h)
```

**That's the entire public surface.**

### Changes

**CLI Surface:**
- Removed `sw` command from public surface
- Removed `hw` command from public surface
- Removed all JSON flags (--json, --full) from public surface
- Legacy commands now route as natural language requests (no custom error message)
- Added --help/-h flags for explicit help display

**REPL Mode:**
- Implemented basic REPL loop
- Exit commands: exit, quit, bye, q
- Help command shows REPL-specific help
- Status command works in REPL

**Dialogue Format:**
- Natural language requests show `[you] to [anna]:` format
- Responses show `[anna] to [you]:` format
- Reliability score displayed (stub: 0% until LLM integration)

**Tests:**
- Added test for --help showing strict surface only
- Added test for status command exit 0
- Added test for --version format
- Added test for legacy command routing (sw, hw)
- Added test for natural language request format

### Breaking Changes

- `annactl sw` no longer shows software overview (routes as request)
- `annactl hw` no longer shows hardware overview (routes as request)
- `annactl` (no args) now enters REPL instead of showing help
- Use `annactl --help` or `annactl -h` for help

### Internal

- Snapshot architecture preserved (internal capabilities only)
- Status command unchanged
- Version output format unchanged: `annactl vX.Y.Z`

---

## v0.0.1 - Specification Lock-In

**Release Date:** 2024-12-03

### Summary

Complete specification reset. Anna transitions from a "snapshot reader with fixed commands" to a "natural language virtual sysadmin" architecture.

### Changes

**Governance:**
- Established immutable operating contract (CLAUDE.md)
- Created implementation roadmap (TODO.md)
- Set up release notes workflow
- Version reset to 0.0.1 (staying in 0.x.x until production)

**Documentation:**
- README.md rewritten for natural language assistant vision
- CLAUDE.md created with full engineering contract
- TODO.md created with phased implementation roadmap
- RELEASE_NOTES.md created for change tracking

**Architecture Decision:**
- Preserve existing snapshot-based telemetry foundation
- Build natural language layer on top
- Strict CLI surface: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- All old commands (sw, hw, JSON flags) become internal capabilities only

**Spec Highlights:**
- 4-player model: User, Anna, Translator, Junior, Senior
- Debug mode always on (visible dialogue)
- Reliability scores on all answers (0-100%)
- Safety classification: read-only, low-risk, medium-risk, high-risk
- Rollback mandate for all mutations
- Recipe learning system
- XP and gamification (levels 0-100, nerdy titles)
- Auto-update every 10 minutes
- Auto Ollama setup

### Breaking Changes

- Version number reset from 7.42.5 to 0.0.1
- Old CLI commands will be removed in 0.0.2
- New CLI surface is strict and minimal

### Migration Path

Existing snapshot infrastructure is preserved. Natural language capabilities will be added incrementally without breaking current performance.

---

## Previous Versions

Prior to v0.0.1, Anna was a snapshot-based telemetry daemon with fixed CLI commands. See git history for v7.x releases.
