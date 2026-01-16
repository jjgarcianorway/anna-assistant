# Phase 28: Stability Validation

Operator instructions for the stability week.

## Rules

1. **No new features.** No new commands, specialists, UX, or "smart" behavior.
2. **Daily usage.** Use Anna for real tasks, not demos.
3. **Record everything.** Outcomes, not vibes.

## Daily Protocol

Run at least 3 requests per day from different categories:

**Read-only diagnostics:**
- `annactl "what's my disk usage?"`
- `annactl "is my wifi working?"`
- `annactl "what's using memory?"`
- `annactl "any failed services?"`
- `annactl "what kernel am I running?"`

**One mutating action per day (max):**
- WiFi diagnosis with safe command execution
- Any assisted operation you're willing to approve

## Recording Outcomes

After each request, verify `outcomes.jsonl` has an entry:

```bash
tail -1 /var/lib/anna/outcomes.jsonl
```

Each entry must have:
- `question`: What you asked
- `outcome`: resolved | failed | cancelled | expired
- `probes_run`: Commands Anna executed
- `duration_ms`: How long it took

**Failure criteria:**
- You had to redo it manually → failed
- You asked Claude for help → failed
- Anna asked for clarification on a clear question → failed
- Read-only question triggered "would you like me to proceed" → failed

## Daily Verification

```bash
# Check outcomes are recording
wc -l /var/lib/anna/outcomes.jsonl

# Check stats
annactl stats

# Verify stats match outcomes
# resolved_count should match grep -c '"resolved"' outcomes.jsonl
```

## What To Watch For

**Good signs:**
- Read-only questions get direct answers
- Probes are relevant to the question
- Stats reflect actual usage

**Bad signs:**
- "Could you be more specific?" on clear questions
- Confirmation prompts on read-only requests
- Stats show fabricated success rates
- Unknown stays at 0% when it should be higher

## End Criteria

Phase 28 ends when:
1. 7 days of recorded usage
2. Stats match outcomes.jsonl
3. Read-only questions never trigger confirmation
4. One fix identified (lowest layer causing most failures)

## After The Week

1. Identify the biggest recurring failure mode
2. Fix at the lowest layer (intent classifier > probe set > plan > evidence > confirmation)
3. Bump version
4. Release with assets and SHA256SUMS
5. Document the fix in CHANGELOG
