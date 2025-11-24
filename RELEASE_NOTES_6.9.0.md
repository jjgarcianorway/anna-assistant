# Release Notes: Anna Assistant 6.9.0

**Release Date:** November 24, 2025
**Type:** Repository Hygiene Release
**Breaking Changes:** None

---

## Overview

Version 6.9.0 is a **repository cleanup release** with no functional changes to Anna's behavior. This release aggressively removes legacy code that no longer aligns with the CLI-only product vision established in 6.0.0.

**Key Philosophy:** "Fewer features, fully production ready"

The repository previously contained remnants of:
- Abandoned TUI experiments (5.x era)
- Consensus simulator prototypes
- Monitoring/observability dashboards
- Old beta version documentation

All of this has been removed to present a clean, maintainable codebase focused solely on the two-command CLI interface (`annactl status` + `annactl "<question>"`).

---

## What Changed

### Removed Components

**Deleted Directories:**
- `monitoring/` - Grafana dashboards, Prometheus configs (28 files)
- `observability/` - Old observability stack (8 files)
- `examples/` - Unused example code (2 files)
- `dist/` - Old distribution artifacts (5 files)
- `logrotate/` - Unused logrotate config (1 file)
- `packaging/` - AUR, deb, rpm, homebrew packaging (9 files)
- `tools/consensus_sim/` - Consensus simulator prototype (2 files)

**Cleaned Documentation:**
- Removed 47 old beta version notes (`BETA_217_NOTES.md` through `BETA_279_NOTES.md`)
- Removed QA audit documents from pre-6.0 era
- Kept only: architecture docs, planner design, recipes, testing strategy

**Total Cleanup:**
- **102 tracked files removed** from git history
- **53 untracked release-v* directories** removed from filesystem
- Repository size reduced significantly
- Cleaner `git status` output

### Updated Documentation

**README.md:**
- Version badges updated to 6.9.0
- Removed reference to deleted `BETA_279_NOTES.md`
- Updated "Future roadmap" to remove "Rebuild TUI" (CLI-only focus)
- Added recent milestones for 6.7.0, 6.8.1, and 6.9.0

**Workspace:**
- Removed `tools/consensus_sim` from `Cargo.toml` workspace members

---

## What Did NOT Change

This release contains **zero functional changes** to Anna's behavior:

- ✅ `annactl status` works identically
- ✅ `annactl "<question>"` works identically
- ✅ Daemon health monitoring unchanged
- ✅ Reflection system unchanged
- ✅ Brain analysis unchanged
- ✅ Historian unchanged
- ✅ Planner unchanged
- ✅ All 428 tests still pass

---

## Why This Release Matters

### Before 6.9.0
```
anna-assistant/
├── monitoring/          ❌ Unused Grafana dashboards
├── observability/       ❌ Unused Prometheus stack
├── examples/            ❌ Unused examples
├── packaging/           ❌ Unused package definitions
├── tools/consensus_sim/ ❌ Unused simulator
├── docs/
│   ├── BETA_217_NOTES.md  ❌ 47 old beta notes
│   ├── BETA_218_NOTES.md  ❌
│   └── ...                ❌
└── release-v5.5.0-beta.1/ ❌ 53 untracked dirs
```

The repository looked like an "archaeological dig site" - multiple layers of abandoned experiments and dead code paths.

### After 6.9.0
```
anna-assistant/
├── crates/
│   ├── annad/           ✅ Daemon
│   ├── annactl/         ✅ CLI client
│   └── anna_common/     ✅ Shared code
├── docs/
│   ├── ARCHITECTURE_BETA_200.md      ✅ System design
│   ├── PLANNER_DESIGN_6x.md          ✅ Planning system
│   ├── HISTORIAN_SCHEMA.md           ✅ History storage
│   └── RECIPES_ARCHITECTURE.md       ✅ Recipe system
├── scripts/
│   ├── install.sh       ✅ Installation
│   └── uninstall.sh     ✅ Cleanup
├── systemd/             ✅ Service files
├── security/            ✅ Signing keys
├── data/                ✅ Test fixtures
└── .github/             ✅ CI/CD
```

Clean, focused, maintainable. Everything in the repo serves the CLI-only product.

---

## Migration Notes

### For Users
No action required. Update normally:
```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

Your configuration, history, and all data remain intact.

### For Developers
If you had local branches referencing:
- `monitoring/` components
- `observability/` stack
- `tools/consensus_sim/`
- Old beta docs in `docs/BETA_*.md`

These directories are now removed. Rebase or merge carefully.

**Updated Build:**
```bash
cargo build --workspace    # consensus_sim no longer in workspace
cargo test --workspace     # All 428 tests still pass
```

---

## Verification

All builds and tests verified before release:

```bash
$ cargo build --workspace
   Compiling anna_common v6.9.0
   Compiling annad v6.9.0
   Compiling annactl v6.9.0
    Finished `release` profile [optimized] target(s)

$ cargo test --workspace
   Running unittests (428 tests)
test result: ok. 428 passed; 0 failed; 0 ignored

$ annactl status
Anna Status Check
==================================================

[REFLECTION]
✓ Anna self-awareness: healthy

[TODAY]
System healthy

[CORE HEALTH]
✓ Daemon (annad): running
✓ LLM (ollama): running
✓ Permissions: healthy
```

---

## Next Steps

**6.10.0 and beyond:** Resume feature development with clean foundation
- Expand Arch Wiki scenario coverage
- Add network diagnostics scenarios
- Implement configuration validation
- Enhance hardware monitoring

---

## Acknowledgments

This cleanup was motivated by the principle: "The repo looks like an archeological dig site at this point, not a product."

6.9.0 establishes Anna as a **focused CLI tool** with a clean, maintainable codebase - ready for production hardening and feature expansion.

---

**For full version history, see [CHANGELOG.md](CHANGELOG.md)**
