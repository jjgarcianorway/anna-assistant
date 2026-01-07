# Contributing to Anna Assistant — CODEX Edition

This document outlines the **mandatory** development workflow and quality standards for the Anna Assistant project.

## 🔒 Core Rules

### 1. Always Use commit-if-green.sh

**NEVER commit directly.** All commits must go through `tools/commit-if-green.sh`.

```bash
# ❌ WRONG - Direct commit
git add . && git commit -m "foo"

# ✅ CORRECT - Quality-gated commit
MSG="Add feature X" tools/commit-if-green.sh
```

Or use the Makefile wrapper:

```bash
make patch && MSG="Add feature X" make commit
```

### 2. Never Push Broken Code

The `commit-if-green.sh` script enforces:

- ✅ `cargo fmt --all --check` — Code must be formatted
- ✅ `cargo clippy --all -- -D warnings` — No clippy warnings allowed
- ✅ `cargo test --all` — All tests must pass
- ✅ `cargo build --release` — Release build must succeed

If any gate fails, the commit is **blocked**. Fix the issues, then retry.

### 3. Version Bumping

Every commit must bump the version:

```bash
# Patch bump (0.5.1 -> 0.5.2)
make patch && MSG="Fix bug X" make commit

# Minor bump (0.5.2 -> 0.6.0)
make minor && MSG="Add feature Y" make commit

# Major bump (0.6.0 -> 1.0.0)
make major && MSG="Breaking change Z" make commit
```

The version is stored in:
- `VERSION` file (single source of truth)
- All workspace crate `Cargo.toml` files (auto-updated by `tools/bump.sh`)

### 4. Branch Policy

- **Do not push directly to `main`**
- Work on short-lived feature branches
- Open a PR for each milestone
- Merge only after CI passes

### 5. Commit Message Format

Commits created by `commit-if-green.sh` automatically include:

```
Your commit message summary

Version: X.Y.Z

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

The `Version:` trailer is **required** for CI to pass on main.

## 🛠️ Development Workflow

### Standard Workflow

```bash
# 1. Create a feature branch
git checkout -b feature/my-feature

# 2. Make changes
# ... edit code ...

# 3. Format code
make fmt

# 4. Check locally (optional but recommended)
make clippy
make test

# 5. Bump version and commit
make patch && MSG="Add my feature" make commit

# 6. Push branch
git push -u origin feature/my-feature

# 7. Open PR on GitHub
gh pr create --title "Add my feature" --body "Description..."
```

### Quick Snapshot Branch

Before starting risky work, create a snapshot:

```bash
make snapshot
# Creates: snapshot-2025-10-24_14-30-15
```

## 🧪 Testing

### Running Tests

```bash
# All tests
make test

# Specific crate
cd cmd/annactl && cargo test

# Specific test
cargo test test_name
```

### Test Coverage Requirements

- **New features:** Must include unit tests
- **Bug fixes:** Add regression test when possible
- **CLI features:** At least one focused integration test

## 📦 Building

```bash
# Development build
cargo build

# Release build (optimized)
make build

# Install locally
make install-user
```

## 🚀 CI Pipeline

The `.github/workflows/ci.yml` workflow runs on every push and PR:

1. **Format check** — `cargo fmt --check`
2. **Clippy** — `cargo clippy -- -D warnings`
3. **Tests** — `cargo test --all`
4. **Build** — `cargo build --release`
5. **Tag verification** (main only) — Checks version tag exists on HEAD

### CI Failure Handling

If CI fails:

1. Pull the latest changes
2. Fix the issue locally
3. Verify all gates pass: `make fmt && make clippy && make test && make build`
4. Commit fix using `make patch && MSG="Fix CI issue" make commit`
5. Push again

## 📝 Changelog

The `CHANGES.txt` file is **automatically maintained** by `commit-if-green.sh`.

Example entry:

```
v0.5.1 — 2025-10-24T14:30:15Z — Add feature X
```

**Do not edit `CHANGES.txt` manually.**

## 🎯 Milestones

For large features, break work into small milestones:

1. Each milestone = one PR
2. Keep changes minimal and local
3. No cross-cutting refactors in feature PRs
4. Preserve backward compatibility

See the main CODEX brief for current milestone definitions.

## ❌ What NOT to Do

- ❌ Direct commits without `commit-if-green.sh`
- ❌ Pushing code that doesn't pass all gates
- ❌ Skipping version bumps
- ❌ Force-pushing to `main`
- ❌ Amending commits on shared branches
- ❌ Committing secrets (`.env`, credentials, etc.)
- ❌ Large schema rewrites in feature PRs

## 🆘 Getting Help

- Check `make help` for available targets
- Read tool scripts in `tools/` directory
- Review CI output for specific failures
- Consult the main CODEX brief for milestone requirements

---

**Remember:** The guardrails exist to prevent regressions and maintain quality. If a gate blocks you, there's a real issue that needs fixing. Don't bypass the process.
