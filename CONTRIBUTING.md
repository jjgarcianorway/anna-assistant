# Contributing to Anna

Issues and PRs welcome at: https://github.com/jjgarcianorway/anna-assistant

## Design Principles

1. **Pure observation** - no modification of the system
2. **Explicit sources** - every number traceable to a real command
3. **Minimal surface** - only essential commands
4. **Local only** - no cloud, no external calls
5. **Clean separation** - Anna internals vs host monitoring
6. **Honest telemetry** - no invented numbers, real data only
7. **NO LLM** - this is a telemetry daemon, not an AI assistant

## CLI Surface Contract

The public CLI surface is EXACTLY:
- `annactl`
- `annactl status`
- `annactl sw`
- `annactl sw <name-or-category>`
- `annactl hw`
- `annactl hw <name-or-category>`

No other public commands. All arguments are case-insensitive.

---

## Repository Maintenance Policy

### Branches

- **main**: Primary development branch, always releasable
- **release tags**: Version tags (v7.34.0, etc.) for releases

### Branch Pruning

- Feature branches should be deleted after merging
- Stale branches (no activity for 30 days) may be deleted without notice
- Protected branches: main only

### Dead Code and Docs

- Unused modules, commands, and docs should be removed promptly
- Legacy code retained for transition must have clear comments explaining why
- No dead code should be user-facing

### Release Checklist

For every release:

1. Update version in Cargo.toml
2. Update lib.rs and grounded/mod.rs headers
3. Update README.md to reflect current product truth
4. Update CHANGELOG.md with changes
5. Run full test suite (`cargo test --workspace`)
6. Build release binaries
7. Create git tag and GitHub release
8. Push binaries and SHA256SUMS to release

### Code Quality

- No file should exceed 400 lines - modularization is key
- Run `cargo clippy --workspace --all-targets` before committing
- Run `cargo test --workspace` before releasing
- No truncation in output - use word wrapping
- Deterministic ordering in all output

---

## File Paths

| Path | Purpose |
|------|---------|
| `/etc/anna/config.toml` | Configuration |
| `/var/lib/anna/` | Data directory |
| `/var/lib/anna/knowledge/` | Object inventory |
| `/var/lib/anna/telemetry.db` | SQLite telemetry database |
| `/var/lib/anna/internal/` | Internal state |
| `/var/lib/anna/internal/update_state.json` | Update scheduler state |
| `/var/lib/anna/internal/ops.log` | Operations audit trail |
