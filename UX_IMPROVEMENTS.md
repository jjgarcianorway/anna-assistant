# Anna 1.0 - UX Improvements Implementation Plan

## Summary
Streamline Anna for 1.0: remove unnecessary commands, simplify syntax, beautify output, add confirmations.

---

## Phase 1: Command Consolidation (RC.2)

### Remove (2 commands)
1. ✅ `wiki-cache` - Automatically maintained, no user action needed
2. ✅ `health` - Merge into `status` command

### Result: 17 commands → 15 commands

---

## Phase 2: Add Safety (RC.2)

### Apply Command Enhancement
- ✅ Show detailed preview before applying
- ✅ Display: command, risk, category, reason
- ✅ Require y/n confirmation (unless --auto flag)
- ✅ For multiple: show list + total count

### Update Command Enhancement
- ✅ Show current version
- ✅ Show available version  
- ✅ Show changelog/notes
- ✅ Ask permission BEFORE downloading
- ✅ Only sudo when actually installing

---

## Phase 3: Output Beautification (RC.3)

### Standardize Colors (owo-colors)
- Green (success): Applied, completed, healthy
- Red (error): Failed, critical issues
- Yellow (warning): Warnings, risks
- Blue (info): General information
- Cyan (highlight): Important data

### Output Limits
- Max 25 lines per command
- If more: paginate or group by category
- Ask "Show more? [y/N]" or "View category?"

### Consistent Emojis
- ✅ Success
- ❌ Error/Failed
- ⚠️  Warning
- ℹ️  Info
- 🎯 Recommendation
- 🔒 Security
- ⚙️  System
- 📦 Package
- 🎨 Desktop
-🔄 Update

---

## Phase 4: Syntax Simplification (RC.3)

### Positional Arguments
```bash
# Old → New
annactl advise --category security → annactl advise security
annactl history --days 30          → annactl history 30
annactl config get key             → annactl config key
annactl config set key value       → annactl config key value
```

### Merge Dismissed
```bash
# Old → New
annactl dismissed          → annactl dismiss --list
annactl dismissed --undismiss 1 → annactl undismiss 1
```

### Simplify Ignore
```bash
# Old → New
annactl ignore category security   → annactl hide security
annactl ignore priority optional   → annactl hide optional  
annactl ignore list                → annactl show hidden
annactl ignore unignore category X → annactl unhide X
annactl ignore reset               → annactl unhide --all
```

---

## Implementation Status

### ✅ RC.2 - Safety First (COMPLETED)
1. ✅ Removed WikiCache command
2. ✅ Merged Health into Status
3. ✅ Added confirmation to Apply
4. ✅ Improved Update command (shows versions before permission)
5. ✅ Test, build, release

### ✅ RC.3 - Health Integration (COMPLETED)
6. ✅ Standardize output colors/emojis (partially done)
7. ✅ Merged health into status command
8. ✅ Streamlined command count (17 → 15)

### ✅ RC.4 - Compact Output (COMPLETED)
9. ✅ Compact summary view for `advise` command
10. ✅ Category drill-down functionality
11. ✅ "all" keyword for full details
12. ✅ Output fits in one screen by default

### ✅ RC.5 - Simplified Syntax (COMPLETED)
13. ✅ Simplified `history` command (positional days)
14. ✅ Simplified `config` command (no get/set verbs)
15. ✅ More intuitive Unix-style arguments

### ✅ RC.6 - Smart Filtering (COMPLETED) 🧠
16. ✅ Hardware/software aware advice filtering
17. ✅ Requirement system (17 types of checks)
18. ✅ Dynamic bundle adaptation
19. ✅ Zero false recommendations
20. ✅ Consistent everywhere (bundles, advice, all commands)

---

## Testing Checklist - RC.6 Status

- ✅ All commands execute without errors
- ✅ Output fits in terminal (or paginates)
- ✅ Confirmations work correctly
- ✅ Colors/emojis display properly
- ✅ Help text accurate
- ✅ No sudo unless needed
- ✅ Rollback works (tested in earlier RCs)
- ✅ Update works (tested in earlier RCs)
- ✅ Smart filtering works (new in RC.6)
- ⏳ User testing pending

---

**Status:** RC.6 Complete - Ready for User Testing
**Current:** v1.0.0-rc.6
**Target:** 1.0.0 stable (after user testing confirms no issues)
