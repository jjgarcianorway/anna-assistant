# Anna Control — Minimal Design

## Philosophy

Anna is **autonomous**. She monitors herself, heals herself, and makes decisions. Humans should rarely need to intervene.

The `annactl` command should be **minimal and beautiful**.

---

## Commands to Keep

### 1. `annactl` (default / no args)
Shows beautiful status overview.

```
╭────────────────────────────────────────────────────────────╮
│  ✅ Anna System Status — Healthy                           │
│                                                             │
│  Daemon: running  │  PID: 1234  │  Uptime: 3d 12h          │
│  RPC p99: 8 ms  │  Memory: 21.1 MB  │  Queue: 0 events    │
│  Journal: all clear                                         │
│                                                             │
│  System healthy. No action needed.                          │
╰────────────────────────────────────────────────────────────╯
```

### 2. `annactl advice`
Anna's recommendations for you.

```
╭────────────────────────────────────────────────────────────╮
│  💡 Anna's Recommendations                                  │
╰────────────────────────────────────────────────────────────╯

  1. 🔄 System update available
     → Run: sudo pacman -Syu
     → Impact: Security fixes + performance

  2. 💾 Btrfs scrub recommended
     → Last scrub: 45 days ago
     → Run: sudo btrfs scrub start /

  3. ⚡ ROG profile: Silent mode
     → Current power draw: 8W
     → Battery will last 12h

✨ Anna says: "Your system is well-maintained."
```

### 3. `annactl version`
Show version info.

```
Anna v1.0.0 — Event-Driven Intelligence
Built: 2025-11-03
Uptime: 3d 12h 47m
```

### 4. `annactl watch`
Live status updates (like `htop` but for Anna).

```
╭────────────────────────────────────────────────────────────╮
│  🌊 Anna Live Monitor     Update: 2s   Iteration: 47       │
╰────────────────────────────────────────────────────────────╯

  ✅ Daemon: active    PID: 1234    Uptime: 3d 12h 48m
  📊 RPC p99: 7ms    Memory: 21.1 MB    Queue: 0 events
  📜 Journal: 0 errors, 0 warnings

  System seems calm and healthy.

  Press Ctrl+C to exit
```

---

## Commands to Remove

Everything else. Anna handles it internally:

- `doctor` → Auto-heals when needed
- `collect`, `classify`, `radar`, `profile` → Internal telemetry
- `actions`, `audit`, `forecast`, `anomalies` → Internal decisions
- `hw`, `advisor`, `storage`, `sensors`, `net`, `disk`, `top` → Too detailed
- `events`, `export` → Debug tools (keep for developers only)
- `health`, `reload`, `config`, `learn`, `profiled`, `autonomy`, `triggers` → Internal systems

---

## Beautiful Help Output

```bash
$ annactl --help
```

```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║                 🤖  Anna Control                              ║
║            Event-Driven Intelligence CLI                      ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

Usage:  annactl [COMMAND]

Commands:
  (none)     Show system status (default)
  advice     Get Anna's recommendations
  version    Show version information
  watch      Live status monitor

Options:
  -h, --help     Show this help
  -V, --version  Show version

Examples:
  annactl              # Check status
  annactl advice       # Get recommendations
  annactl watch        # Live monitor

💙 Anna is autonomous — she handles the rest herself.
```

---

## Implementation Plan

1. **Create new minimal main.rs**
   - 4 commands total
   - Beautiful help output
   - Consistent aesthetic

2. **Keep core functionality**
   - `status_cmd.rs` (already beautified ✓)
   - Create `advice_cmd.rs` (recommendations)
   - Simple `version` display
   - Enhance `watch_mode.rs`

3. **Move debug commands to separate binary**
   - `anna-debug` for developers
   - Keeps main CLI clean

---

## User Experience

### Scenario 1: Daily Check
```bash
$ annactl
✅ Healthy. 3 recommendations available.

$ annactl advice
💡 Run: sudo pacman -Syu
💡 Btrfs scrub recommended
💡 Battery optimization available
```

### Scenario 2: Something Wrong
```bash
$ annactl
❌ Daemon not running

$ systemctl status annad
● annad.service - Anna Assistant Daemon
   Loaded: loaded
   Active: failed (Result: exit-code)

# Anna auto-repairs on next restart
$ sudo systemctl restart annad

$ annactl
✅ Healthy. Recovered from crash.
```

### Scenario 3: Live Monitoring
```bash
$ annactl watch
# Shows live updates every 2s
# Perfect for watching Anna work
```

---

## Result

- **Simple**: 4 commands, one purpose each
- **Beautiful**: Every output flows through beautiful.rs
- **Calm**: Anna's personality shines through
- **Autonomous**: Humans only see what they need

**The Perfect CLI** 🌸
