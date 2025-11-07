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

## Implementation Order

### RC.2 (Immediate)
1. Remove WikiCache command
2. Merge Health into Status
3. Add confirmation to Apply
4. Improve Update command (show versions)
5. Test, build, release

### RC.3 (Next)
6. Standardize output colors/emojis
7. Add pagination for long outputs
8. Simplify command syntax
9. Update all help text
10. Test, build, release

### RC.4 (Polish)
11. Beautify installer script
12. Final testing of all commands
13. Documentation updates
14. Prepare for 1.0.0 stable

---

## Testing Checklist (Per RC)

- [ ] All commands execute without errors
- [ ] Output fits in terminal (or paginates)
- [ ] Confirmations work correctly
- [ ] Colors/emojis display properly
- [ ] Help text accurate
- [ ] No sudo unless needed
- [ ] Rollback works
- [ ] Update works

---

**Status:** Implementation started
**Current:** RC.2 preparation
**Target:** 1.0.0 stable
