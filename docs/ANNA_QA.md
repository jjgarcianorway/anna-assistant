# Anna QA Harness v0.82.0

Automated QA testing framework for Anna answers with structured JSON output.

## Overview

The QA harness allows running test scenarios against Anna to verify:
- Answer reliability (Green/Yellow/Red)
- Latency compliance (junior_ms, senior_ms)
- Dialog trace correctness (probes, draft, verdict)

## Quick Start

```bash
# Run benchmark with default settings
ANNA_QA_MODE=1 ./scripts/anna_bench.sh

# Run with custom profile and runs
ANNA_QA_MODE=1 ./scripts/anna_bench.sh --profile razorback --runs 5

# Run single test with anna_qa.sh
ANNA_QA_MODE=1 ./scripts/anna_qa.sh cpu
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANNA_QA_MODE` | Yes | Set to `1` to enable JSON output from annactl |
| `ANNA_QA_OUTPUT_DIR` | No | Directory for results (default: `/tmp/anna_qa`) |
| `ANNA_QA_QUESTION` | For `single` | Custom question for single-question tests |
| `ANNA_QA_VERBOSE` | No | Set to `1` for detailed timing output |

---

## QA JSON Schema (v0.82.0)

This is the stable schema for benchmarking and trend analysis. All fields are populated for each question run.

```json
{
  "headline": "You have 32 CPU threads available",
  "details": ["16 physical cores", "2 threads per core (SMT enabled)"],
  "evidence": ["cpu.info: CPU(s): 32, Model: AMD Ryzen 9 9950X"],
  "reliability_label": "Green",
  "score_overall": 0.95,
  "junior_ms": 1247,
  "senior_ms": 892,
  "iterations": 1,
  "probes_used": ["cpu.info"],
  "error_kind": null,
  "dialog_trace": {
    "junior_plan_probes": ["cpu.info"],
    "junior_had_draft": true,
    "senior_verdict": "approve"
  }
}
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `headline` | string | One-line direct answer |
| `details` | string[] | Bullet-point specifics |
| `evidence` | string[] | Probe-backed facts |
| `reliability_label` | string | Green, Yellow, or Red |
| `score_overall` | f64 | Overall confidence 0.0-1.0 |
| `junior_ms` | u64 | Junior LLM latency in milliseconds |
| `senior_ms` | u64 | Senior LLM latency in milliseconds |
| `iterations` | u32 | Number of LLM iterations |
| `probes_used` | string[] | List of probe IDs executed |
| `error_kind` | string? | null, "timeout", "llm_parse_error", "probe_failure", "refused" |
| `dialog_trace` | object | Internal dialog trace |

### Dialog Trace Fields

| Field | Type | Description |
|-------|------|-------------|
| `junior_plan_probes` | string[] | Probes requested by Junior |
| `junior_had_draft` | bool | Whether Junior provided draft |
| `senior_verdict` | string | approve, fix_and_accept, refuse |

### Error Kinds

| Value | Meaning |
|-------|---------|
| `null` | Success - no error |
| `timeout` | Budget exceeded before completion |
| `llm_parse_error` | Failed to parse LLM response |
| `probe_failure` | Probe execution failed |
| `refused` | Question refused (unsafe/unsupported) |

---

## Canonical Questions

Questions with stable IDs for cross-run comparison:

| ID | Question | Category |
|----|----------|----------|
| Q001 | How many CPU threads do I have? | cpu |
| Q002 | How much RAM do I have? | mem |
| Q003 | What CPU model do I have? | cpu |
| Q004 | How many physical cores do I have? | cpu |
| Q005 | How much memory is available right now? | mem |
| Q006 | Is Anna healthy? | self_health |
| Q007 | What Steam games do I have installed? | unsupported |
| Q008 | What DNS servers am I using? | unsupported |

---

## Razorback Acceptance Thresholds

Profile: **razorback** (Local Ollama with qwen2.5:7b)

### Simple Questions (cpu, mem)

| Metric | Threshold |
|--------|-----------|
| `avg_score` | >= 0.95 |
| `avg_latency_ms` | <= 25000 |
| `failures` | 0 |

### Self-Health Questions

| Metric | Threshold |
|--------|-----------|
| `avg_score` | >= 0.90 |
| `avg_latency_ms` | <= 30000 |

### Unsupported Questions

| Metric | Expected |
|--------|----------|
| `reliability_label` | Never Green |
| `error_kind` | "refused" |
| `headline` | Honest refusal (no hallucination) |

### Pass/Fail Definition

A question **passes** when:
- `score_overall >= 0.90`
- `reliability_label == "Green"`
- `error_kind == null`

A question **fails** if any of these conditions are not met.

---

## Benchmark Workflow

### Running the Benchmark

```bash
# 1. Ensure annad is running
sudo systemctl start annad

# 2. Run benchmark
ANNA_QA_MODE=1 ./scripts/anna_bench.sh --profile razorback --runs 3

# 3. Check outputs
ls -la .bench/
cat QA/ANNA_BENCH_REPORT.md
```

### Output Files

| File | Purpose |
|------|---------|
| `.bench/anna_qa_raw_<timestamp>.jsonl` | Raw per-run results |
| `.bench/anna_bench_summary_<timestamp>.json` | Aggregated statistics |
| `QA/ANNA_BENCH_REPORT.md` | Human-readable report |

### Summary JSON Structure

```json
{
  "meta": {
    "timestamp_utc": "2025-11-28T12:00:00Z",
    "profile": "razorback",
    "runs_per_question": 3,
    "anna_version": "0.82.0"
  },
  "questions": {
    "Q001": {
      "question_text": "How many CPU threads do I have?",
      "runs": 3,
      "passes": 3,
      "failures": 0,
      "avg_score": 0.95,
      "min_score": 0.93,
      "max_score": 0.97,
      "avg_latency_ms": 12500,
      "min_latency_ms": 11000,
      "max_latency_ms": 14000,
      "avg_iterations": 1.0,
      "most_common_senior_verdict": "approve",
      "most_common_probes": ["cpu.info"]
    }
  }
}
```

---

## Reliability Criteria

- **Green**: >=90% confidence, probe-backed answer
- **Yellow**: 70-89% confidence, partial evidence
- **Red**: <70% confidence, insufficient evidence

## Latency Budget (Simple Questions)

| Metric | Budget |
|--------|--------|
| `max_junior_ms_simple` | 15000 |
| `max_senior_ms_simple` | 15000 |
| `simple_question_max_iterations` | 1 |

Simple questions: single probe from {cpu.info, mem.info, hardware.ram}

---

## Analyzing Results

```bash
# Count by reliability
cat .bench/anna_qa_raw_*.jsonl | jq -s 'group_by(.reliability_label) | map({(.[0].reliability_label): length})'

# Average latency
cat .bench/anna_qa_raw_*.jsonl | jq -s 'map(.junior_ms + .senior_ms) | add / length'

# Failed tests only
cat .bench/anna_qa_raw_*.jsonl | jq -c 'select(.score_overall < 0.90 or .reliability_label != "Green")'

# Questions with errors
cat .bench/anna_qa_raw_*.jsonl | jq -c 'select(.error_kind != null)'
```

## CI Integration

```yaml
# GitHub Actions example
- name: Run Anna Benchmark
  run: |
    ANNA_QA_MODE=1 ./scripts/anna_bench.sh --runs 3
    # Check all questions pass
    jq -e '.questions | to_entries | all(.value.failures == 0)' .bench/anna_bench_summary_*.json
```
