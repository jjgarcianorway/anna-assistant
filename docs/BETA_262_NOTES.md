# Beta.262: TUI Layout Level 2 – Stable Grid, Spacing, and Scroll Indicators

**Release Date**: 2025-11-22
**Type**: Layout & UX Enhancement
**Jira**: N/A (Internal polish)

## Overview

Beta.262 addresses TUI layout grid stability, spacing consistency, and scroll indicators. This is about making the layout predictable and clean like Claude CLI/Codex, with deterministic behavior across terminal sizes.

**Key Achievement**: TUI layout is now centralized, stable, and predictable with graceful degradation, scroll indicators, and no header/status bar wrapping.

## Goals Achieved

1. **Centralized Layout Grid**: Single `compute_layout()` function for all panel rectangles
2. **Panel Spacing Coherence**: Consistent borders (`Borders::ALL`) and padding across all panels
3. **Scroll Indicators**: Visual ▲/▼ indicators in conversation panel title
4. **Header/Status Bar Stability**: Deterministic truncation, always 1 line, never wraps
5. **Comprehensive Tests**: 17 layout tests covering grid, scroll, and text composition

## Before/After Examples

### Before Beta.262

**Layout Grid**:
```rust
// Scattered across render.rs:
let main_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),  // Header
        Constraint::Min(3),     // Content (unclear minimum)
        Constraint::Length(1),  // Status bar
        Constraint::Length(input_height), // Dynamic input
    ])
    .split(size);
```

**Problems**:
- No graceful degradation for small terminals
- Layout logic duplicated in event_loop.rs (input height calculation)
- No minimum conversation height guarantee

**Scroll Indicators**:
```
 Conversation [↑↓ 15/42]
```
- Numeric indicator, unclear meaning
- No visual indication of scroll direction availability

**Header/Status Bar**:
- Could wrap on narrow terminals
- No truncation logic
- Jitter when resizing

### After Beta.262

**Layout Grid**:
```rust
// Single canonical function:
pub fn compute_layout(frame_area: Rect) -> TuiLayout {
    // Returns: header, conversation, diagnostics, status_bar, input
    // Graceful degradation: diagnostics omitted if terminal < 18 lines
    // Minimum conversation: 8 lines guaranteed
}
```

**Used in render.rs**:
```rust
let layout_grid = layout::compute_layout(size);
draw_header(f, layout_grid.header, state);
draw_conversation_panel(f, layout_grid.conversation, state);
// Diagnostics conditionally shown based on layout_grid.diagnostics.height
```

**Scroll Indicators**:
```
 Conversation ▲▼
 Conversation ▼  (at top)
 Conversation ▲  (at bottom)
 Conversation    (no overflow)
```
- Visual arrows show scroll availability
- Cleaner than numeric indicator

**Header with Truncation**:
- Wide (80 cols): `Anna v5.7.0-beta.262 │ qwen2.5:3b │ user@hostname`
- Narrow (50 cols): `Anna v5.7.0-beta.262 │ qwen2.5... │ user@hostname`
- Very Narrow (30 cols): `Anna v5.7.0-beta.262 │ qwen...`
- Ultra Narrow (20 cols): `Anna v5.7.0-beta.262`

**Status Bar with Truncation**:
- Wide: `Mode: TUI │ 15:42:08 │ Health: ✓ │ CPU: 8% │ RAM: 4.2G │ Daemon: ✓ │ Brain: 2⚠`
- Narrow: `Mode: TUI │ 15:42:08 │ Health: ✓ │ CPU: 8% │ RAM: 4.2G`
- Very Narrow: `Mode: TUI │ 15:42:08 │ Health: ✓`

## Implementation Details

### 1. Layout Module

**File**: `crates/annactl/src/tui/layout.rs` (NEW, 566 lines)

Created comprehensive layout module with:

```rust
/// TUI layout structure
pub struct TuiLayout {
    pub header: Rect,
    pub conversation: Rect,
    pub diagnostics: Rect,
    pub status_bar: Rect,
    pub input: Rect,
}

/// Compute canonical TUI layout grid
pub fn compute_layout(frame_area: Rect) -> TuiLayout
```

**Graceful Degradation Logic**:
- Header: Always 1 line (minimum)
- Status bar: Always 1 line (minimum)
- Input: Always 3 lines (minimum)
- Conversation: Always 8 lines (minimum)
- Diagnostics: 5 lines (omitted if terminal < 18 lines total)

**Degradation order**:
1. Diagnostics panel hidden first (small terminals)
2. Conversation gets remaining space
3. Never produce zero-height conversation panel

### 2. Scroll Indicators

**Functions**:
```rust
pub fn should_show_scroll_up_indicator(scroll_offset: usize) -> bool
pub fn should_show_scroll_down_indicator(
    total_content_lines: usize,
    visible_lines: usize,
    scroll_offset: usize,
) -> bool
```

**Integration in `render.rs`**:
```rust
let can_scroll_up = layout::should_show_scroll_up_indicator(actual_scroll);
let can_scroll_down = layout::should_show_scroll_down_indicator(total_lines, visible_lines, actual_scroll);

let scroll_indicator = if total_lines > visible_lines {
    let up_arrow = if can_scroll_up { "▲" } else { " " };
    let down_arrow = if can_scroll_down { "▼" } else { " " };
    format!(" {}{} ", up_arrow, down_arrow)
} else {
    String::new()
};
```

### 3. Text Composition Functions

**Header Composition**:
```rust
pub fn compose_header_text(
    width: u16,
    version: &str,
    username: &str,
    hostname: &str,
    model_name: &str,
) -> String
```

**Truncation priority** (rightmost truncates first):
1. Anna version (always shown)
2. username@hostname (truncates hostname)
3. Model name (truncates first)

**Status Bar Composition**:
```rust
pub fn compose_status_bar_text(
    width: u16,
    time_str: &str,
    thinking: bool,
    health_ok: bool,
    cpu_pct: f64,
    ram_gb: f64,
    daemon_ok: bool,
    brain_critical: usize,
    brain_warning: usize,
) -> String
```

**Progressive disclosure** (rightmost omitted first):
1. Mode, Time, Health/Thinking (always shown)
2. CPU, RAM (shown if space)
3. Daemon status (shown if space)
4. Brain diagnostics (shown if space and issues exist)

### 4. Render Module Updates

**File**: `crates/annactl/src/tui/render.rs`

**Updated `draw_ui()`**:
```rust
let layout_grid = layout::compute_layout(size);

// Only split for diagnostics if height > 0
if layout_grid.diagnostics.height > 0 {
    // Horizontal split: conversation | brain panel
} else {
    // Small terminal: conversation takes full width
}
```

**Updated `draw_header()`**:
```rust
let text = layout::compose_header_text(
    area.width,
    &state.system_panel.anna_version,
    &username,
    hostname,
    &state.llm_panel.model_name,
);
```

**Updated `draw_status_bar()`**:
```rust
let text = layout::compose_status_bar_text(
    area.width,
    &time_str,
    state.is_thinking,
    health_ok,
    state.system_panel.cpu_load_1min,
    state.system_panel.ram_used_gb,
    daemon_ok,
    critical_count,
    warning_count,
);
```

### 5. Test Coverage

**File**: `crates/annactl/src/tui/layout.rs`

Added 17 comprehensive tests:

**Layout Grid Tests** (8 tests):
1. `test_layout_80x24()` - Classic terminal size
2. `test_layout_100x40()` - Larger terminal
3. `test_layout_120x30()` - Wide but moderate height
4. `test_layout_small_terminal_no_diagnostics()` - Diagnostics omitted
5. `test_layout_minimal_viable()` - Minimum viable size
6. `test_scroll_up_indicator()` - Scroll up logic
7. `test_scroll_down_indicator()` - Scroll down logic
8. `test_layout_width_propagation()` - Width consistency

**Text Composition Tests** (9 tests):
1. `test_compose_header_full_width()` - Full header with all fields
2. `test_compose_header_truncate_model()` - Model name truncation
3. `test_compose_header_truncate_hostname()` - Hostname truncation
4. `test_compose_header_minimal()` - Ultra narrow terminal
5. `test_compose_status_bar_full_width()` - Full status bar
6. `test_compose_status_bar_narrow()` - Narrow status bar
7. `test_compose_status_bar_thinking()` - Thinking indicator
8. `test_compose_status_bar_brain_critical()` - Critical issues
9. `test_compose_status_bar_brain_warning()` - Warning issues

All tests deterministic, no RPC/LLM/telemetry dependencies.

## Test Results

All test suites pass:
- **TUI layout**: 17/17 ✅ (NEW)
- **TUI flow**: 7/7 ✅
- **TUI formatting**: 6/6 ✅
- **Health content**: 10/10 ✅
- **Daily snapshot**: 8/8 ✅

## Files Modified/Created

1. **crates/annactl/src/tui/layout.rs** (NEW, 566 lines) - Layout grid and text composition
2. **crates/annactl/src/tui/mod.rs** - Added layout module
3. **crates/annactl/src/tui/render.rs** - Uses compute_layout, scroll indicators, text composition

## Breaking Changes

None. This is a purely internal layout enhancement. The TUI interface remains the same from the user's perspective.

## Migration Guide

No migration needed. All changes are internal improvements to TUI layout logic.

## Known Limitations

1. Header and status bar use uniform styling (no per-field colors) for simplicity
2. Scroll indicators in title only (not in borders themselves)
3. Diagnostics panel degrades to zero height (not responsive width split)
4. Text composition functions don't support multi-byte character awareness (may truncate mid-character)

## Future Enhancements

1. **Responsive Width Splits**: Adjust conversation/diagnostics ratio based on terminal width
2. **Per-Field Styling**: Restore granular color coding in header/status bar
3. **Border Scroll Indicators**: Render ▲/▼ in border corners instead of title
4. **Unicode-Aware Truncation**: Properly handle multi-byte characters in text composition
5. **Configurable Minimums**: Allow users to set minimum panel heights via config

## Philosophy

Beta.262 follows the principle: **"Make the layout predictable and stable across all terminal sizes"**

The TUI should:
- Use a single canonical layout function (no ad-hoc layout logic)
- Degrade gracefully on small terminals in deterministic priority order
- Never wrap header or status bar (always exactly 1 line)
- Show scroll indicators when content overflows
- Maintain consistent borders and padding across all panels

## Related Betas

- **Beta.261**: TUI Welcome, Exit, and UX Flow v1
- **Beta.260**: TUI Visual Coherence v1 (canonical answer formatting)
- **Beta.259**: Daily Snapshot Everywhere & Final Health Coherence
- **Beta.247**: PageUp/PageDown scrolling and scroll offset fixes
- **Beta.234**: Brain diagnostics background polling

## Verification Steps

1. Start TUI: `annactl`
2. Resize terminal to various sizes (80×24, 100×40, 120×30, 60×20)
3. Verify panels resize smoothly without wrapping or jitter
4. Verify diagnostics panel disappears on small terminals
5. Scroll conversation with PageUp/PageDown
6. Verify ▲/▼ indicators appear in title when scrollable
7. Verify header and status bar always 1 line, never wrap

## Technical Debt Paid

- Centralized layout logic (no duplication)
- Deterministic panel sizing with graceful degradation
- Header/status bar never wrap (guaranteed 1 line)
- Visual scroll indicators instead of confusing numeric offsets
- Comprehensive layout test coverage (17 tests)

## Credits

Developed as part of the TUI layout coherence initiative to make Anna's interface stable, predictable, and clean across all terminal sizes, matching the polish of Claude CLI/Codex.
