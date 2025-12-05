# Claude Code Instructions

## Code Quality
- Keep all files under 400 lines. Modularization and scalability is key.

## Release Workflow - CRITICAL
After completing any implementation work:

1. **Always push to GitHub** - anna has auto-update enabled, so changes must be pushed for users to receive updates
2. **Update VERSION file** - Bump version when making releases
3. **Update CHANGELOG.md** - Document changes in the changelog
4. **Create git tag** for releases - e.g., `git tag v0.0.23`

### Quick Release Checklist
```bash
# 1. Verify tests pass
cargo test --workspace

# 2. Stage and commit changes
git add -A
git commit -m "v0.0.XX: Description"

# 3. Push to GitHub (REQUIRED for auto-update!)
git push origin main

# 4. For releases, also tag
git tag v0.0.XX
git push origin v0.0.XX
```

## Project Structure
- `crates/anna-shared/` - Shared types and utilities
- `crates/annad/` - Daemon (backend)
- `crates/annactl/` - CLI client
