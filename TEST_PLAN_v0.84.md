# Anna Test Plan v0.84.0

Formal test plan for reliability and performance validation.

---

## 1. Scope

### What v0.84.0 Tests Cover

| Area | Description |
|------|-------------|
| **Latency** | Total response time, Junior/Senior/command breakdown |
| **Reliability** | Confidence scores, success/failure rates |
| **Learning Loop** | Knowledge caching, reduced LLM calls over time |
| **XP System** | Correct XP awards, level progression |
| **Confidence Scoring** | Accurate probe-backed scoring |
| **Failure Modes** | Root cause classification, refusal behavior |
| **Time Budgets** | Enforcement of v0.83.0 budget limits |

### What v0.84.0 Does NOT Test

- New feature development
- UI/UX changes
- New probe types
- Architecture changes

---

## 2. Environments

### Primary Reference: Razorback

| Component | Specification |
|-----------|---------------|
| CPU | AMD Ryzen 9 5950X (32 threads) |
| RAM | 64 GB DDR4 |
| GPU | NVIDIA RTX (for LLM acceleration) |
| Storage | NVMe SSD |
| OS | Arch Linux |
| LLM Backend | Ollama with qwen3:4b (Junior), qwen3:8b (Senior) |

### Target Performance on Razorback

| Metric | Target |
|--------|--------|
| Simple question latency | < 15 seconds |
| Hardware probe latency | < 10 seconds |
| Repeated question latency | < 5 seconds (cached) |
| Confidence for probe-backed answers | >= 90% |

### Other Environments

- Weaker hardware profiles exist but are not primary targets for v0.84.0
- Tests should pass on razorback first before optimization for other machines

---

## 3. Test Suites

### 3.1 Smoke Tests

Quick sanity checks that Anna is operational.

| Test ID | Question | Expected Behavior |
|---------|----------|-------------------|
| SMOKE-01 | "hello" | Greeting response, no crash |
| SMOKE-02 | "version" | Version info displayed |
| SMOKE-03 | "status" | Status output with daemon health |

### 3.2 Core Hardware Questions

| Test ID | Question | Expected Answer Contains | Probes Used |
|---------|----------|-------------------------|-------------|
| HW-01 | "how many cores has my computer?" | "16 physical cores" or "32 threads" | cpu.info |
| HW-02 | "what CPU model do I have?" | "AMD Ryzen" or "Intel" | cpu.info |
| HW-03 | "how much RAM is installed?" | Memory in GB | mem.info |
| HW-04 | "how much free RAM do I have?" | Memory in GB/MB | mem.info |
| HW-05 | "what GPU do I have?" | GPU model or "none" | hardware.gpu |
| HW-06 | "how much disk space is free?" | Size in GB/TB | disk.lsblk |

### 3.3 System Logs and Journal

| Test ID | Question | Expected Behavior |
|---------|----------|-------------------|
| LOG-01 | "show me the logs for annad service" | Journal output or refusal with reason |
| LOG-02 | "are there any errors in system logs?" | Summary or refusal with reason |

### 3.4 Updates and Unsupported Domains

| Test ID | Question | Expected Behavior |
|---------|----------|-------------------|
| UNS-01 | "are there system updates pending?" | Clear statement about no probe |
| UNS-02 | "is my wifi stable?" | Clear refusal: no network probe |
| UNS-03 | "is Steam installed?" | Clear refusal: no package probe |
| UNS-04 | "what is the weather?" | Clear refusal: out of scope |

### 3.5 Self-Health Diagnostics

| Test ID | Question | Expected Behavior |
|---------|----------|-------------------|
| HEALTH-01 | "diagnose your own health" | Status of daemon, models, tools |
| HEALTH-02 | "are you working correctly?" | Self-assessment with evidence |

### 3.6 Learning Loop / Reinforcement

Run same questions multiple times and verify improvement.

| Test ID | Scenario | Verification |
|---------|----------|--------------|
| LEARN-01 | Ask "how many CPU cores?" 3 times | Later runs should have fewer LLM calls |
| LEARN-02 | Ask "how much RAM?" 3 times | Latency should decrease on repeats |
| LEARN-03 | Verify XP increases after successful answers | Check `annactl status` |

---

## 4. Metrics

### Per-Question Metrics

| Metric | Description | Source |
|--------|-------------|--------|
| `total_ms` | Total answer time | Orchestrator |
| `self_ms` | Self-solve attempt time | Orchestrator |
| `junior_ms` | Junior LLM time | LLM client |
| `senior_ms` | Senior LLM time | LLM client |
| `cmd_ms` | Command execution time | Probe runner |
| `confidence` | Final confidence score (0.0-1.0) | Answer engine |
| `junior_calls` | Number of Junior invocations | LLM client |
| `senior_calls` | Number of Senior invocations | LLM client |
| `probes_used` | List of probes executed | Probe runner |
| `success` | Whether answer was correct | Benchmark script |
| `xp_delta` | XP change from this answer | Progression system |

### Aggregate Metrics (24-hour window)

| Metric | Description |
|--------|-------------|
| Questions answered | Total count |
| Success rate | % with confidence >= 70% |
| Average confidence | Mean confidence score |
| Average latency | Mean total_ms |
| XP gained | Sum of positive deltas |
| XP lost | Sum of negative deltas |
| Net XP | Gained - Lost |

---

## 5. Pass/Fail Criteria

### Hard Targets for v0.84.0 on Razorback

| Criterion | Target | Failure |
|-----------|--------|---------|
| Simple hardware question latency | < 15s | > 20s |
| Cached question latency | < 5s | > 10s |
| Probe-backed confidence | >= 90% | < 70% |
| Unsupported domain refusal | Clear "no probe" message | Fabricated answer |
| Learning effect | Latency decreases on repeat | No improvement |
| Budget violations | < 10% of questions | > 25% |
| Crash rate | 0% | Any crash |

### Acceptable Degradation

- First-time questions may be slower (cold cache)
- Complex multi-probe questions may exceed 15s
- Senior escalation adds latency (acceptable for verification)

---

## 6. How to Run the Tests

### 6.1 Single Question Benchmark

```bash
# Run with benchmark mode enabled
ANNA_BENCH_MODE=1 annactl "how many cores has my computer?"
```

### 6.2 Full Benchmark Suite

```bash
# Run the complete benchmark script
bash scripts/anna-bench-v0.84.sh
```

### 6.3 Inspect XP and Progression

```bash
# View current status with 24h metrics
annactl status

# View recent XP events
annactl xp-log --limit 20
```

### 6.4 Check Metrics in Logs

```bash
# View per-question metrics
journalctl -u annad -b | grep ANNA_METRICS

# View budget violations
journalctl -u annad -b | grep ANNA_WARN

# View failure causes
journalctl -u annad -b | grep failure_cause
```

### 6.5 View Benchmark Results

```bash
# Benchmark results are stored in JSON
ls -la /var/log/anna/bench_v0.84/

# View latest benchmark
cat /var/log/anna/bench_v0.84/benchmark-*.json | jq .
```

---

## 7. Failure Cause Classification

When answers fail or have low confidence, they are tagged with a root cause:

| Cause | Description |
|-------|-------------|
| `no_probe_available` | Question requires a probe that doesn't exist |
| `probe_data_misread` | Probe ran but data was misinterpreted |
| `llm_hallucination` | LLM fabricated information not in evidence |
| `timeout_or_latency` | Exceeded time budget |
| `unsupported_domain` | Question is outside Anna's scope |
| `orchestration_bug` | Internal error in answer pipeline |
| `bad_command_proposal` | Junior proposed unsafe or invalid command |

---

## 8. v0.85 Architecture Tests

v0.85.0 introduces the Brain layer for self-sufficient answers. These tests validate the new architecture.

### 8.1 Simple Recall Tests

| Test ID | Procedure | Expected Result |
|---------|-----------|-----------------|
| ARCH-01 | Ask "how many CPU cores?" 5 times consecutively | First call uses Junior+Senior, later calls skip LLM and use Brain only |
| ARCH-02 | Check debug output for `[ANNA_BRAIN]` entries | Brain summary shows pattern_match and decision |
| ARCH-03 | Measure latency improvement on repeat | 3rd+ call should be <2s |

### 8.2 Command Refinement Tests

| Test ID | Procedure | Expected Result |
|---------|-----------|-----------------|
| ARCH-04 | Force JSON parse error (mock lscpu -J failure) | Brain tries fallback to plain lscpu |
| ARCH-05 | Check COMMAND_LIBRARY after successful answer | Pattern is stored with reliability score |
| ARCH-06 | Check FAILURE_MEMORY after bad command | Command is recorded to avoid repeat |

### 8.3 Timeout Simulation

| Test ID | Procedure | Expected Result |
|---------|-----------|-----------------|
| ARCH-07 | Set ANNA_SIMULATE_LLM_SLOW=1 | Fallback to Brain with failure_cause=llm_timeout |
| ARCH-08 | Check error message | "LLM timeout, here is the best evidence I can provide" |

### 8.4 Reinforcement Stability

| Test ID | Procedure | Expected Result |
|---------|-----------|-----------------|
| ARCH-09 | Generate BrainSelfSolve event | XP increases by +15 |
| ARCH-10 | Generate JuniorBadCommand event | XP decreases by -8 |
| ARCH-11 | Generate SeniorGreenApproval event | XP increases by +12 |

### 8.5 Prompt Size Validation

| Test ID | Check | Threshold |
|---------|-------|-----------|
| ARCH-12 | Junior system prompt size | < 2 KB |
| ARCH-13 | Senior system prompt size | < 4 KB |

### 8.6 Performance Budget Enforcement

| Test ID | Check | Threshold |
|---------|-------|-----------|
| ARCH-14 | Total question time | < 12s |
| ARCH-15 | Junior LLM time | < 3s |
| ARCH-16 | Senior LLM time | < 4s |
| ARCH-17 | LLM calls for simple question | <= 1 |

### 8.7 Reliability Thresholds

| Test ID | Check | Threshold |
|---------|-------|-----------|
| ARCH-18 | Evidence score | >= 80% |
| ARCH-19 | Coverage score | >= 90% |
| ARCH-20 | Reasoning score | >= 90% |

---

## 9. Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.85.0 | 2025-11-28 | Added v0.85 Architecture Tests (Brain, performance, XP) |
| v0.84.0 | 2025-11-28 | Initial test plan creation |
