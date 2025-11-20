# Testing Guide: Beta.147 ActionPlan System

## 🚀 Complete ActionPlan Flow Testing

This guide demonstrates the full ActionPlan system including display, execution, and rollback.

---

## Prerequisites

1. Build the latest version:
   ```bash
   cargo build --release
   ```

2. Run annactl TUI:
   ```bash
   ./target/release/annactl
   ```

---

## Test Scenarios

### 🟢 **Test 1: Safe ActionPlan (Read-Only)**

**Purpose**: Test display and safe execution

**Steps**:
1. In the TUI, type: `demo`
2. Press Enter
3. **Observe**: Beautiful ActionPlan display appears with:
   - 📋 Action Plan header (Magenta)
   - Analysis section
   - Goals (numbered list)
   - Necessary Checks with ℹ️ blue indicators
   - Command Plan with ℹ️ Info risk levels
   - Notes for user
   - Max Risk: Info (Blue)

4. Press `Ctrl+X` to execute
5. **Observe**: Execution runs automatically (no confirmation for Info risk)
6. **Observe**: Results appear:
   ```
   ✅ Execution completed successfully!

   Checks passed: 1
   Steps completed: 2
   ```

**Expected Behavior**:
- ✅ Display shows all sections correctly
- ✅ Risk indicators are blue (Info)
- ✅ Commands execute without confirmation
- ✅ Both `df -h` and `df -i` run successfully
- ✅ Results displayed in conversation

---

### 🟡 **Test 2: Risky ActionPlan with Rollback**

**Purpose**: Test confirmation prompts and rollback capability

**Steps**:
1. In the TUI, type: `demo risky`
2. Press Enter
3. **Observe**: ActionPlan display with:
   - ⚠️ Medium risk indicators (Yellow)
   - Rollback Plan section showing:
     ```
     ↩ remove-test-file: Remove the test file
       $ rm -f /tmp/anna_demo_test.txt
     ```
   - Max Risk: Medium (Yellow)

4. Press `Ctrl+X` to execute
5. **Observe**: Execution asks for confirmation:
   ```
   ⚠️  This plan requires confirmation.
      Max Risk: Medium
      Steps: 2

   Execute this plan? (y/N):
   ```

6. Type `y` and press Enter (in terminal where annactl is running)
7. **Observe**: Execution proceeds:
   ```
   🚀 Executing command plan...
     1. ⚠️ Create test file with timestamp
        ✅ Success
     2. ℹ️ Display the created file content
        ✅ Success

   ✅ Action plan completed successfully!
   ```

8. **Verify**: Check that file was created:
   ```bash
   cat /tmp/anna_demo_test.txt
   # Should show: Anna Demo Test
   ```

**Expected Behavior**:
- ✅ Display shows rollback plan
- ✅ Risk indicators are yellow (Medium)
- ✅ Confirmation prompt appears
- ✅ File gets created and displayed
- ✅ Execution completes successfully

---

### 🔴 **Test 3: Rollback on Failure**

**Purpose**: Test automatic rollback when a command fails

**Steps**:
1. Create a modified risky demo that will fail
2. Execute and observe rollback

**Manual Test** (modify the demo code):
- Change one of the commands to fail (e.g., `cat /nonexistent/file.txt`)
- Execute with Ctrl+X
- **Observe**: Automatic rollback occurs:
  ```
  🔄 Command failed, initiating rollback...
    ↩ Rollback: Remove the test file
       ✅ Rollback successful
  ```

**Expected Behavior**:
- ✅ Failure detected
- ✅ Rollback steps execute in reverse order
- ✅ Test file removed automatically
- ✅ Clear error messaging

---

## Keyboard Shortcuts

While in TUI:

| Key | Action |
|-----|--------|
| `Ctrl+C` | Exit |
| `Ctrl+L` | Clear conversation |
| `Ctrl+U` | Clear input |
| **`Ctrl+X`** | **Execute last ActionPlan** ⭐ |
| `F1` | Toggle help |
| `↑ / ↓` | Navigate history |
| `PgUp / PgDn` | Scroll conversation |

---

## Demo Commands

| Command | Description | Risk Level |
|---------|-------------|-----------|
| `demo` | Safe disk space check | ℹ️ Info |
| `demo risky` | File creation with rollback | ⚠️ Medium |

---

## Visual Features to Verify

### ActionPlan Display Formatting

✅ **Header**: Magenta "📋 Action Plan"

✅ **Risk Indicators**:
- ℹ️ Blue (Info) - Safe operations
- ✅ Green (Low) - Minor changes
- ⚠️ Yellow (Medium) - Requires confirmation
- 🚨 Red (High) - Critical operations

✅ **Sections**:
- Analysis (reasoning)
- Goals (numbered list)
- Necessary Checks (with commands)
- Command Plan (with rollback links)
- Rollback Plan (undo steps)
- Notes (plain English explanation)
- Max Risk (summary indicator)

✅ **Command Display**:
```
  1. ⚠️ Create test file with timestamp
      $ echo 'Anna Demo Test' > /tmp/anna_demo_test.txt
      ↩ Rollback: remove-test-file
```

---

## Execution Output Verification

### Successful Execution
```
✅ Execution completed successfully!

Checks passed: 1
Steps completed: 2
```

### Failed Execution with Rollback
```
❌ Execution failed.

Steps failed: 1
🔄 Rollback performed (1 steps)
```

---

## Safety Verification

### Confirmation Prompts
- ℹ️ Info: No confirmation
- ✅ Low: Confirmation required
- ⚠️ Medium: Confirmation required
- 🚨 High: Confirmation required

### Rollback Behavior
1. Only completed steps are rolled back
2. Rollback executes in reverse order
3. Rollback results tracked separately
4. Clear messaging about rollback status

---

## Common Issues & Solutions

### Issue: "Ctrl+X does nothing"
**Solution**: Make sure an ActionPlan has been displayed first (type `demo`)

### Issue: "Confirmation prompt not showing in TUI"
**Solution**: Confirmation appears in the terminal where you ran annactl, not in the TUI itself

### Issue: "Commands don't execute"
**Solution**: Check that you pressed Ctrl+X (not just X)

---

## Test Checklist

Use this checklist to verify complete functionality:

- [ ] TUI launches successfully
- [ ] `demo` command generates ActionPlan
- [ ] ActionPlan displays with correct formatting
- [ ] Risk indicators show correct colors
- [ ] All sections render properly (Analysis, Goals, etc.)
- [ ] Ctrl+X triggers execution
- [ ] Safe commands execute without confirmation
- [ ] Risky commands prompt for confirmation
- [ ] Execution results display in conversation
- [ ] `demo risky` creates file successfully
- [ ] File content matches expected output
- [ ] Rollback plan section displays
- [ ] Help overlay shows Ctrl+X shortcut (F1 to view)

---

## Performance Notes

- ActionPlan display: Instant rendering
- Execution: Real-time command execution
- Rollback: Automatic on failure
- TUI responsiveness: Maintained during execution (async)

---

## Next Steps After Testing

1. ✅ Verify all tests pass
2. 🔄 Report any issues found
3. 🚀 Test with real LLM-generated ActionPlans
4. 📊 Measure execution performance
5. 🎯 Gather user feedback

---

## Developer Notes

### Code Locations
- **Display**: `crates/annactl/src/tui_v2.rs:render_action_plan_lines()`
- **Execution**: `crates/annactl/src/action_plan_executor.rs:ActionPlanExecutor`
- **Demo Commands**: `crates/annactl/src/tui_v2.rs:send_demo_action_plan()`

### Extension Points
- Add more demo scenarios
- Integrate with dialogue_v3_json for real LLM plans
- Add execution history/logging
- Implement pause/resume functionality

---

**Happy Testing!** 🎉

This is a **game-changing** feature for Anna. Enjoy exploring the ActionPlan system!
