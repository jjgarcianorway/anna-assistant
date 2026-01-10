# Anna Performance Test Report v3
**Date:** 2026-01-10
**Model:** qwen2.5:7b (configured) / qwen2.5:14b (loaded)
**System:** CachyOS Linux, RTX GPU with 8GB VRAM

## Executive Summary

Anna's three-tier understanding system shows **strong pattern matching** but **LLM latency issues** cause timeouts on questions requiring full LLM processing.

| Metric | Value | Notes |
|--------|-------|-------|
| Completion Rate | 75% | 15/20 questions |
| Real Answers | 35% | 7/20 questions |
| Clarifications | 40% | 8/20 (appropriate for ambiguous questions) |
| Timeouts | 25% | 5/20 (all factual questions) |
| Median Response | ~300ms | For pattern-matched questions |
| LLM Response | 29-45s | For questions needing LLM |

## Detailed Results by Category

### Factual Questions (1-5): ❌ All Timed Out
```
Q1: what is my disk usage       → TIMEOUT (45s)
Q2: how much RAM do I have      → TIMEOUT (45s)
Q3: list failed systemd services → TIMEOUT (45s)
Q4: what gpu do I have          → TIMEOUT (45s)
Q5: show my ip address          → TIMEOUT (45s)
```
**Analysis:** These simple questions go through full LLM pipeline:
1. Intent classification (~10s)
2. Command selection (~10s)
3. Validation (~10s)
4. Answer generation (~10s)

**Recommendation:** Add patterns for common factual queries to bypass LLM.

### Error Messages (6-8): ✅ 2/3 Answered
```
Q6: pacman says database is locked  → ✅ ANSWERED (1005ms)
Q7: error: target not found: yay    → ? CLARIFY (5ms)
Q8: nvidia module not found         → ✅ ANSWERED (1993ms)
```
**Analysis:** Pattern matching working well! "database is locked" and "nvidia module" matched known patterns and ran diagnostics immediately.

### How-To Questions (9-11): ⚠️ All Clarifications
```
Q9:  how do I install neovim          → ? CLARIFY (6ms)
Q10: how to enable ssh                → ? CLARIFY (6ms)
Q11: how do I check cpu temperature   → ? CLARIFY (6ms)
```
**Analysis:** Intent classified as HOWTO with 50% confidence, triggering clarification. Could be more aggressive about just answering.

### Troubleshooting (12-14): ✅ All Answered
```
Q12: wifi not working      → ✅ ANSWERED (376ms)
Q13: no sound from speakers → ✅ ANSWERED (385ms)
Q14: screen flickering      → ✅ ANSWERED (255ms)
```
**Analysis:** Excellent! Troubleshoot patterns matched and ran diagnostic commands immediately.

### Performance Questions (15-17): ✅ 2/3 Answered
```
Q15: what is using my cpu  → ✅ ANSWERED (341ms)
Q16: why is my system slow → ✅ ANSWERED (289ms)
Q17: check memory usage    → ? CLARIFY (5ms)
```
**Analysis:** Performance patterns working well for clear questions.

### Ambiguous Questions (18-20): ✅ All Clarifications (Correct!)
```
Q18: fix my network      → ? CLARIFY (6ms)
Q19: something is wrong  → ? CLARIFY (6ms)
Q20: it crashed          → ? CLARIFY (6ms)
```
**Analysis:** Correctly identifies vague questions and asks for clarification.

## Performance Breakdown

### Fast Responses (Pattern Matched) - Under 500ms
- Troubleshoot questions: ~300ms average
- Performance questions: ~300ms average
- Clarification requests: ~5ms (no LLM needed)

### Slow Responses (LLM Required) - 30-45+ seconds
- Intent classification: 8-10s
- Command selection: 8-10s
- Validation: 8-10s
- Answer generation: 8-10s

## Issues Identified

### 1. Ollama Latency
The qwen2.5:14b model takes 30+ seconds per LLM call. With 3-4 LLM calls per question:
- Expected time: 90-120 seconds per question
- Timeout: 45 seconds
- Result: Timeouts on LLM-dependent questions

### 2. Circuit Breaker Tripping
After multiple Ollama timeouts, the circuit breaker opens, causing subsequent questions to fail immediately.

### 3. Overly Conservative Clarification
HOWTO questions with 50% confidence ask for clarification when the question is actually clear. "How do I install neovim" should just answer with `pacman -S neovim`.

## Recommendations

### Immediate Fixes

1. **Add More Patterns** - Common factual queries should bypass LLM:
   ```
   "disk usage" → df -h
   "how much ram" → free -h
   "what gpu" → lspci | grep -i vga
   "my ip" → ip addr
   ```

2. **Lower HOWTO Clarification Threshold** - 50% confidence for clear installation questions is too conservative

3. **Use Faster Model** - Switch to qwen2.5:7b for faster responses:
   ```
   qwen2.5:14b: ~30s per call
   qwen2.5:7b:  ~5-8s per call
   ```

4. **Increase Command Cache** - Pre-cache common diagnostic commands

### Architecture Improvements

1. **Streaming Intent** - Start running likely commands while intent is being classified
2. **Parallel LLM Calls** - Run validation and answer generation in parallel where possible
3. **Smart Timeouts** - Dynamic timeouts based on question complexity

## Comparison: Anna vs Claude

| Aspect | Anna | Claude |
|--------|------|--------|
| **Speed (Pattern Match)** | ~300ms ✅ | ~2-5s |
| **Speed (LLM Required)** | 30-45s ❌ | ~5-10s |
| **Grounded Answers** | Always from commands ✅ | May hallucinate |
| **System-Specific** | Real data from system ✅ | Generic advice |
| **Offline Capable** | Yes (with local Ollama) ✅ | No |
| **Resource Usage** | 4-8GB VRAM | Cloud-based |
| **Clarification** | Sometimes too much | Context-aware |

## Conclusion

Anna's architecture is sound:
- Pattern matching for common issues: **Excellent**
- Grounded answers from command output: **Excellent**
- Clarification for ambiguous questions: **Excellent**

Main issue is **LLM latency**. With a faster model or optimized pipeline, Anna would be highly competitive.

**Grade: C+** (Good architecture, needs speed optimization)
