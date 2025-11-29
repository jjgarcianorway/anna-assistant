# Anna Benchmark Protocol v1.3.0

A before/after test protocol for measuring Anna's performance from a clean state.

## Overview

This protocol allows you to:
1. Reset Anna's learned experience (XP, telemetry, stats)
2. Run a fixed set of test questions
3. Measure success rate, latency, and reliability
4. Compare results across runs

## Prerequisites

- Anna daemon running (`sudo systemctl status annad`)
- Ollama running with models available (`ollama list`)
- Fresh build of annactl and annad

## Two Reset Modes (v1.3.0)

Anna supports two distinct reset modes:

### Experience Reset (Soft Reset)

Resets XP, trust, counters to baseline values. **Preserves knowledge.**

- XP store reset to level 1, trust 0.5, XP 0
- Telemetry cleared
- Stats directory cleared
- Knowledge base: **PRESERVED**

**Trigger phrases:**
- "reset your experience"
- "reset your xp"
- "clear your telemetry"
- "wipe your learning history"
- "start fresh"

**Confirmation:** Type `yes`

### Factory Reset (Hard Reset)

Deletes ALL learned data including knowledge. **This is irreversible.**

- XP store reset to baseline
- Telemetry cleared
- Stats directory cleared
- Knowledge base: **DELETED**

**Trigger phrases:**
- "factory reset"
- "delete everything"
- "wipe all your data"
- "full reset"
- "delete knowledge"

**Confirmation:** Type exactly `I UNDERSTAND AND CONFIRM FACTORY RESET`

## Protocol Steps

### STEP 0: Verify Healthy State

```bash
annactl status
```

Confirm:
- annad is running
- Ollama is running
- Models are available
- Self-health shows healthy

### STEP 1: Snapshot Pre-Reset State

Run `annactl status` and record:

| Metric | Value |
|--------|-------|
| Lifetime Questions | |
| Lifetime Success Rate | |
| Recent (last 100) Success | |
| Recent Avg Latency (ms) | |
| Brain Questions | |
| Brain Success Rate | |
| Brain Avg Latency (ms) | |
| Junior Questions | |
| Junior Success Rate | |
| Senior Questions | |
| Anna XP Level | |
| Anna Total XP | |

### STEP 2: Reset Experience

Ask Anna to reset her experience (soft reset):

```bash
annactl
anna> reset your experience
```

Anna will show current state and ask for confirmation:
```
This will reset XP, trust, and counters to baseline (level 1, trust 0.5).
Telemetry and stats will be cleared. Knowledge is preserved.

Current state:
- Anna level: 5, XP: 1234
- Questions answered: 100
- Telemetry events: 150

Type 'yes' to confirm.
```

Type `yes` to confirm.

Verify the reset:
```bash
annactl status
```

You should see:
- "No telemetry yet. Ask a few questions to gather data."
- XP metrics showing level 1, XP 0, trust 0.50
- Progression showing fresh state

### STEP 3: Run Test Question Set

Run through these 10 canonical questions, one at a time:

```bash
annactl
```

1. **CPU Model**
   ```
   what CPU model do I have, how many physical cores and threads?
   ```

2. **RAM Capacity**
   ```
   how much RAM is installed and how much is free?
   ```

3. **Disk Space**
   ```
   how much free space do I have on my root filesystem and how big is it?
   ```

4. **Self Health**
   ```
   diagnose your own health (daemon, permissions, models, tools)
   ```

5. **Anna Logs**
   ```
   show me annad logs for the last 6 hours and highlight important errors/warnings
   ```

6. **System Updates**
   ```
   are there any pending system updates and propose a safe review/update plan
   ```

7. **GPU Info**
   ```
   what GPU(s) are present?
   ```

8. **OS Info**
   ```
   what OS and kernel am I running?
   ```

9. **Uptime**
   ```
   how long has this machine been up?
   ```

10. **Network Interfaces**
    ```
    list my network interfaces and basic info
    ```

For each question, note:
- Origin (Brain/Junior+Senior)
- Reliability score
- Duration (seconds)
- Whether answer was correct/useful

### STEP 4: Snapshot Post-Run Metrics

Run `annactl status` and record:

| Metric | Value |
|--------|-------|
| Questions (should be ~10) | |
| Success Rate | |
| Avg Latency (ms) | |
| Brain Questions | |
| Brain Success Rate | |
| Brain Avg Latency (ms) | |
| Junior Questions | |
| Senior Questions | |
| XP Gained | |
| Current XP Level | |

### STEP 5: Analysis

Compare the results:

**Expected Behavior:**
- Questions count increased by ~10
- Brain fast path should handle: CPU, RAM, disk, health, uptime
- LLM path should handle: logs, updates, GPU, OS, network
- Brain answers should be <100ms
- LLM answers should be <15s
- Success rate should reflect actual answer quality
- XP should increase (Brain self_solve_ok = +5 XP each)

**Red Flags:**
- Questions count doesn't match runs
- Success rate shows 0% with actual successful answers
- Telemetry shows "No telemetry" after running questions
- XP didn't increase after successful answers

## Factory Reset (For Complete Fresh Start)

If you need to completely wipe Anna's state (including knowledge):

```bash
annactl
anna> factory reset
```

⚠️ **WARNING**: This will show a confirmation prompt:
```
⚠️  FACTORY RESET WARNING ⚠️

This will delete ALL learned data including:
- XP, levels, trust, streaks (reset to baseline)
- Telemetry history (150 events)
- Stats and learning artifacts
- Knowledge base and learned facts (5 files)

Current state: Level 5, 1234 XP, 100 questions answered

This is IRREVERSIBLE. To confirm, type exactly:
I UNDERSTAND AND CONFIRM FACTORY RESET
```

Type the exact phrase to confirm.

## Automated Benchmark (Future)

A future version may include:
```bash
annactl benchmark --reset --questions 10 --passes 3
```

This would:
1. Reset experience
2. Run N questions M times
3. Output JSON report with all metrics

## Troubleshooting

### Reset not working

Check file permissions:
```bash
ls -la /var/lib/anna/xp/
ls -la /var/log/anna/
ls -la /var/lib/anna/knowledge/stats/
```

If permission denied, fix with:
```bash
sudo chown -R anna:anna /var/lib/anna /var/log/anna
```

### Telemetry not recording

Ensure directories exist:
```bash
sudo mkdir -p /var/lib/anna/xp
sudo mkdir -p /var/lib/anna/knowledge/stats
sudo mkdir -p /var/log/anna
sudo chown -R anna:anna /var/lib/anna /var/log/anna
```

### XP not updating

The XP store saves to `/var/lib/anna/xp/xp_store.json`. Check:
```bash
cat /var/lib/anna/xp/xp_store.json
```

If empty or missing, the directory may not be writable.

### After Reset, Status Shows No Data

This is expected! After a reset:
- "No telemetry yet" is correct (telemetry was cleared)
- Level 1, XP 0, trust 0.50 is the baseline state
- Ask a few questions to start gathering new metrics

## Version History

- v1.3.0: Two reset modes (Experience vs Factory), strong confirmation for factory reset
- v1.2.0: Initial benchmark protocol with experience reset
