# Claude Code Instructions

## Code Quality
- Keep all files under 400 lines. Modularization and scalability is key.

## Release Workflow - CRITICAL - DO NOT SKIP ANY STEP!!!
After completing any implementation work:

1. **Update VERSION file** - Bump version when making releases
2. **Update CHANGELOG.md** - Document changes in the changelog
3. **Run tests** - `cargo test --workspace`
4. **Commit and push** - `git add -A && git commit && git push origin main`
5. **Create and push tag** - `git tag v0.0.XX && git push origin v0.0.XX`
6. **CREATE GITHUB RELEASE** - `gh release create v0.0.XX --title "v0.0.XX" --notes "description"`

⚠️ **STEP 6 IS MANDATORY** - Anna auto-update checks GitHub RELEASES, not just tags!
Without `gh release create`, users will NOT receive the update!

### Quick Release Checklist
```bash
# 1. Verify tests pass
cargo test --workspace

# 2. Stage and commit changes
git add -A
git commit -m "v0.0.XX: Description"

# 3. Push to GitHub
git push origin main

# 4. Create and push tag
git tag v0.0.XX
git push origin v0.0.XX

# 5. CREATE THE GITHUB RELEASE (THIS IS WHAT TRIGGERS AUTO-UPDATE!!!)
gh release create v0.0.XX --title "v0.0.XX" --notes "Release notes here"
```

## Project Structure
- `crates/anna-shared/` - Shared types and utilities
- `crates/annad/` - Daemon (backend)
- `crates/annactl/` - CLI client
