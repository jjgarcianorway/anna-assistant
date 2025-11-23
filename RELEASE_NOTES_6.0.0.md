# Release Notes: Anna 6.0.0 - Prototype Reset

**Release Date:** 2025-11-23

---

## Overview

Anna 6.0.0 represents a **complete reset** of the project's direction. After extensive work on the 5.x TUI proved unstable, we made the decision to step back, disable the UI, and focus on what actually works: the daemon, CLI, and core intelligence features.

**This is not a continuation of 5.x. It is a fresh start.**

---

## Philosophy

Version 6.0.0 is explicitly:
- **Experimental** - Not production-ready
- **Prototype-focused** - Building foundations, not features
- **Honest** - Only claiming what genuinely works
- **CLI-first** - Terminal UI will return when it can be done right

The goal is to create a **stable, minimal, boring foundation** that can be built upon, rather than continuing to patch an unstable UI.

---

## What Changed in 6.0.0

### ❌ Removed / Disabled

1. **Interactive TUI**
   - The entire 5.x TUI has been disabled
   - Code archived in `crates/annactl/src/tui_legacy/`
   - Reason: Persistent instability (message duplication, layout bugs, streaming issues)

2. **TUI-Related Features**
   - Right panel displays
   - Brain panel
   - Welcome flow screens
   - Streaming UI
   - Multi-panel layouts
   - REPL interface

3. **Repository Clutter**
   - Obsolete documentation moved to `docs/archive/`
   - Legacy tests moved to `tests_legacy/`
   - Dead code removed or archived
   - Misleading README claims eliminated

### ✅ Kept / Stable

1. **Daemon (`annad`)**
   - All Beta.279 daemon features remain **unchanged and stable**
   - Historian (JSONL storage, 6 temporal rules)
   - ProactiveAssessment (correlated issue detection)
   - Diagnostic engine (9 rules)
   - 77+ deterministic recipes
   - RPC server

2. **CLI Interface (`annactl`)**
   - `annactl status` - System health check
   - `annactl "<question>"` - One-shot natural language queries
   - `annactl --help` - Help text
   - JSON output mode (`--json`)

3. **LLM Integration**
   - Ollama integration works
   - Natural language query processing
   - System prompt architecture

### 🔧 Fixed

1. **Build System**
   - Compiles cleanly without TUI modules
   - No dead module references
   - Clear separation between active and archived code

2. **Documentation**
   - README.md now accurately describes what works
   - No false promises about features
   - Clear warnings about experimental status
   - Proper architecture documentation links

3. **Tests**
   - Legacy TUI tests moved to `tests_legacy/`
   - Active tests reflect actual 6.0.0 functionality
   - No pretense of test coverage we don't have

---

## Migration Guide

### From 5.x

**If you were using the TUI:**
- The TUI is gone. Use `annactl status` or one-shot queries instead.
- No migration path exists - TUI will be rebuilt from scratch in future releases

**If you were using CLI/daemon:**
- Everything still works exactly as before
- No changes needed

**Configuration:**
- All existing configuration remains compatible
- Daemon behavior unchanged

---

## Architecture in 6.0.0

```
┌─────────────────────────────────────┐
│         annactl (Client)            │
│  ┌──────────┐      ┌─────────────┐ │
│  │  status  │      │  NL queries │ │
│  └──────────┘      └─────────────┘ │
└─────────────┬───────────────────────┘
              │ (RPC over Unix socket)
┌─────────────▼───────────────────────┐
│         annad (Daemon)              │
│  ┌──────────────────────────────┐  │
│  │  Diagnostic Engine (9 rules) │  │
│  ├──────────────────────────────┤  │
│  │  Historian (temporal rules)  │  │
│  ├──────────────────────────────┤  │
│  │  ProactiveAssessment         │  │
│  ├──────────────────────────────┤  │
│  │  77+ Recipes                 │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
```

**Simple, stable, and focused.**

---

## What 6.0.0 is NOT

- ❌ **Not a bug fix release** - It's a reset
- ❌ **Not backward compatible** with TUI usage
- ❌ **Not production software** - Experimental prototype
- ❌ **Not feature-complete** - Intentionally minimal

---

## Future of Anna (6.x Line)

The 6.x series will focus on:

1. **Stability first** - No new features until foundation is solid
2. **CLI maturity** - Make CLI experience excellent
3. **Testing expansion** - Real test coverage
4. **Documentation** - Accurate, helpful docs
5. **Eventually: New UI** - When ready, build a TUI/Web UI that actually works

**6.1.0+** will incrementally add features only when they can be done right.

---

## Known Issues

1. **No interactive interface** - By design, will return in future releases
2. **Warnings during build** - Dead code warnings expected (archived modules)
3. **Incomplete test coverage** - Acknowledged and documented

---

## Installation

**From source:**
```bash
git clone https://github.com/jjgarcianorway/anna-assistant.git
cd anna-assistant
cargo build --release
sudo cp target/release/annad /usr/local/bin/
sudo cp target/release/annactl /usr/local/bin/
```

**Via install script:**
```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

---

## Testing

Run the active test suite:
```bash
cargo test
```

Legacy tests (not run by default):
```bash
# These are archived and may not pass
cargo test --manifest-path crates/annactl/tests_legacy/Cargo.toml
```

---

## Feedback

6.0.0 is a clean slate. We want to hear:
- What works well in the CLI experience
- What's confusing or broken
- What features you actually need (not hypothetically want)

**Report issues:** https://github.com/jjgarcianorway/anna-assistant/issues

---

## Credits

This reset was made possible by honest reflection on what wasn't working and the courage to step back rather than continue patching.

Thank you to everyone who provided feedback on 5.x. Your patience and criticism led to this reset.

---

**Anna 6.0.0: A focused, honest prototype. Building a foundation, not chasing features.**
