# Anna Beautification Guide

**"Not a single ugly output shall pass."** — The Law

Every message Anna speaks must be beautiful, calm, and competent. This guide ensures consistency across all commands.

---

## 🎨 The Beautiful Output Library

All beautiful output flows through `/src/anna_common/src/beautiful.rs`:

```rust
use anna_common::beautiful::*;
use anna_common::beautiful::colors::*;
use anna_common::beautiful::boxes::*;
```

### Available Functions

#### Headers & Sections
```rust
header("Anna System Status")       // Ceremonial double-line header
section("Phase 1: Detection")       // Section divider with ━━━
```

#### Status Messages
```rust
success("Installation complete")    // Green checkmark ✓
error("Failed to connect")          // Red cross ✗
warning("Queue depth elevated")     // Yellow warning ⚠
info("Backup created")              // Cyan info ℹ
```

#### Progress Indicators
```rust
step("🔍", "Finding latest release")          // Blue with emoji
substep("Latest version: v1.0.0-rc.9")         // Indented gray ↳
progress_bar(50, 100, 40)                      // [████████░░░░] 50%
```

#### Boxes & Containers
```rust
box_with_content("Status: Healthy\nUptime: 3d 12h")  // Rounded box
celebration("✨ Installation Complete! ✨", &details) // Double-line celebration
```

#### Utility Formatters
```rust
duration(3661)        // "1h 1m"
file_size(1048576)    // "1.0 MB"
```

#### Anna's Voice
```rust
voice::greeting("lhoqvso")   // "Hello lhoqvso. I'm ready to assist."
voice::working()              // "Working on it..."
voice::complete()             // "All done."
voice::recovered()            // "I've recovered. Thank you for your patience."
voice::healthy()              // "All systems nominal."
```

---

## 🌈 Color Palette

### Semantic Colors
- **Green** (`GREEN`): Success, healthy states
- **Yellow** (`YELLOW`): Warnings, non-critical issues
- **Red** (`RED`): Errors, critical failures
- **Cyan** (`CYAN`): Headers, informational
- **Blue** (`BLUE`): Progress, working states
- **Magenta** (`MAGENTA`): Highlights, version numbers
- **Gray** (`GRAY`, `DIM`): Metadata, timestamps
- **White** (`WHITE`, `BOLD`): Key values, emphasis

### Usage Examples
```rust
println!("{GREEN}✓ Success{RESET}");
println!("{YELLOW}⚠ Warning{RESET}");
println!("{RED}✗ Error{RESET}");
println!("{CYAN}{BOLD}Anna Status{RESET}");
```

---

## 📦 Box Drawing Characters

### Rounded Corners (Friendly, Default)
```
╭─────────────╮
│   Content   │
╰─────────────╯
```

Use: `TOP_LEFT`, `TOP_RIGHT`, `BOTTOM_LEFT`, `BOTTOM_RIGHT`, `HORIZONTAL`, `VERTICAL`

### Double Lines (Ceremonial Moments)
```
╔═══════════════╗
║  🎉 Success!  ║
╚═══════════════╝
```

Use: `DOUBLE_TOP_LEFT`, `DOUBLE_TOP_RIGHT`, `DOUBLE_BOTTOM_LEFT`, `DOUBLE_BOTTOM_RIGHT`

Use double lines for:
- Installation complete
- Upgrade successful
- Major celebrations
- Critical headers

---

## ✨ When To Use What

### Commands (annactl)

| Command | Style | Example |
|---------|-------|---------|
| `status` | Rounded box with colored sections | See `status_cmd.rs` |
| `doctor` | Step-by-step with progress | "Phase 1: Detection" |
| `report` | Structured sections with boxes | Health score, recommendations |
| `advisor` | Numbered list with emoji | "1. 🔄 Enable auto-updates" |
| `install` | Multi-phase with spinners | See `install.sh` |

### Error Messages

**Bad** ❌
```
Error: Connection refused
```

**Good** ✅
```
╭────────────────────────────────────────╮
│  ✗ Connection Error                     │
│                                          │
│  Failed to connect to daemon socket     │
│  /run/anna/annad.sock                    │
│                                          │
│  → Is the daemon running?                │
│    sudo systemctl start annad            │
│                                          │
╰────────────────────────────────────────╯
```

### Success Messages

**Bad** ❌
```
Done.
```

**Good** ✅
```
╔═══════════════════════════════════════════╗
║                                           ║
║        ✨  All Done! ✨                    ║
║                                           ║
║     Anna is ready to assist               ║
║                                           ║
╚═══════════════════════════════════════════╝

  Next steps:
    annactl status   # Check system health
    annactl report   # View full report
```

---

## 🎭 Anna's Personality

### Tone
- **Calm**: Never panicked, never frantic
- **Competent**: Knows what she's doing
- **Brief**: Says what's needed, nothing more
- **Helpful**: Always suggests next steps

### Voice Examples

#### Working
```
→ Reading current version from Cargo.toml...
→ Checking for code changes since v1.0.0-rc.9...
→ Fetching latest tag from GitHub API...
```

#### Success
```
✓ Backup created: /var/lib/anna/backups/pre-upgrade-v1.0.0.tar.gz
✓ Database migrated (2.3s)
✓ Health check passed
```

#### Error (With Recovery)
```
✗ Database integrity check failed
  → Running automatic repair...
  ✓ Repair complete (8.7s)
  ✓ System restored
```

#### Celebration
```
╔═══════════════════════════════════════════╗
║                                           ║
║     ✨  Upgrade Successful! ✨             ║
║                                           ║
║        Anna v1.0.0 is now running         ║
║                                           ║
╚═══════════════════════════════════════════╝

  📊 Upgrade Statistics:
     • Duration: 23.4s
     • Downtime: 4.1s
     • Data preserved: 100%

✨ Anna says: "I feel sharper already. Thank you for keeping me updated."
```

---

## 📝 Code Examples

### Before (Ugly) ❌
```rust
println!("Error: failed to connect");
eprintln!("daemon not running");
println!("exit code: {}", code);
```

### After (Beautiful) ✅
```rust
use anna_common::beautiful::*;

println!("{}", error("Failed to connect to daemon"));
println!("{}", info("Daemon may not be running"));
println!("{}", substep("Exit code: {}"), code);
```

### Full Example: Beautiful Status Display
```rust
use anna_common::beautiful::*;
use anna_common::beautiful::colors::*;
use anna_common::beautiful::boxes::*;

pub fn display_status(status: &Status) -> Result<()> {
    // Header
    println!("\n{DIM}{TOP_LEFT}{}{TOP_RIGHT}",
        HORIZONTAL.repeat(60));

    // Title with emoji
    let (emoji, color) = match status.state {
        State::Healthy => ("✅", GREEN),
        State::Degraded => ("⚠️ ", YELLOW),
        State::Failed => ("❌", RED),
    };

    println!("{VERTICAL}{RESET}  {color}{BOLD}{emoji} Anna System Status{RESET}  {DIM}{VERTICAL}{RESET}");
    println!("{VERTICAL}{RESET}                                                   {DIM}{VERTICAL}{RESET}");

    // Content
    println!("{VERTICAL}{RESET}  {BOLD}Daemon:{RESET} {GREEN}running{RESET}  {DIM}│{RESET}  {BOLD}PID:{RESET} {}  {DIM}{VERTICAL}{RESET}",
        status.pid);

    // Footer
    println!("{DIM}{BOTTOM_LEFT}{}{BOTTOM_RIGHT}\n{RESET}",
        HORIZONTAL.repeat(60));

    Ok(())
}
```

---

## 🚀 Migration Checklist

To beautify a command:

1. **Import the library**
   ```rust
   use anna_common::beautiful::*;
   use anna_common::beautiful::colors::*;
   use anna_common::beautiful::boxes::*;
   ```

2. **Replace `println!` with beautiful functions**
   - `println!("Success")` → `println!("{}", success("Success"))`
   - `println!("Error: {}", e)` → `println!("{}", error(&format!("Error: {}", e)))`
   - `eprintln!("WARN")` → `println!("{}", warning("Warning"))`

3. **Add structure**
   - Use `section()` to divide major parts
   - Use `step()` and `substep()` for progress
   - Use boxes for important messages

4. **Add color to key values**
   - Version numbers: `{MAGENTA}v1.0.0{RESET}`
   - Durations: `{CYAN}23.4s{RESET}`
   - Status: `{GREEN}healthy{RESET}` / `{RED}failed{RESET}`

5. **End with advice**
   - Always suggest next steps
   - Use `voice::` functions for personality

---

## 🎯 Priority Order

Beautify in this order (highest impact first):

1. ✅ **Status command** — Most visible (DONE)
2. **Doctor command** — Self-healing output
3. **Install script** — First impression (DONE)
4. **Error messages** — Error display library
5. **Report command** — Health reports
6. **Advisor command** — Recommendations
7. **All other commands** — Apply systematically

---

## 🔥 The Golden Rule

> **"If a human has to read it, make it beautiful."**

No exceptions. Every `println!`, every error, every log message that reaches human eyes must flow through the beautiful output library.

---

**Remember**: Anna is not just a tool. She's a living command-line organism. Every output is her voice. Make it sing. 🌸
