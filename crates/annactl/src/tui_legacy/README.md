# TUI Legacy Code (Archived)

**This directory contains archived TUI code that is not part of Anna 6.0.0.**

## Status

- **Disabled**: The TUI is not available in 6.0.0
- **Reason**: Prototype reset to focus on stable CLI interface
- **Future**: TUI will be rebuilt as a stable feature in future releases

## Contents

This directory contains the Beta 5.x TUI implementation:
- `tui/` - TUI modules (event loop, rendering, input handling)
- `tui_state.rs` - TUI state management
- `tui_v2.rs` - TUI entry point wrapper

## Historical Context

The 5.x TUI had several stability issues:
- Message duplication bugs
- Layout instability
- Streaming inconsistencies

Rather than continue patching, 6.0.0 represents a clean reset focusing on:
- **CLI-only interface** (stable and tested)
- **Daemon features** (Historian, ProactiveAssessment, health checks)
- **Foundation for future UI work**

## Do Not Use

This code is kept for reference only and is not compiled or tested in 6.0.0.
