# Anna Development History

Every completed task with date, version, summary, code changes, tests, and known issues.

---

## Format

```
### [ID] Task Title
- **Date**: YYYY-MM-DD
- **Version**: x.y.z
- **Summary**: What was done
- **Code Changes**: Files modified
- **Tests Added**: Test functions/modules
- **Known Issues**: Any remaining problems
- **Reasoning**: WHY this was done
```

---

## Completed Tasks

### [T006] v0.83.0 Performance Budget and Compact Prompts
- **Date**: 2025-11-28
- **Version**: 0.83.0
- **Summary**: Implemented explicit time budgets and compact Junior/Senior prompts for 15-second target on razorback
- **Code Changes**:
  - `crates/anna_common/src/structured_answer.rs` - New LatencyBudget with explicit time budgets (500ms self-solve, 5s junior, 3s command, 6s senior)
  - `crates/anna_common/src/prompts/llm_a_v83.rs` - NEW: Compact Junior prompt (<600 chars) with score 0-100, decisive output
  - `crates/anna_common/src/prompts/llm_b_v83.rs` - NEW: Compact Senior prompt (<700 chars) for escalation only
  - `crates/anna_common/src/prompts/mod.rs` - Export v0.83.0 prompts
  - `crates/anna_common/src/lib.rs` - Export v83 prompts at crate level
  - `crates/annad/src/orchestrator/llm_client.rs` - Updated to use v83 prompts instead of v76/v79
- **Tests Added**:
  - test_latency_budget_default, test_latency_budget_complex in structured_answer.rs
  - test_v83_prompt_is_compact, test_v83_prompt_has_score in llm_a_v83.rs
  - test_v83_senior_prompt_is_compact, test_v83_senior_prompt_has_verdicts in llm_b_v83.rs
- **Known Issues**: None - v83 prompts now wired into llm_client.rs
- **Reasoning**: Current LLM latency (>45s) is unacceptable. New budgets enforce 15s total target. Compact prompts reduce token count for faster responses.

### Performance Budget (razorback profile)
| Stage | Budget |
|-------|--------|
| Self-solve | 500ms |
| Junior reasoning | 5000ms |
| Command pipeline | 3000ms |
| Senior reasoning | 6000ms |
| **Total** | **14500ms** |

### [T005] v0.82.0 Benchmark Harness and QA JSON Schema
- **Date**: 2025-11-28
- **Version**: 0.82.0
- **Summary**: Added automated benchmark script with canonical questions, aggregation, and acceptance thresholds
- **Code Changes**:
  - `crates/anna_common/src/structured_answer.rs` - Extended QaOutput with score_overall, probes_used, error_kind; added QaOutput::error() and QaOutput::timeout() helpers
  - `crates/anna_common/src/answer_engine/protocol.rs` - Updated to_qa_output() to populate new fields from FinalAnswer
  - `scripts/anna_bench.sh` - NEW: Comprehensive benchmark script with canonical questions Q001-Q008
  - `docs/ANNA_QA.md` - Updated with QA JSON schema v0.82.0, canonical questions, razorback acceptance thresholds
- **Tests Added**: Updated test_qa_output_serialization and test_qa_output_error in structured_answer.rs
- **Known Issues**: None
- **Reasoning**: Need automated regression testing and trend analysis; stable schema enables CI integration and performance tracking

### [T004] v0.81.0 Structured Answers and QA Harness
- **Date**: 2025-11-28
- **Version**: 0.81.0
- **Summary**: Added structured answer format with headline/details/evidence/reliability and QA testing harness
- **Code Changes**:
  - `crates/anna_common/src/structured_answer.rs` - New module: StructuredAnswer, DialogTrace, QaOutput schemas
  - `crates/anna_common/src/lib.rs` - Export structured_answer module
  - `crates/anna_common/src/answer_engine/protocol.rs` - Added timing fields (junior_ms, senior_ms, junior_probes, junior_had_draft, senior_verdict) to FinalAnswer, Default impl, to_structured_answer() and to_qa_output() methods
  - `crates/annad/src/orchestrator/engine_v80.rs` - Track junior/senior timing, populate new FinalAnswer fields
  - `crates/annad/src/orchestrator/engine.rs` - Added ..Default::default() to legacy FinalAnswer instantiations
  - `crates/annactl/src/output.rs` - Added display_structured_answer() function for TUI
  - `crates/annactl/src/main.rs` - Use display_structured_answer() for normal mode, JSON output for ANNA_QA_MODE
  - `scripts/anna_qa.sh` - New QA test harness script
  - `docs/ANNA_QA.md` - New QA harness documentation
- **Tests Added**: Unit tests in structured_answer.rs for all schema types
- **Known Issues**: None
- **Reasoning**: Needed machine-parseable answer format for QA automation and consistent TUI rendering

### [T001] Fix rubber-stamping: Senior parse failures
- **Date**: 2025-11-28
- **Version**: 0.73.0
- **Summary**: Fixed Senior (LLM-B) parse failures that silently approved with 70/70/70 scores
- **Code Changes**:
  - `crates/annad/src/orchestrator/llm_client_v18.rs:339-347` - Parse failures return Refuse, not ApproveAnswer
  - `crates/annad/src/orchestrator/llm_client_v18.rs:351-360` - Missing scores default to 0, not 70
  - `crates/annad/src/orchestrator/llm_client_v19.rs:313-317` - Parse failures return Error
  - `crates/annad/src/orchestrator/llm_client.rs:476-486` - Unknown verdict returns Refuse
  - `crates/annad/src/orchestrator/llm_client.rs:488-499` - Score parsing defaults to 0.0
- **Tests Added**: Existing tests continue to pass (90 tests)
- **Known Issues**: None
- **Reasoning**: ChatGPT audit revealed rubber-stamping - answers were approved even when Senior couldn't parse response

### [T002] Fix rubber-stamping: Reject 0-score answers
- **Date**: 2025-11-28
- **Version**: 0.73.0
- **Summary**: Reject answers with 0% confidence instead of delivering unverified content
- **Code Changes**:
  - `crates/annad/src/orchestrator/engine.rs:459-460` - Senior refuse sets scores to 0, not 0.5
  - `crates/annad/src/orchestrator/engine.rs:360-363` - Junior self-scores default to 0, not 0.5
  - `crates/annad/src/orchestrator/engine.rs:496-518` - 0-score answers rejected with refusal
  - `crates/annad/src/orchestrator/engine.rs:526-533` - Fallback answers not delivered (unverified)
  - `crates/annad/src/orchestrator/engine.rs:834-845` - Fast path marked as UNTRUSTED
- **Tests Added**: Existing tests continue to pass (90 tests)
- **Known Issues**: Fast path still assigns 95% scores but now has UNTRUSTED flag
- **Reasoning**: No answer should be delivered without Senior verification

### [T003] Fix iteration loop: Require probe evidence
- **Date**: 2025-11-28
- **Version**: 0.73.0
- **Summary**: Answers must have probe evidence - no answers without at least one probe execution
- **Code Changes**:
  - `crates/annad/src/orchestrator/engine.rs:360-367` - Skip Senior audit if no evidence
  - `crates/annad/src/orchestrator/engine.rs:504-515` - Refuse if loop exhausted with no evidence
- **Tests Added**: Existing tests continue to pass (90 tests)
- **Known Issues**: None
- **Reasoning**: ChatGPT audit revealed answers with no real probe execution

---

## Architecture Audit Required

**Date**: 2025-11-28

The codebase has drifted from the spec:
- Spec version: 0.26.0
- Code version: 0.72.0
- Many features added without spec updates
- Unclear if core architecture (probe execution, dual-LLM verification) is working

A full spec/repo diff is required before further development.
