# Beta.261: TUI Welcome, Exit, and UX Flow v1

**Release Date**: 2025-11-22
**Type**: UX Enhancement + Flow Coherence
**Jira**: N/A (Internal polish)

## Overview

Beta.261 addresses the UX flow pain points: how Anna welcomes you, exits, and hints at next steps. This is about making the TUI experience feel deliberate and professional, using the health and diagnostic machinery we already built.

**Key Achievement**: TUI startup, hints, and exit now feel like one coherent voice instead of scattered fragments.

## Goals Achieved

1. **TUI Startup Welcome**: Compact welcome panel using existing health data (no new RPC calls)
2. **TUI Exit Summary**: Brief goodbye screen with health summary and hint
3. **Aligned Hints**: Consistent diagnostic hint wording across CLI and TUI
4. **Health Wording Coherence**: TUI uses same health lines as canonical formatters

## Before/After Examples

### Before Beta.261

**TUI Startup**:
```
[Empty conversation, no context]
```

**TUI Exit**:
```
[Immediate terminal restoration, no goodbye]
```

**Hints**:
- Various phrases scattered across different panels
- No consistent wording

### After Beta.261

**TUI Startup**:
```
Welcome to Anna Assistant
Today: All clear, no critical issues

<ready for input>
```

Or if degraded:
```
Welcome to Anna Assistant
Today: Degraded – critical issues require attention
  2 critical, 1 warnings

<ready for input>
```

Or if daemon offline:
```
Welcome to Anna Assistant
✗ Brain diagnostics unavailable (daemon offline)
  Try: $ sudo systemctl start annad

<ready for input>
```

**TUI Exit**:
```
┌ Anna Assistant ──────────────────────────────┐
│                                              │
│ Anna Assistant v5.7.0-beta.261 on testhost  │
│ Session health: All clear, no critical...   │
│                                              │
│ Tip: Try asking "check my system health"... │
│                                              │
└──────────────────────────────────────────────┘

[Pauses 1 second, then exits]
```

**Hints**:
- Consistent: "Try asking: 'check my system health' or 'run a full diagnostic'"
- Same wording in CLI output, TUI panels, and exit screen

## Implementation Details

### 1. TUI Flow Module

**File**: `crates/annactl/src/tui/flow.rs` (NEW, 349 lines)

Created comprehensive flow management module:

```rust
/// Generate startup welcome lines for TUI conversation
pub fn generate_welcome_lines(state: &AnnaTuiState) -> Vec<Line<'static>>

/// Generate exit summary lines for TUI
pub fn generate_exit_summary(state: &AnnaTuiState) -> Vec<Line<'static>>

/// Generate canonical diagnostic hint
pub fn generate_diagnostic_hint() -> String

/// Generate daemon unavailable hint for TUI
pub fn generate_daemon_unavailable_lines() -> Vec<Line<'static>>
```

Features:
- Uses existing DailySnapshotLite from state (no new RPC calls)
- Reuses TuiStyles for consistent visual hierarchy
- Compact messages (2-4 lines for welcome, 4-5 lines for exit)
- Deterministic health wording matching canonical formatters

### 2. Event Loop Integration

**File**: `crates/annactl/src/tui/event_loop.rs`

**Startup Welcome** (lines 121-133):
```rust
// Beta.261: Show startup welcome panel using flow module
let welcome_lines = super::flow::generate_welcome_lines(state);
for line in welcome_lines {
    let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    if !text.trim().is_empty() {
        state.add_system_message(text);
    }
}
```

**Exit Summary** (lines 71-74, 89-121):
```rust
// Beta.261: Show exit summary screen before cleanup
if result.is_ok() {
    let _ = show_exit_summary(&mut terminal, &state);
}

/// Beta.261: Show exit summary screen
fn show_exit_summary(...) -> Result<()> {
    let exit_lines = super::flow::generate_exit_summary(state);

    terminal.draw(|f| {
        let paragraph = Paragraph::new(exit_lines)
            .block(Block::default()
                .title(" Anna Assistant ")
                .borders(Borders::ALL))
            .style(...);
        f.render_widget(paragraph, area);
    })?;

    // Pause for 1 second
    std::thread::sleep(Duration::from_millis(1000));
    Ok(())
}
```

### 3. Canonical Hint Wording

**Unified across all surfaces**:
- CLI continuation hints: `annactl "check my system health"`
- TUI exit screen: `Try asking "check my system health" for a full diagnostic`
- Flow module: `Try asking: "check my system health" or "run a full diagnostic"`

All refer to the same natural language phrases that trigger diagnostics.

### 4. Health Wording Alignment

**TUI uses same wording as CLI**:

| Health State | Wording |
|--------------|---------|
| Healthy | "All clear, no critical issues" |
| DegradedWarning | "Degraded – warnings detected" |
| DegradedCritical | "Degraded – critical issues require attention" |

Defined in `flow.rs::format_health_line()`, matches `diagnostic_formatter.rs` patterns.

### 5. Test Coverage

**File**: `crates/annactl/src/tui/flow.rs`

Added 7 comprehensive tests:

1. `test_welcome_healthy()` - Welcome with healthy state
2. `test_welcome_degraded_critical()` - Welcome with critical issues
3. `test_welcome_no_health()` - Welcome when daemon offline
4. `test_exit_summary_healthy()` - Exit with health data
5. `test_exit_summary_no_health()` - Exit without health data
6. `test_diagnostic_hint_consistency()` - Hint mentions both phrases
7. `test_daemon_unavailable_lines()` - Daemon error formatting

All tests deterministic, no RPC/LLM/telemetry dependencies.

## Test Results

All test suites pass:
- **TUI flow**: 7/7 ✅ (NEW)
- **TUI formatting**: 6/6 ✅
- **Health content**: 10/10 ✅
- **Daily snapshot**: 8/8 ✅

## Files Modified/Created

1. **crates/annactl/src/tui/flow.rs** (NEW) - Welcome, exit, and hint generation
2. **crates/annactl/src/tui/mod.rs** - Added flow module
3. **crates/annactl/src/tui/event_loop.rs** - Startup welcome and exit summary integration

## Breaking Changes

None. This is a purely additive UX enhancement.

## Migration Guide

No migration needed. All changes are internal improvements to TUI flow.

## Known Limitations

1. Exit summary shown for 1 second (not configurable)
2. Welcome panel uses System messages (not a dedicated welcome panel type)
3. No session time tracking (future enhancement)
4. Exit summary only shown on successful exit (not on errors)

## Future Enhancements

1. **Session Time Tracking**: Show "Session duration: 5m 32s" in exit summary
2. **Last Action Recap**: "You ran 3 diagnostics this session"
3. **Configurable Exit Delay**: Allow users to set exit pause duration
4. **Welcome Customization**: Allow disabling welcome panel via config

## Philosophy

Beta.261 follows the principle: **"Make the experience feel deliberate and professional"**

The TUI should:
- Welcome users with meaningful context (health state, session type)
- Provide consistent hints using the same language everywhere
- Exit gracefully with a summary and next-step suggestion
- Reuse existing health and diagnostic data (no duplicate RPC calls)

## Related Betas

- **Beta.260**: TUI Visual Coherence v1 (canonical answer formatting)
- **Beta.259**: Daily Snapshot Everywhere & Final Health Coherence
- **Beta.258**: "How is my system today?" Daily Snapshot v1
- **Beta.234**: Brain diagnostics background polling
- **Beta.227**: TUI error handling and graceful degradation

## Verification Steps

1. Start TUI: `annactl`
2. Verify welcome message shows health state or daemon fallback
3. Type "quit" or press Ctrl+C
4. Verify exit summary shows for ~1 second before terminal restoration
5. Check that exit summary includes version, hostname, and health
6. Verify hint wording matches: "check my system health"

## Technical Debt Paid

- TUI startup no longer silent and contextless
- Exit is graceful with summary instead of abrupt
- Hints use one canonical phrase instead of multiple variations
- Health wording matches CLI exactly (no drift)

## Credits

Developed as part of the UX flow coherence initiative to make Anna's TUI feel polished and intentional, with consistent voice and messaging from startup through exit.
