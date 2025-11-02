# Anna Assistant - Terminal Style Guide

**Version:** 1.0
**Status:** Active
**Last Updated:** 2025-11-02

## Overview

This document defines the unified terminal UX for Anna Assistant across all components: installer scripts, CLI commands (`annactl`), and daemon logs (`annad`). The goal is consistent, retro DOS/Borland-style formatting that is sysadmin-friendly, with smart degradation for different terminal capabilities.

## Design Philosophy

1. **Consistency** - Same visual language across all tools
2. **Adaptability** - Graceful degradation from full Unicode+color to plain ASCII
3. **Clarity** - Information density without clutter
4. **Accessibility** - Respects user preferences (NO_COLOR, etc.)
5. **Nostalgia** - Retro computing aesthetic (DOS, Borland Turbo Pascal)

## Environment Variables

Anna respects the following environment variables:

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | If set (any value), disables all color output |
| `CLICOLOR` | If set to `0`, disables color output |
| `LANG` / `LC_ALL` | Checks for `UTF-8` to enable Unicode box drawing |
| `TERM` | `dumb` or `unknown` disables color; `xterm`/`tmux` enables emoji |
| `COLUMNS` | Terminal width (clamped to 60-120 columns) |

## Terminal Capability Detection

### Three Capability Levels

| Feature | Full | Degraded | Minimal |
|---------|------|----------|---------|
| Color | ✓ | ✗ | ✗ |
| UTF-8 | ✓ | ✓ | ✗ |
| Emoji | ✓ | ✗ | ✗ |

**Full Terminal:**
- Modern terminal with UTF-8 locale
- Color support enabled
- xterm/tmux/alacritty/kitty

**Degraded Terminal:**
- UTF-8 box drawing characters
- No color, no emoji
- Example: `TERM=screen NO_COLOR=1`

**Minimal Terminal:**
- ASCII-only output
- No color, no emoji, no box drawing
- Example: `TERM=dumb`, CI logs, journalctl

### Detection Algorithm (Rust)

```rust
use anna_common::TermCaps;

let caps = TermCaps::detect();
// caps.color  - ANSI colors enabled
// caps.emoji  - Wide Unicode glyphs supported
// caps.utf8   - UTF-8 box drawing available
// caps.width  - Terminal width (60-120)
```

### Detection Algorithm (Bash)

```bash
source scripts/lib/style.sh

detect_caps  # Sets ST_COLOR, ST_EMOJI, ST_UTF8, ST_WIDTH

if [ "$ST_COLOR" = "1" ]; then
    echo "Color supported"
fi
```

## Color Palette

### Pastel Colors for Dark Terminals

| Color | ANSI Code | Use Case |
|-------|-----------|----------|
| Cyan | `\x1b[38;5;87m` | Primary (headers, titles, info) |
| Green | `\x1b[38;5;120m` | Success messages |
| Yellow | `\x1b[38;5;228m` | Warnings |
| Red | `\x1b[38;5;210m` | Errors |
| Magenta | `\x1b[38;5;213m` | Accent (special highlights) |
| Gray 250 | `\x1b[38;5;250m` | Default foreground text |
| Gray 240 | `\x1b[38;5;240m` | Dim text (hints, timestamps) |

### Color Usage Rules

**DO:**
- Use green only for success/completion
- Use red only for errors/failures
- Use yellow for warnings and degraded states
- Use cyan for headings and primary UI elements
- Use gray for supplementary information

**DON'T:**
- Mix semantic colors (no red success messages)
- Use color as the only indicator (always pair with symbols)
- Use bright/bold colors excessively
- Rely on color in CI logs or non-TTY output

## Typography

### Heading Levels

#### Level 1: Header Box (Top-level title)

**Full:**
```
╭─ Anna Hardware Profile ──────────────────╮
```

**ASCII:**
```
+- Anna Hardware Profile ------------------+
```

**Usage (Rust):**
```rust
use anna_common::{TermCaps, header};

let caps = TermCaps::detect();
println!("{}", header(&caps, "Anna Hardware Profile"));
```

**Usage (Bash):**
```bash
st_header "Anna Installer"
```

#### Level 2: Section Title

**Full:**
```
System Information
──────────────────────────────────────────
```

**ASCII:**
```
System Information
------------------------------------------
```

**Usage (Rust):**
```rust
println!("{}", section(&caps, "System Information"));
```

**Usage (Bash):**
```bash
st_section "Step 1: Detection"
```

#### Level 3: Subsection (Plain text, colored)

**Full:**
```
CPU Details
```

Just the title string, colored if supported.

### Status Lines

Status lines combine symbols with messages:

| Level | Full Symbol | ASCII | Color | Example |
|-------|-------------|-------|-------|---------|
| OK | ✓ | OK | Green | `✓ Installation complete` |
| Warn | ⚠ | ! | Yellow | `⚠ dmidecode requires root` |
| Err | ✗ | X | Red | `✗ Failed to start daemon` |
| Info | ▶ | > | Cyan | `▶ Downloading binaries` |

**Usage (Rust):**
```rust
use anna_common::{TermCaps, Level, status};

let caps = TermCaps::detect();
println!("{}", status(&caps, Level::Ok, "Installation complete"));
println!("{}", status(&caps, Level::Warn, "Missing optional tool"));
println!("{}", status(&caps, Level::Err, "Build failed"));
```

**Usage (Bash):**
```bash
st_status ok "Installation complete"
st_status warn "Missing optional dependency"
st_status err "Failed to connect to daemon"
```

### Key-Value Pairs

Aligned columns for readability:

```
CPU:        AMD Ryzen 9 5900X 16 cores
Memory:     32.0 GB
GPU:        NVIDIA RTX 3080
Kernel:     6.17.6-arch1-1
```

**Usage (Rust):**
```rust
println!("{}", kv(&caps, "CPU", "AMD Ryzen 9 5900X 16 cores"));
println!("{}", kv(&caps, "Memory", "32.0 GB"));
```

**Usage (Bash):**
```bash
st_kv "CPU" "AMD Ryzen 9 5900X"
st_kv "Memory" "32 GB"
```

### Bullet Lists

```
  • Item one
  • Item two
  • Item three
```

ASCII fallback uses `-` instead of `•`.

**Usage (Rust):**
```rust
println!("{}", bullet(&caps, "Item one"));
```

**Usage (Bash):**
```bash
st_bullet "Run tests"
st_bullet "Build binaries"
st_bullet "Install to /usr/local/bin"
```

### Code Blocks

Command examples:

```
$ annactl hw show --json
$ sudo systemctl restart annad
```

**Usage (Rust):**
```rust
println!("{}", code(&caps, "annactl hw show --json"));
```

**Usage (Bash):**
```bash
st_code "annactl status"
```

### Hints (Dimmed Help Text)

```
Try: sudo systemctl status annad
```

**Usage (Rust):**
```rust
println!("{}", hint(&caps, "Try: sudo systemctl status annad"));
```

**Usage (Bash):**
```bash
st_hint "Check /var/log/anna/install.log for details"
```

### Progress Indicators

Simple text-based progress:

```
[2/5] Installing binaries
[3/5] Configuring systemd
```

**Usage (Rust):**
```rust
println!("{}", progress(&caps, "Installing binaries", 2, 5));
```

**Usage (Bash):**
```bash
st_progress 2 5 "Installing binaries"
```

### Progress Bars

Visual percentage display:

**Full:**
```
████████████░░░░░░░░ 60%
```

**ASCII:**
```
############-------- 60%
```

**Usage (Rust):**
```rust
use anna_common::progress_bar;

let bar = progress_bar(&caps, 75.0, 20);
println!("{} 75%", bar);
```

## Tables

### Auto-sized Columns

Tables automatically fit within terminal width (60-120 columns):

```
Name      Value    Status
────────  ───────  ──────
CPU       100%     OK
Memory    50%      OK
Disk      85%      WARN
```

- Headers are highlighted (cyan if color supported)
- Numbers are right-aligned
- Text is left-aligned
- Long values truncated with `…` (UTF-8) or `...` (ASCII)

**Usage (Rust):**
```rust
use anna_common::table;

let headers = vec!["Name", "Value", "Status"];
let rows = vec![
    vec!["CPU".to_string(), "100%".to_string(), "OK".to_string()],
    vec!["Memory".to_string(), "50%".to_string(), "OK".to_string()],
];

print!("{}", table(&caps, &headers, &rows));
```

### Column Width Rules

- Minimum column width: 4 characters
- Maximum total width: terminal width - (number of columns * 3)
- Proportional scaling when total exceeds available width
- Ellipsis truncation preserves readability

## Symbol Reference

| Symbol | Full | ASCII | Use Case |
|--------|------|-------|----------|
| OK | ✓ | OK | Success indicator |
| Warn | ⚠ | ! | Warning indicator |
| Error | ✗ | X | Error indicator |
| Info/Play | ▶ | > | Info/progress indicator |
| Bullet | • | - | List items |
| Folder | 📁 | [dir] | Directory references |
| Chip | 🧠 | [AI] | AI/intelligence features |
| Wrench | 🔧 | [fix] | Repair operations |
| Box corners | ╭╮╰╯ | ++++ | Box drawing (UTF-8 only) |
| Horizontal | ─ | - | Separators |
| Vertical | │ | \| | Borders |

## Message Templates

### Success Message

```rust
println!("{}", header(&caps, "Installation Complete"));
println!();
println!("{}", status(&caps, Level::Ok, "All components installed successfully"));
println!();
println!("{}", kv(&caps, "Version", "0.12.3"));
println!("{}", kv(&caps, "Install Path", "/usr/local/bin"));
println!("{}", kv(&caps, "Config", "/etc/anna"));
println!();
println!("{}", section(&caps, "Next Steps"));
println!("{}", bullet(&caps, "Start daemon: sudo systemctl start annad"));
println!("{}", bullet(&caps, "Check status: annactl status"));
```

### Warning Message

```rust
println!("{}", status(&caps, Level::Warn, "Optional dependency not found"));
println!("{}", hint(&caps, "Install for full features: sudo pacman -S nvidia-smi"));
```

### Error Message

```rust
println!("{}", status(&caps, Level::Err, "Build failed"));
println!();
println!("{}", kv(&caps, "Reason", "Missing dependency"));
println!("{}", hint(&caps, "Try: sudo pacman -S base-devel"));
```

## Bash Toolkit API

Full API reference for `scripts/lib/style.sh`:

```bash
# Source the toolkit
source scripts/lib/style.sh

# Capability detection (automatic on source)
detect_caps  # Sets ST_COLOR, ST_EMOJI, ST_UTF8, ST_WIDTH

# Formatting functions
st_header "Title"                    # Top-level header box
st_section "Section"                 # Section title/divider
st_status ok|warn|err|info "Message" # Status line with symbol
st_kv "Key" "Value"                  # Aligned key-value pair
st_bullet "Item"                     # Bullet list item
st_hint "Help text"                  # Dimmed hint
st_code "command"                    # Code/command example
st_progress 2 5 "Label"              # Progress indicator [2/5] Label
```

## Rust Toolkit API

Full API reference for `anna_common::tui`:

```rust
use anna_common::{
    TermCaps, Level,
    header, section, status, kv, bullet, hint, code, progress, table,
    progress_bar, dim, ok, warn, err, primary, accent
};

// Detect capabilities
let caps = TermCaps::detect();

// Formatting functions
header(&caps, "Title")              // Header box
section(&caps, "Title")             // Section divider
status(&caps, Level::Ok, "Message") // Status line
kv(&caps, "Key", "Value")           // Key-value pair
bullet(&caps, "Item")               // Bullet item
hint(&caps, "Help")                 // Dimmed hint
code(&caps, "command")              // Code block
progress(&caps, "Label", 2, 5)      // Progress [2/5] Label
progress_bar(&caps, 75.0, 20)       // Progress bar
table(&caps, &headers, &rows)       // ASCII table

// Color helpers
ok(&caps, "text")      // Green
warn(&caps, "text")    // Yellow
err(&caps, "text")     // Red
primary(&caps, "text") // Cyan
accent(&caps, "text")  // Magenta
dim(&caps, "text")     // Gray
```

## Testing

### Visual Regression Testing

Create snapshots for key outputs:

```bash
# Capture snapshot with full capabilities
annactl hw show > tests/snapshots/tui/hw_show_full.snap

# Capture snapshot with NO_COLOR
NO_COLOR=1 annactl hw show > tests/snapshots/tui/hw_show_nocolor.snap

# Capture snapshot in dumb terminal
TERM=dumb annactl hw show > tests/snapshots/tui/hw_show_ascii.snap
```

### Unit Testing (Rust)

```rust
#[test]
fn test_header_utf8() {
    let caps = TermCaps::full();
    let h = header(&caps, "Test");
    assert!(h.contains("╭─"));
}

#[test]
fn test_header_ascii() {
    let caps = TermCaps::none();
    let h = header(&caps, "Test");
    assert!(h.contains("+-"));
    assert!(!h.contains('\x1b')); // No ANSI codes
}
```

## Common Patterns

### Installer Script Pattern

```bash
#!/usr/bin/env bash
set -euo pipefail

# Source style toolkit
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/style.sh"

# Header
st_header "Anna Installer"
echo

# Detection phase
st_section "Step 1: Detection"
st_status info "Detecting system..."
st_kv "Architecture" "x86_64"
st_kv "OS" "Arch Linux"
echo

# Installation phase
st_section "Step 2: Installation"
st_progress 1 3 "Downloading binaries"
# ... download logic ...
st_status ok "Downloaded successfully"

st_progress 2 3 "Installing to /usr/local/bin"
# ... install logic ...
st_status ok "Installed successfully"

# Summary
echo
st_section ""
st_status ok "Installation complete"
echo
st_hint "Next: annactl status"
```

### CLI Command Pattern

```rust
use anna_common::{TermCaps, header, section, kv, status, Level, table};

pub fn show_hardware() -> Result<()> {
    let caps = TermCaps::detect();
    let profile = fetch_hardware()?;

    // Header
    println!("{}", header(&caps, "Hardware Profile"));
    println!();

    // Key information
    println!("{}", section(&caps, "System"));
    println!("{}", kv(&caps, "CPU", &profile.cpu.model));
    println!("{}", kv(&caps, "Memory", &format!("{:.1} GB", profile.memory.total_gb)));
    println!();

    // Table of devices
    println!("{}", section(&caps, "Storage Devices"));
    let headers = vec!["Device", "Size", "Type"];
    let rows: Vec<Vec<String>> = profile.storage.devices
        .iter()
        .map(|d| vec![d.name.clone(), d.size.clone(), d.type.clone()])
        .collect();
    print!("{}", table(&caps, &headers, &rows));

    // Status
    println!("{}", status(&caps, Level::Ok, "Profile collected successfully"));

    Ok(())
}
```

## Width Constraints

- **Default:** 80 columns
- **Minimum:** 60 columns
- **Maximum:** 120 columns
- **Overflow:** Truncate with ellipsis, never wrap

### Handling Long Text

```rust
// Automatic truncation in tables
let headers = vec!["Description"];
let long_text = "This is a very long description that will be truncated if it exceeds the column width";
let rows = vec![vec![long_text.to_string()]];

// Output: "This is a very long description that wi…"
print!("{}", table(&caps, &headers, &rows));
```

## Guidelines for New Code

1. **Always use the toolkit** - No ad-hoc `println!` with manual colors
2. **Detect capabilities once** - Cache `TermCaps::detect()` result
3. **Test all three modes** - Full, degraded, minimal
4. **Keep lines under 80 columns** - Use truncation helpers
5. **Pair colors with symbols** - Never rely on color alone
6. **Add snapshot tests** - Capture expected output for regression testing

## Migration Checklist

When refactoring existing output:

- [ ] Replace manual box drawing with `header()` or `section()`
- [ ] Replace colored `println!` with `status()` or color helpers
- [ ] Replace tabular output with `table()`
- [ ] Add capability detection at function entry
- [ ] Test with `NO_COLOR=1`
- [ ] Test with `TERM=dumb`
- [ ] Create snapshot test
- [ ] Update any documentation/examples

## References

- **Rust TUI Module:** `src/anna_common/src/tui.rs`
- **Bash Style Toolkit:** `scripts/lib/style.sh`
- **Example Usage:** `src/annactl/src/hw_cmd.rs`
- **Snapshot Tests:** `tests/snapshots/tui/`
- **Terminal Capability Detection:** ECMA-48, ANSI X3.64 standards
- **Box Drawing:** Unicode Box Drawing block (U+2500–U+257F)

---

**Version History:**
- v1.0 (2025-11-02): Initial style guide with unified toolkit
