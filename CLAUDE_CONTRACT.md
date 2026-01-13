# Claude Contract

Binding rules for Claude when working on anna-assistant. Violations require immediate correction.

## Permissions (ALLOWED)

Claude MAY:
- Edit existing code files in `crates/`
- Edit existing scripts in `scripts/`
- Edit existing test files in `tests/`
- Edit existing markdown files (README.md, CHANGELOG.md, SPEC.md, VISION.md, CLAUDE.md)
- Run `cargo test`, `cargo build`, `cargo clippy`, `cargo fmt`
- Run `git` commands (status, diff, log, add, commit, push, tag)
- Run `gh` commands for releases
- Delete dead code, unused files, deprecated tests
- Refactor for clarity without changing behavior
- Add tests that enforce existing contracts

## Prohibitions (FORBIDDEN)

Claude MUST NOT:
- Create new markdown files (unless explicitly requested)
- Create new directories outside existing structure
- Write to user home directories (`~`, `$HOME`, `dirs::`)
- Add new dependencies without explicit approval
- Bump version without completing all release steps
- Skip tests before committing
- Add features during cleanup/governance tasks
- Expose manual commands to users in error messages
- Use `sudo cp` for deployment
- Create backup/archive files in the repo
- Add time estimates to plans

## User-Facing Messages

All user-facing text MUST:
- NOT contain "sudo", "systemctl start", "Run:", "Try:", "Execute:"
- NOT suggest manual recovery steps
- Describe what happened, not how to fix it manually

Forbidden patterns (regex):
```
sudo systemctl
Run: sudo
Try: sudo
Execute:
Run this command
```

## File Limits

- All source files: max 400 lines
- If a file exceeds 400 lines: split into modules immediately

## System Paths (Canonical)

Code MUST use only these paths:
- Config: `/etc/anna/`
- State: `/var/lib/anna/`
- Runtime: `/run/anna/`
- Socket: `/run/anna/anna.sock`

Any use of `dirs::`, `$HOME`, `~`, or XDG paths in production code is a violation.

## Test Requirements

Before any commit:
1. `cargo test --workspace` must pass
2. `cargo clippy --workspace -- -D warnings` must pass (if available)

Before any version bump:
1. All tests pass
2. Documentation updated
3. CHANGELOG.md updated
4. No dead code introduced

## Recovery Behavior

When infrastructure fails, code MUST:
1. Attempt automatic recovery (retry, pkexec, fallback)
2. Track recovery metrics in RecoveryStatus
3. Never expose manual recovery commands to user
4. If all recovery fails: show friendly error, not instructions

## Enforcement

These rules are tested by:
- `test_no_manual_commands_in_errors` in recovery modules
- `test_no_manual_commands_in_recovery` in recovery.rs
- `test_no_manual_commands_in_service` in ollama/service.rs

Violations caught by tests block CI.
