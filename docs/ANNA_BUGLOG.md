# Anna Bug Log

## Legend

- `[OPEN]` - Bug is known, not yet fixed
- `[FIXED]` - Bug has been resolved
- `[WONTFIX]` - Intentional behavior or won't be changed

---

## Current Issues

### [FIXED] Spinner causes SSH session crashes during long LLM calls

**Found in**: v0.26.0
**Fixed in**: v0.27.0
**Impact**: SSH window crashes when running multiple annactl queries

**Details**: The spinner animation in annactl updated every 80ms with ANSI color escape codes. During long LLM calls (2+ minutes per query), this generated thousands of rapid prints with escape sequences. When running multiple queries (e.g., a test script with 10 questions), the accumulated terminal output caused SSH sessions to become unresponsive or crash.

**Fix**:
1. Increased spinner update interval from 80ms to 200ms (2.5x slower)
2. Added TTY detection - if stdout is not a terminal (piped, scripted), skip spinner animation entirely
3. Non-TTY mode prints static messages without escape codes

```rust
// Before: Always animated, rapid updates
std::thread::sleep(Duration::from_millis(80));

// After: TTY detection, slower updates
let is_tty = io::stdout().is_terminal();
if !is_tty {
    println!("[anna]  ... {}", message);  // Static output
    return;
}
std::thread::sleep(Duration::from_millis(200));  // 2.5x slower
```

---

### [FIXED] GPU detection causes installer arithmetic errors

**Found in**: v0.26.0
**Fixed in**: v0.26.1
**Impact**: Installer fails on systems with GPUs but missing drivers

**Details**: The `detect_gpu()` function in install.sh used `log_warn` to output warning messages about missing GPU drivers. When the function was called via command substitution `vram_mb=$(detect_gpu)`, these warnings were captured to stdout and included in the `vram_mb` variable, causing bash arithmetic errors when comparing VRAM values.

**Error message**: `arithmetic syntax error: operand expected (error token is "~  NVIDIA GPU detected but nvidia-smi not working...)`

**Fix**: Redirect log_warn calls inside detect_gpu to stderr using `>&2` so they display but don't pollute the return value.

```bash
# Before (broken)
log_warn "NVIDIA GPU detected but nvidia-smi not working - drivers may be missing"

# After (fixed)
log_warn "NVIDIA GPU detected but nvidia-smi not working - drivers may be missing" >&2
```

---

## Fixed Issues

### [FIXED] Missing `now` variable in daemon_watchdog.rs

**Found in**: v0.26.0 development
**Fixed in**: v0.26.0
**Impact**: Compilation error prevented build

**Details**: The `record_restart()` method referenced `now` variable before it was declared.

**Fix**: Added `let now = chrono::Utc::now().timestamp();` before the `if let Some(ref mut trace)` block.

---

### [FIXED] Ambiguous `UpdateResult` type

**Found in**: v0.26.0 development
**Fixed in**: v0.26.0
**Impact**: Compilation error due to name conflict

**Details**: `UpdateResult` existed in both `types.rs` and `protocol_v26.rs`, causing ambiguity.

**Fix**: Renamed to `UpdateResultV26` in protocol_v26.rs and updated all references.

---

### [FIXED] SHA256 LowerHex formatting

**Found in**: v0.26.0 development
**Fixed in**: v0.26.0
**Impact**: Checksum verification failed to compile

**Details**: `LowerHex` trait not implemented for `[u8; 32]`, couldn't format hash directly.

**Fix**: Changed from `format!("{:x}", hasher.finalize())` to iterating over bytes:
```rust
hash.iter().map(|b| format!("{:02x}", b)).collect()
```

---

### [FIXED] Unused variable warning

**Found in**: v0.26.0 development
**Fixed in**: v0.26.0
**Impact**: Warning noise in build

**Details**: Unused `now` variable at daemon_watchdog.rs:166.

**Fix**: Removed the unused variable declaration.

---

## Historical Issues

### [FIXED] GPU detection on hybrid systems

**Found in**: v0.19.0
**Fixed in**: v0.19.2
**Impact**: Incorrect hardware reporting

---

## Issue Template

When logging new issues, use this format:

```markdown
### [OPEN] Brief description

**Found in**: vX.Y.Z
**GitHub Issue**: #123 (if applicable)
**Impact**: What breaks or is affected

**Details**: Full description of the issue.

**Steps to reproduce**:
1. Step one
2. Step two
3. Expected vs actual behavior

**Proposed fix**: If known
```
