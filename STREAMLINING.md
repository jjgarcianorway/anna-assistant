# Anna 1.0 - UX Streamlining Plan

## Goals
1. Output fits in one terminal screen (24-30 lines max)
2. Simple command syntax - no complex flags
3. Beautiful, consistent output everywhere
4. Always inform before acting
5. Sudo only when needed
6. Easy to use and debug

---

## Command Consolidation

### KEEP (Core - 10 commands)
1. `status` - System status + health (MERGE health into this)
2. `advise [category]` - Recommendations (simplify syntax)
3. `apply <id>` - Apply with confirmation
4. `bundles` - List bundles
5. `rollback <id>` - Undo changes
6. `doctor` - Diagnostics + report (MERGE report into this)
7. `config <key> [value]` - Settings (simplify)
8. `history [days]` - Audit log (simplify)
9. `update` - Self-update with version info
10. `completions <shell>` - Shell completions

### REMOVE (3 commands)
- `wiki-cache` - Make automatic, remove from CLI
- `health` - Merge into `status`
- `report` - Merge into `doctor`

### SIMPLIFY (4 commands)
- `dismiss <id>` - Dismiss advice
- `show dismissed` - NEW: Show dismissed (replaces `dismissed`)
- `show hidden` - NEW: Show ignored (replaces `ignore list`)
- `hide <what>` - NEW: Hide category/priority (replaces `ignore`)

---

## New Simple Syntax

### Before → After

```bash
# Status and health
annactl health                    → annactl status
annactl status                    → annactl status (unchanged)

# Advice
annactl advise --category security → annactl advise security
annactl advise --priority critical → annactl advise critical
annactl advise                     → annactl advise (unchanged)

# Dismissing
annactl dismiss 5                  → annactl dismiss 5 (unchanged)
annactl dismissed                  → annactl show dismissed
annactl dismissed --undismiss      → annactl undismiss

# Hiding/Ignoring
annactl ignore category security   → annactl hide security
annactl ignore priority optional   → annactl hide optional
annactl ignore list                → annactl show hidden
annactl ignore reset               → annactl unhide all

# Config
annactl config get autonomy_limit  → annactl config autonomy_limit
annactl config set autonomy_limit 2 → annactl config autonomy_limit 2
annactl autonomy 2                 → annactl config autonomy 2

# Doctor/Report
annactl report                     → annactl doctor
annactl report --category disk     → annactl doctor disk
annactl doctor --fix               → annactl doctor --fix (unchanged)

# History
annactl history --days 30          → annactl history 30
annactl history --detailed         → annactl history --verbose
```

---

## Output Standards

### Use owo-colors consistently:
- ✅ Green for success
- ❌ Red for errors
- ⚠️  Yellow for warnings
- ℹ️  Blue for info
- 🎯 Cyan for highlights

### Format:
```
# Good - fits in one screen
annactl advise security

🔒 Security Recommendations (3)

  1. [CRITICAL] Install microcode updates
     → Fixes: CPU security vulnerabilities
     
  2. [RECOMMENDED] Enable firewall
     → Why: Protects against network attacks
     
  3. [OPTIONAL] Install AppArmor
     → Info: Additional security layer

Apply? [1-3, all, none]: _
```

### Bad - too much output:
```
annactl advise

Showing 120 recommendations...
(scrolls for pages)
```

---

## Confirmation Prompts

### Apply commands MUST show preview:

```bash
annactl apply 5

📋 Preview: Install Firefox

Command: pacman -S --noconfirm firefox
Risk: Low
Category: Applications
Why: Modern web browser recommended

Proceed? [y/N]: _
```

### Apply multiple:
```bash
annactl apply 1-5

📋 Applying 5 recommendations:
  1. Install microcode
  2. Enable firewall  
  3. Update system
  4. Install vim
  5. Configure git

Total commands: 5
Estimated time: 2 minutes

Proceed? [y/N]: _
```

---

## Update Improvements

### Before (ugly):
```
Checking for updates...
[downloads immediately, asks for sudo]
```

### After (informative):
```
🔄 Checking for updates...

Current:   1.0.0-rc.1
Available: 1.0.0-rc.2

Changes:
  • Fixed apply confirmation bug
  • Improved output formatting
  • Better error messages

Download and install? [y/N]: _

[only asks for sudo if user says yes and needs it]
```

---

## Pagination

### When output > 30 lines:

```
[Shows first 25 lines]

... 95 more recommendations

Show more? [y/N/all]: _
```

Or group intelligently:
```
🔒 Security (12)  ⚙️  System (45)  🎨 Desktop (8)  📦 Apps (30)

View category [security/system/desktop/apps/all]: _
```

---

## Implementation Priority

1. ✅ Command consolidation (remove/merge)
2. ✅ Simplify syntax (positional args)
3. ✅ Standardize output (owo-colors everywhere)
4. ✅ Add confirmations to apply
5. ✅ Improve update flow
6. ✅ Add pagination where needed
7. ✅ Test all commands
8. ✅ Update docs
9. ✅ Release rc.2
