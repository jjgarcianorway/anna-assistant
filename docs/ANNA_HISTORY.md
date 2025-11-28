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
