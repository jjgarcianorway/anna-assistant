# Claude Code Instructions

## Code Quality
- Keep all files under 400 lines. Modularization and scalability is key.

## Release Workflow - CRITICAL - DO NOT SKIP ANY STEP!!!
After completing any implementation work, run ALL these steps:

### Quick Release Checklist (COPY-PASTE THIS EVERY TIME)
```bash
# 1. Update version in Cargo.toml
# Edit Cargo.toml: version = "0.0.XX"

# 2. Update CHANGELOG.md with changes

# 3. Run tests
cargo test --workspace

# 4. Commit and push
git add -A
git commit -m "v0.0.XX: Description"
git push origin main

# 5. Create and push tag
git tag v0.0.XX
git push origin v0.0.XX

# 6. Create GitHub release
gh release create v0.0.XX --title "v0.0.XX" --notes "Release notes"

# 7. BUILD RELEASE BINARIES
cargo build --release --workspace

# 8. PREPARE AND UPLOAD BINARIES (THIS IS WHAT AUTO-UPDATE NEEDS!!!)
cp target/release/annactl annactl-linux-x86_64
cp target/release/annad annad-linux-x86_64
sha256sum annactl-linux-x86_64 annad-linux-x86_64 > SHA256SUMS
gh release upload v0.0.XX annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS --clobber
rm annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS
```

⚠️ **STEPS 7-8 ARE MANDATORY** - Anna auto-update downloads binaries from GitHub releases!
Without uploading `annactl-linux-x86_64`, `annad-linux-x86_64`, and `SHA256SUMS`, users will NOT receive the update!

## Project Structure
- `crates/anna-shared/` - Shared types and utilities
- `crates/annad/` - Daemon (backend)
- `crates/annactl/` - CLI client
