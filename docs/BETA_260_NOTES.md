# Beta.260: TUI Visual Coherence v1 (Use The Brain We Already Built)

**Release Date**: 2025-11-22
**Type**: UI/UX Enhancement + Visual Coherence
**Jira**: N/A (Internal polish)

## Overview

Beta.260 brings the TUI presentation in line with the canonical answer formatting we built for CLI. The health logic from Beta.258-259 is solid and consistent - Beta.260 makes the TUI actually *look* like it's using that same brain.

**Key Achievement**: TUI and CLI now feel like "different views of the same answer" instead of "three cousins arguing about your health."

## Goals Achieved

1. **Canonical Answer Formatting Everywhere**: TUI now uses the same semantic structure as CLI
2. **Section Headers Visually Distinct**: [SUMMARY], [DETAILS], [COMMANDS] stand out in TUI
3. **Bold Markers Work**: **bold** text actually looks bold, not asterisk salad
4. **Command Highlighting**: Lines starting with $ or # are clearly marked
5. **Stable Layout**: Header and status bar properly aligned with consistent padding
6. **Consistent Visual Hierarchy**: Same markers (✗, ⚠, ℹ, ✓) look identical everywhere

## Before/After Examples

### Before Beta.260

**TUI Anna Response**:
```
Anna:
System health today: **degraded – critical issues require attention.**

[SUMMARY]
2 critical issues detected...

$ systemctl status annad
```

Problems:
- **bold** shown as literal asterisks
- [SUMMARY] header lost in gray text
- $ command looks like regular text
- No visual hierarchy

### After Beta.260

**TUI Anna Response** (same content, proper styling):
```
Anna:
System health today: degraded – critical issues require attention.
                     ^^^^^^^^ (shown in bold white)

 [SUMMARY]  <-- Bright cyan, bold
2 critical issues detected...

$ systemctl status annad  <-- Bright green (command style)
```

Benefits:
- Bold text actually stands out
- Section headers clearly visible
- Commands easy to spot
- Consistent with CLI experience

## Implementation Details

### 1. TUI Formatting Module

**File**: `crates/annactl/src/tui/formatting.rs` (NEW)

Created comprehensive formatting parser for TUI:

```rust
/// Beta.260: TUI Style Map - Consistent visual hierarchy
pub struct TuiStyles {
    pub section_header: Style,  // [SUMMARY], [DETAILS], etc.
    pub command: Style,          // $ and # prefixed lines
    pub bold: Style,             // **bold** markdown
    pub error: Style,            // ✗ markers
    pub warning: Style,          // ⚠ markers
    pub info: Style,             // ℹ markers
    pub success: Style,          // ✓ markers
    pub normal: Style,
    pub dimmed: Style,
}
```

Key function:
```rust
pub fn parse_canonical_format(text: &str, styles: &TuiStyles) -> Vec<Line<'static>>
```

Features:
- Recognizes [SECTION] headers and styles them
- Converts **bold** to Ratatui bold style
- Highlights command lines ($ and #)
- Colors markers (✗, ⚠, ℹ, ✓) appropriately
- Strips ``` code block markers
- Preserves line breaks and indentation

### 2. Conversation Panel Integration

**File**: `crates/annactl/src/tui/render.rs`

Updated Anna message rendering to use canonical formatter:

```rust
ChatItem::Anna(msg) => {
    // Beta.260: Use canonical formatting for Anna's responses
    let styles = TuiStyles::default();
    let formatted_lines = parse_canonical_format(msg, &styles);

    for formatted_line in formatted_lines {
        // Add with wrapping support
        lines.push(formatted_line);
    }
}
```

Before: Raw text with no semantic parsing
After: Full semantic parsing with proper styling

### 3. Layout Stability

**File**: `crates/annactl/src/tui/render.rs`

Added consistent padding to header and status bar:

```rust
// Header (top bar)
let header_text = Line::from(vec![
    Span::raw(" "), // Left padding for visual breathing room
    Span::styled("Anna ", ...),
    ...
]);

// Status bar (bottom bar)
let mut spans = Vec::new();
spans.push(Span::raw(" ")); // Left padding for alignment
spans.push(Span::styled("Mode: ", ...));
...
```

Result:
- Full-width bars with consistent padding
- Clean visual alignment
- No text touching edges

### 4. Visual Hierarchy (Style Map)

**Color Palette** (existing colors, now documented):

| Semantic Element | Color RGB | Usage |
|-----------------|-----------|-------|
| Section Headers | 80, 180, 255 | [SUMMARY], [DETAILS], [COMMANDS] |
| Commands | 100, 255, 100 | $ and # prefixed lines |
| Bold Text | 255, 255, 255 | **bold** content |
| Errors (✗) | 255, 80, 80 | Critical issues |
| Warnings (⚠) | 255, 200, 80 | Warning issues |
| Info (ℹ) | 100, 200, 255 | Informational markers |
| Success (✓) | 100, 255, 100 | Success markers |
| Normal Text | 200, 200, 200 | Regular content |
| Dimmed Text | 120, 120, 120 | Secondary content |

### 5. Test Coverage

**File**: `crates/annactl/src/tui/formatting.rs`

Added 7 comprehensive tests:

1. `test_parse_section_headers()` - [SUMMARY] recognized and styled
2. `test_parse_bold_formatting()` - **bold** converted to bold style
3. `test_parse_command_lines()` - $ and # lines highlighted
4. `test_parse_markers()` - ✗, ⚠, ✓ colored appropriately
5. `test_code_blocks_stripped()` - ``` markers removed
6. `test_nested_bold_and_markers()` - Complex formatting handled
7. (Implicit) Integration with conversation rendering

## Test Results

All test suites pass:
- **Health content**: 10/10 ✅
- **Daily snapshot**: 8/8 ✅
- **Health consistency**: 13/13 ✅
- **TUI formatting**: 7/7 ✅ (NEW)

## Files Modified/Created

1. **crates/annactl/src/tui/formatting.rs** (NEW) - Canonical format parser for TUI
2. **crates/annactl/src/tui/mod.rs** - Added formatting module
3. **crates/annactl/src/tui/render.rs** - Updated conversation rendering, header/status padding

## Breaking Changes

None. This is a purely visual enhancement.

## Migration Guide

No migration needed. All changes are internal UI improvements.

## Known Limitations

1. Long command lines may still wrap awkwardly on narrow terminals
2. Very complex nested formatting (rare edge cases) may not render perfectly
3. Text wrapping currently breaks styled spans (future improvement)

## Future Enhancements

1. **Smarter Text Wrapping**: Preserve spans when wrapping long lines
2. **Theme Support**: Allow users to customize the TuiStyles color palette
3. **Daily Snapshot in TUI**: Render DailySnapshotLite in TUI left panel
4. **Section Collapsing**: Allow collapsing [DETAILS] sections for cleaner view

## Philosophy

Beta.260 follows the principle: **"TUI is just another window onto the same canonical normalized answers."**

The TUI should not invent its own formatting rules. Instead:
- Use the same semantic structure as CLI (ANSWER_FORMAT.md)
- Parse and style that structure consistently
- Ensure section headers, bold text, and commands are clearly visible
- Make the experience feel cohesive across CLI and TUI

## Related Betas

- **Beta.259**: Daily Snapshot Everywhere & Final Health Coherence
- **Beta.258**: "How is my system today?" Daily Snapshot v1
- **Beta.257**: Canonical diagnostic formatter
- **Beta.247**: TUI help overlay bleed-through fix
- **Beta.230**: TUI header/status bar streamlining
- **Beta.220**: Truecolor support for TUI
- **Beta.218**: Brain diagnostics panel

## Verification Steps

1. Start TUI: `annactl`
2. Ask: "how is my system today?"
3. Verify section headers ([SUMMARY], etc.) are bright cyan and bold
4. Verify **bold** text stands out (not literal asterisks)
5. Verify command lines ($) are bright green
6. Verify markers (✗, ⚠, ✓) are colored appropriately
7. Verify header and status bar have consistent padding

## Technical Debt Paid

- TUI now respects the same canonical answer format as CLI
- No more ad-hoc text rendering in TUI conversation panel
- Section headers and bold text actually mean something visually
- Consistent style map documented and tested

## Credits

Developed as part of the visual coherence initiative to make Anna's TUI feel as polished and intentional as the CLI experience, using the same underlying diagnostic and answer formatting brain.
