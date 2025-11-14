## [5.5.1-beta.1] - 2025-11-14

### Patch Release: Critical Bug Fixes for v5.5.0

**This patch release fixes 4 critical bugs discovered during first real-world testing on Arch Linux.**

---

#### Bug Fixes

**1. Database Schema Migration (v5.4 → v5.5 Compatibility)** ✅

**Problem**: Users upgrading from v5.4 encountered error:
  table user_preferences has no column named updated_at

**Fix**:
- Added backwards-compatible migration system to ContextDb
- Migration automatically adds `updated_at` column if missing
- Migrates existing data from `set_at` column
- No manual intervention required
- Added 3 comprehensive tests for upgrade path

**Impact**: Users can now upgrade from v5.4 to v5.5+ without errors

---

**2. Ollama Installer Robustness** ✅

**Problem**: Installer failed with unhelpful message when:
- Ollama not in official Arch repos
- AUR helper (yay) not installed
- Error: "Failed to install Ollama via pacman or yay"

**Fix**:
- Check `pacman -Si ollama` first (official repos)
- Only fall back to AUR if no official package exists
- Improved error messages with actionable steps:
  • Ollama is not in official Arch repos
  • No AUR helper (yay) found

  To fix this:
  1. Install an AUR helper: sudo pacman -S yay
  2. Or manually install Ollama: curl -fsSL https://ollama.com/install.sh | sh
  3. Then run this setup again, or choose 'Remote API' option

- Wizard now offers "Configure remote API instead" when local setup fails
- No more silent failures

**Impact**: Clear guidance when automatic Ollama install isn't possible

---

**3. Installer Terminal Formatting** ✅

**Problem**: Raw markdown code fences (```) printed in terminal during install
- Installer's "What's new" section showed: ```bash instead of clean output

**Fix**:
- Removed markdown code fences from release notes
- Used indentation instead for code examples
- Installer output now displays cleanly

**Impact**: Professional, clean install experience

---

**4. LLM Wizard Re-entry** ✅

**Problem**: If LLM setup failed or was skipped, users couldn't retry later
- No way to manually trigger wizard again
- System could be left in half-configured state

**Fix**:
- Added `Intent::SetupBrain` for manual wizard triggering
- Users can now say: "Anna, set up your brain" to re-run wizard
- Wizard always saves consistent config (never half-configured)
- Failed setup offers graceful choice: remote API or disabled mode

**Impact**: Users can retry LLM setup anytime without reinstalling

---

#### Technical Details

**Files Modified**:
- `crates/anna_common/src/context/db.rs` - Migration system (+44 lines)
- `crates/anna_common/src/ollama_installer.rs` - Robust install flow
- `crates/annactl/src/llm_wizard.rs` - Better failure handling
- `crates/annactl/src/intent_router.rs` - New SetupBrain intent
- `crates/annactl/src/main.rs` - SetupBrain handler
- `crates/annactl/src/repl.rs` - SetupBrain handler
- `release-notes-5.5.0-beta.1.md` - Formatting fixes

**Testing**:
- All DB migration tests passing (5/5) ✅
- `cargo build --release` succeeds ✅
- `cargo test --all` passes ✅
- No schema errors on v5.4 → v5.5+ upgrade ✅

**Safety Guarantees Preserved**:
- ✅ Local LLM default recommendation
- ✅ Remote API explicit opt-in with warnings
- ✅ Per-change sudo requests (no global privilege)
- ✅ File backup system for all modifications
- ✅ One-time notifications (no nagging)

---

#### Migration Notes

**Automatic Update**:
If you installed Anna manually (via curl installer), your daemon will:
- Detect this new version within ~10 minutes
- Download and verify binaries (SHA256 checksums)
- Backup current binaries
- Atomically swap to new version
- Restart seamlessly
- Show you this update notice on next interaction

**Manual Update**:
If you want to update immediately:

  curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash

**Package Manager Installations**:
If you installed via AUR/pacman:
- Anna will notify you that an update is available
- Update using your package manager: `yay -Syu anna` or `pacman -Syu anna`

---

**Version**: 5.5.1-beta.1
**Status**: Production-ready patch release
**Tested on**: Arch Linux x86_64
