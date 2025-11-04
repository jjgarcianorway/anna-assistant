# Git Hooks for Anna Assistant

This directory contains Git hooks to help maintain version consistency.

## Installation

To enable these hooks, run:

```bash
git config core.hooksPath .githooks
```

## Available Hooks

### pre-commit

Runs before each commit to:
- Detect version changes in `Cargo.toml`
- Warn about version mismatches between source and built binaries
- Provide helpful reminders after version bumps

## Disabling Hooks

If you need to bypass hooks for a specific commit:

```bash
git commit --no-verify
```

## Manual Version Check

You can always run the comprehensive version check manually:

```bash
./scripts/verify_versions.sh
```
