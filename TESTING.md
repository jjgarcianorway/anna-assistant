# Anna Assistant 1.0 - Testing Checklist

This document tracks testing requirements before releasing 1.0 stable.

## Testing Strategy

**Goal:** Ensure all CLI commands work correctly and reliably.
**Scope:** Focus on `annactl` command-line interface only (TUI disabled for 1.0).
**Release Candidate Process:** Fix bugs → rc.2 → test → fix → rc.3 → repeat until stable.

---

## Core Command Testing

### System Information
- [ ] `annactl status` - Shows daemon status, version, uptime
- [ ] `annactl health` - Reports system health metrics
- [ ] `annactl doctor` - Detects and reports common issues
- [ ] `annactl doctor --fix` - Attempts to fix detected issues
- [ ] `annactl doctor --dry-run` - Shows what would be fixed without doing it

### Advice Management
- [ ] `annactl advise` - Lists all recommendations
- [ ] `annactl advise --category <cat>` - Filters by category
- [ ] `annactl advise --priority <pri>` - Filters by priority
- [ ] `annactl apply <number>` - Applies single advice by number
- [ ] `annactl apply <id>` - Applies single advice by ID
- [ ] `annactl apply <number> --dry-run` - Shows what would be executed
- [ ] `annactl apply-all` - Applies all Critical advice
- [ ] `annactl apply-all --priority Recommended` - Applies all Recommended
- [ ] `annactl apply-bundle <name>` - Applies complete bundle

### Filtering & Ignoring
- [ ] `annactl ignore list` - Shows current filters
- [ ] `annactl ignore category <name>` - Hides category
- [ ] `annactl ignore priority <level>` - Hides priority
- [ ] `annactl ignore unignore category <name>` - Removes category filter
- [ ] `annactl ignore unignore priority <level>` - Removes priority filter
- [ ] `annactl ignore reset` - Clears all filters

### Dismissal System
- [ ] `annactl dismiss <number>` - Dismisses advice by number
- [ ] `annactl dismissed` - Lists dismissed advice
- [ ] `annactl dismissed --undismiss` - Undismisses all

### History & Auditing
- [ ] `annactl history` - Shows recent actions (7 days)
- [ ] `annactl history --days 30` - Shows 30 days of history
- [ ] `annactl history --detailed` - Shows detailed audit log

### Configuration Management
- [ ] `annactl config list` - Shows all configuration
- [ ] `annactl config get <key>` - Gets specific setting
- [ ] `annactl config set <key> <value>` - Sets configuration
- [ ] `annactl autonomy <0-3>` - Sets autonomy level
- [ ] Verify config persists across restarts

### Bundle System
- [ ] Verify Hyprland bundle detects installed Hyprland
- [ ] Verify bundle only shows if WM already installed
- [ ] Verify multi-step bundles execute in order
- [ ] Verify bundle respects hardware detection (laptop vs desktop)
- [ ] Test bundle rollback: \`annactl rollback bundle <name>\`
- [ ] Test bundle rollback dry-run

### Update System
- [ ] \`annactl update --check\` - Checks for updates without installing
- [ ] \`annactl update --install\` - Installs available update
- [ ] Verify GitHub release detection
- [ ] Verify binary download and replacement
- [ ] Verify update notifications

### Reporting
- [ ] \`annactl report\` - Generates full system report
- [ ] \`annactl report --category <cat>\` - Filtered report
- [ ] Verify report includes all relevant diagnostics

### Shell Completions
- [ ] \`annactl completions bash\` - Generates bash completions
- [ ] \`annactl completions zsh\` - Generates zsh completions
- [ ] \`annactl completions fish\` - Generates fish completions
- [ ] Install and verify completions work

---

## Known Issues / Won't Fix for 1.0

### TUI Disabled
- Interactive TUI disabled for 1.0 release
- Will be re-enabled in 2.0 with better UX
- All functionality available via CLI

---

## Release Criteria

Before releasing 1.0.0 stable, ALL core commands must work correctly.

---

**Last Updated:** 2025-11-06
**Current Version:** 1.0.0-rc.1
**Testing Status:** Not Started
