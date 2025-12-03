# Anna Assistant - Release Notes

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
