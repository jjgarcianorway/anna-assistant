# Reliability-First Development Workflow

**Core Principle**: Every change must maintain or improve reliability. Speed is secondary.

---

## Current Baseline (v0.3.169)

**Overall Reliability**: **77%**

### Category Breakdown:
- **Accuracy**: 80% (4/5) ⚠️
  - ✓ Disk usage detection
  - ✓ RAM usage detection
  - ✓ Service status
  - ✗ User context (answers as root, not actual user)
  - ✓ Hostname detection

- **Completeness**: 100% (1/1) ✓
  - ✓ System status includes multiple metrics

- **Consistency**: 100% (1/1) ✓
  - ✓ Same question = same answer

- **Safety**: 100% (1/1) ✓
  - ✓ No dangerous commands suggested

- **Speed**: 0% (0/1) ✗
  - ✗ Simple questions: 6-11 seconds (target: <5s)
  - Issue: Even "What is my hostname?" takes 6+ seconds

### Identified Issues:
1. **User Context Problem**: Anna answers from root perspective instead of actual user
2. **Speed Problem**: All questions too slow, even trivial ones
3. **No Instant Answers**: Everything goes through full LLM pipeline

---

## Mandatory Testing Workflow

### Before ANY Code Change:

```bash
# 1. Run baseline test
./tests/reliability_framework.sh

# 2. Save current score
echo "Baseline: X%" > change_baseline.txt
```

### After Code Change:

```bash
# 1. Rebuild
cargo build --release --workspace

# 2. Restart Anna
sudo systemctl restart annad
# OR
sudo killall annad && sudo /usr/local/bin/annad &

# 3. Wait for startup
sleep 5

# 4. Run reliability test
./tests/reliability_framework.sh

# 5. Compare with baseline
# If reliability decreased → REVERT immediately
# If reliability same/better → Commit
```

### Comparison Test (Weekly):

```bash
# Run Anna vs Claude comparison
./tests/anna_vs_claude_comparison.sh

# Target: Anna ≥90% accuracy, <15s average response
```

---

## Change Approval Criteria

### ✅ APPROVED if:
1. Reliability ≥77% (baseline)
2. No regression in any category
3. Speed improvements don't sacrifice accuracy

### ⚠️  CAUTION if:
1. Reliability = baseline but different category mix
2. Speed improved but accuracy decreased slightly
3. New features pass but existing features regress

### ✗ REJECT if:
1. Reliability <77%
2. Any category drops >10%
3. New safety issues introduced
4. Accuracy <75%

---

## Incremental Improvement Strategy

### Phase 1: Fix Critical Issues (Reliability Target: 90%)

#### Change 1.1: Fix User Context
**Problem**: Answers from root perspective
**Solution**: Detect actual user from environment/commands
**Risk**: Low
**Test**: "What user am I?" must return actual user

#### Change 1.2: Add Instant Answers for Simple Questions
**Problem**: "hostname" takes 6 seconds
**Solution**: Pattern matching for trivial questions
**Risk**: Medium (could break for edge cases)
**Test**: 10 simple questions must be <1s each

#### Change 1.3: Smart Model Routing
**Problem**: Using 14b model for everything
**Solution**: Use 3b/7b for simple questions
**Risk**: Medium (accuracy might drop)
**Test**: Accuracy must stay ≥80%

### Phase 2: Speed Optimization (Reliability Target: 85%+)

#### Change 2.1: Command Caching
**Problem**: Running same commands repeatedly
**Solution**: Cache command results (TTL-based)
**Risk**: Low (stale data possible)
**Test**: Consistency must stay 100%

#### Change 2.2: Parallel Execution
**Problem**: Sequential command execution
**Solution**: Run independent commands in parallel
**Risk**: Medium (race conditions)
**Test**: All accuracy tests must pass

#### Change 2.3: Response Streaming
**Problem**: Wait for full answer before showing
**Solution**: Stream tokens as generated
**Risk**: Low (UX improvement)
**Test**: Complete answers must match

### Phase 3: Intelligence Enhancement (Reliability Target: 95%+)

#### Change 3.1: Recipe Learning
**Solution**: Auto-create recipes from patterns
**Risk**: Medium (wrong recipes = wrong answers)
**Test**: Accuracy must improve or stay same

#### Change 3.2: Memory System
**Solution**: Session continuity
**Risk**: Low
**Test**: Consistency and context awareness

#### Change 3.3: Multi-Perspective Integration
**Solution**: Better module orchestration
**Risk**: Low (already implemented)
**Test**: Complex questions must be more accurate

---

## Testing Checklist

### Before Commit:
- [ ] Reliability test passed (≥77%)
- [ ] No accuracy regression
- [ ] No safety regression
- [ ] Change documented in commit message
- [ ] Rollback plan documented

### Before Release:
- [ ] All tests passed
- [ ] Comparison test passed (vs Claude)
- [ ] Manual testing on complex questions
- [ ] CHANGELOG updated
- [ ] Reliability score documented

### After Release:
- [ ] Monitor for 24 hours
- [ ] Check user reports
- [ ] Run weekly comparison test
- [ ] Update baseline if improved

---

## Rollback Procedure

If reliability drops after a change:

```bash
# 1. Revert commit
git revert HEAD

# 2. Rebuild
cargo build --release --workspace

# 3. Redeploy
# (copy binaries or restart service)

# 4. Verify baseline restored
./tests/reliability_framework.sh

# 5. Investigate why change failed
# 6. Fix and re-test before re-applying
```

---

## Current Priority Queue

### Immediate (This Week):
1. **Fix User Context** (Accuracy: 80% → 100%)
   - Target: All 5 accuracy tests pass
   - Risk: Low
   - Testing: Extensive

2. **Add Instant Answer Paths** (Speed: 0% → 60%+)
   - Target: Simple questions <1s
   - Risk: Medium
   - Testing: Compare all instant vs LLM answers

3. **Smart Model Routing** (Speed: 6-11s → 3-5s average)
   - Target: 3b/7b for simple, 14b for complex
   - Risk: Medium
   - Testing: Accuracy must not regress

### This Month:
4. **Command Caching** (Speed improvement)
5. **Recipe Learning** (Accuracy improvement)
6. **Parallel Execution** (Speed improvement)
7. **Memory System** (Intelligence improvement)

### Long Term:
- All 30 improvements from NEXT_IMPROVEMENTS.md
- But only after proven reliable at each step

---

## Success Metrics

### Week 1 Target:
- Reliability: 77% → 90%
- Accuracy: 80% → 100%
- Speed: Simple questions <5s

### Month 1 Target:
- Reliability: 90% → 95%
- Speed: Average <5s
- Recipe learning active

### Month 3 Target:
- Reliability: 95% → 98%
- Speed: Average <3s
- All 30 improvements shipped
- Anna vs Claude: Competitive

---

## Baseline Archive

All baseline tests saved in `~/anna_test_baselines/`:

- `baseline_v0.3.169_initial.json` - Current baseline (77%)
- Future baselines will be added as improvements land

---

## Test Execution

### Quick Test (5 minutes):
```bash
./tests/reliability_framework.sh
```

### Full Test (15 minutes):
```bash
./tests/anna_vs_claude_comparison.sh
```

### Continuous Testing (Daily):
```bash
# Add to cron:
0 3 * * * /path/to/tests/reliability_framework.sh > /tmp/daily_reliability.log 2>&1
```

---

## Notes

- **Never sacrifice reliability for speed**
- **Always test before commit**
- **Document why changes improve things**
- **Rollback immediately if reliability drops**
- **Baseline must be updated only when improved**

---

**Current Status**: ✅ Baseline established, testing framework active
**Next Action**: Implement Change 1.1 (Fix User Context)
**Goal**: Reliability 77% → 90% by end of week
