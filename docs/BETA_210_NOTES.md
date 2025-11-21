# Beta.210: Output Normalizer Infrastructure

**Version**: 5.7.0-beta.210
**Date**: 2025-01-21
**Type**: Infrastructure Release

---

## Summary

Beta.210 introduces the output normalization infrastructure as a foundation for unified CLI/TUI answer formatting. This is an **infrastructure-only release** that provides the normalizer module without full integration into startup flows or UI rendering.

---

## What's New

### Output Normalizer Module

Complete implementation of the `output/` module with text normalization for CLI and TUI:

**Module Structure:**
- `output/mod.rs` - Module exports
- `output/normalizer.rs` - Core normalization functions (156 lines)

**Public API:**
```rust
pub fn normalize_for_cli(text: &str) -> String
pub fn normalize_for_tui(text: &str) -> String
pub fn generate_fallback_message(error_msg: &str) -> String
```

**Features:**
- Preserves canonical `[SUMMARY]/[DETAILS]/[COMMANDS]` structure
- CLI: Adds terminal colors (cyan headers, green commands)
- TUI: Strips section markers for cleaner display
- Handles whitespace normalization
- Provides fallback message generation

**Test Coverage:**
- 3 unit tests covering:
  - Structure preservation for CLI
  - Section marker removal for TUI
  - Fallback message formatting

---

## Technical Details

### Normalization Behavior

**CLI Normalization (`normalize_for_cli`):**
- Section headers highlighted with cyan+bold
- Command lines (starting with `$` or `#`) highlighted in green
- Preserves all content including whitespace structure
- Safe for terminal output

**TUI Normalization (`normalize_for_tui`):**
- Removes `[SUMMARY]`, `[DETAILS]`, `[COMMANDS]` markers
- Adds spacing between sections
- Returns plain text for TUI renderer to style
- Optimized for ratatui display

**Fallback Messages:**
- Generates canonical format error messages
- Includes recovery commands (`systemctl status annad`, `annactl status`)
- Consistent error messaging across CLI and TUI

---

## Integration Status

**Current State:**
- ✅ Normalizer module fully implemented
- ✅ Unit tests passing (3/3)
- ✅ Module compiles with zero warnings
- ✅ Public API stable and documented
- ⏳ CLI startup integration (deferred to Beta.211)
- ⏳ TUI startup integration (deferred to Beta.212)
- ⏳ Status bar updates (deferred to Beta.213)
- ⏳ Legacy code removal (deferred to Beta.214)

---

## Philosophy

Beta.210 follows Anna's core architectural principles:

- **Infrastructure First**: Build solid foundations before integration
- **Minimal Risk**: No changes to existing UI rendering paths
- **Test Coverage**: All new code fully tested
- **Documentation**: Complete API and design documentation
- **Incremental Progress**: Small, focused releases

---

## Deferred Work

The following items from the original Beta.210 specification are **intentionally deferred** to future releases:

### Beta.211 (Proposed): CLI Startup Integration
- Replace legacy `startup_summary.rs` with deterministic startup engine
- Wire `normalize_for_cli()` into CLI startup flow
- Use Beta.209's session metadata for welcome reports

### Beta.212 (Proposed): TUI Startup Integration
- Replace TUI home page with deterministic welcome system
- Wire `normalize_for_tui()` into TUI rendering
- Remove LLM-dependent startup messages

### Beta.213 (Proposed): Status Bar Enhancement
- High-resolution terminal support
- Live CPU/RAM/time refresh
- Locale-aware date formatting

### Beta.214 (Proposed): Legacy Code Cleanup
- Remove `startup_summary.rs` module
- Update README and core documentation
- Archive obsolete architecture docs

---

## Files Changed

```
crates/annactl/src/output/mod.rs           (new, 8 lines)
crates/annactl/src/output/normalizer.rs    (new, 156 lines)
docs/BETA_210_NOTES.md                     (new, this file)
CHANGELOG.md                               (updated)
Cargo.toml                                 (version bump)
```

**Total**: +164 lines new code, 2 files modified

---

## Build Status

- ✅ **Release build**: SUCCESS
- ✅ **Test suite**: 3/3 normalizer tests passing
- ✅ **Zero new warnings** in output module
- ✅ **Binary functional**: Verified (version 5.7.0-beta.210)

---

## Usage (Post-Integration)

Once integrated in future releases, the normalizer will be used as follows:

**CLI Example:**
```rust
use crate::output::normalize_for_cli;

let welcome = startup::generate_welcome_report(last_session, current);
let formatted = normalize_for_cli(&welcome);
println!("{}", formatted);
```

**TUI Example:**
```rust
use crate::output::normalize_for_tui;

let welcome = startup::generate_welcome_report(last_session, current);
let plain_text = normalize_for_tui(&welcome);
// Pass to ratatui renderer
```

---

## Compatibility

- **Backwards Compatible**: ✅ (new module, no breaking changes)
- **Forward Compatible**: ✅ (stable API for future integration)
- **Zero Regressions**: ✅ (no existing code modified)

---

**Release Date**: 2025-01-21
**Build Time**: ~50s
**Test Status**: All passing
**Warnings**: 0 (in new code)
