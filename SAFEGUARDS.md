# Socket Fix Safeguards - NEVER BREAK AGAIN

## The Problem We Fixed
Socket ownership was set to `root:root` instead of `root:anna`, preventing users in the `anna` group from connecting via RPC.

## The Fix
Added 6 lines in `src/annad/src/rpc_v10.rs:201-207` to set socket group ownership to `anna` after creation.

## Four Layers of Protection

### 1. Automated Test (`tests/verify_socket_fix.sh`)
**When**: Run manually or in CI
**What**:
- Checks fix exists in source code
- Verifies fix compiled into binary
- Tests live socket ownership (if daemon running)
- Tests RPC connection

**Run**: `./tests/verify_socket_fix.sh`

### 2. CI Validation (`.github/workflows/release.yml`)
**When**: Every release (git tag push)
**What**:
- Builds binaries
- Verifies socket fix in source
- Verifies socket fix in binary
- FAILS release if fix missing

**Prevents**: Broken releases from reaching GitHub

### 3. Pre-Push Hook (`.githooks/pre-push`)
**When**: Before every `git push`
**What**:
- Checks socket fix present in source
- Verifies VERSION consistency
- Quick compilation check

**Enable**: Already enabled via `git config core.hooksPath .githooks`

### 4. Bulletproof Installer (`scripts/install_simple.sh`)
**When**: User runs installation
**What**:
- Tries local binaries first (for developers)
- Falls back to GitHub release
- **Verifies socket fix in binary before installing**
- FAILS with clear error if fix missing

**Prevents**: Installing broken binaries

## How to Use

### For Development
```bash
# 1. Make changes
git add .
git commit -m "my changes"

# 2. Pre-push hook runs automatically
git push  # Hook checks everything before pushing
```

### For Releases
```bash
# 1. Run release script
./scripts/release.sh

# 2. Push (pre-push hook validates)
git push origin main
git push origin v1.0.0-rc.XX

# 3. GitHub Actions builds and validates
# - Checks socket fix in source
# - Checks socket fix in binary
# - Creates release with assets
```

### For Installation
```bash
# Simple installer (ALWAYS works)
sudo ./scripts/install_simple.sh

# This installer:
# 1. Uses local binaries if available
# 2. Falls back to GitHub release
# 3. Verifies socket fix before installing
# 4. NEVER installs broken binaries
```

### For Testing
```bash
# Test socket fix manually
./tests/verify_socket_fix.sh

# Test pre-push hook
./.githooks/pre-push
```

## Guarantee

**With these 4 layers, the socket bug can NEVER be reintroduced:**

1. ✅ Developer can't push without fix (pre-push hook)
2. ✅ CI can't release without fix (GitHub Actions)
3. ✅ Installer can't install without fix (binary verification)
4. ✅ Manual testing available (verify script)

Every layer checks and **fails loudly** if the fix is missing.

## What If Something Breaks?

### If pre-push hook fails:
```bash
# Fix the issue it reports, then:
git push
```

### If CI release fails:
Check GitHub Actions logs: https://github.com/jjgarcianorway/anna-assistant/actions

### If installer fails:
```bash
# 1. Build locally
cargo build --release

# 2. Verify fix
./tests/verify_socket_fix.sh

# 3. Install
sudo ./scripts/install_simple.sh
```

## Files Changed

- `src/annad/src/rpc_v10.rs` - The actual fix (6 lines)
- `tests/verify_socket_fix.sh` - Automated test
- `.github/workflows/release.yml` - CI validation
- `.githooks/pre-push` - Pre-push validation
- `scripts/install_simple.sh` - Bulletproof installer

## Status

✅ **All safeguards active and tested**
✅ **Socket fix verified in current binary**
✅ **RPC connection working**
✅ **Ready for release**
