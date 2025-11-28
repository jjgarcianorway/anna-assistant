# Anna TODO Tracker

All pending tasks with ID, description, and status.

Status: TODO -> IN_PROGRESS -> DONE

---

## v0.73.0 Engineering Reset

### Critical Fixes (Core Pipeline)

| ID | Description | Status | Priority |
|----|-------------|--------|----------|
| T001 | Fix rubber-stamping: Senior parse failures must NOT approve with 70/70/70 | DONE | CRITICAL |
| T002 | Fix rubber-stamping: Reject answers with overall score 0 | DONE | CRITICAL |
| T003 | Fix iteration loop: Must run at least 1 real probe before answering | DONE | CRITICAL |
| T004 | Test: CPU question - "What CPU do I have?" | DONE | CRITICAL |
| T005 | Test: RAM question - "How much RAM do I have?" | DONE | CRITICAL |
| T006 | Test: Disk question - "What disks do I have?" | DONE | CRITICAL |
| T007 | Test: GPU question - "What GPU do I have?" | DONE | CRITICAL |
| T008 | Test: OS question - "What operating system?" | DONE | CRITICAL |
| T009 | Update ANNA_SPEC.md to reflect fixed v0.73.0 | TODO | HIGH |

### Test Results (v0.73.2)

| Test | Answer | Reliability | Notes |
|------|--------|-------------|-------|
| T004 CPU | Intel i9-14900HX | 0% Red | Correct (Senior didn't score) |
| T005 RAM | 31 GB | 80% Yellow | Correct with real score |
| T006 Disk | nvme0n1 + partitions | 80% Yellow | Correct with real score |
| T007 GPU | NVIDIA RTX 4060 Max-Q | 70% Yellow | Correct with real score |
| T008 OS | Refused | 0% Red | Correct refusal (no OS probe) |

### Deferred (Keep but UNTRUSTED)

| Feature | Status | Notes |
|---------|--------|-------|
| Progression/XP | KEEP | Numbers may change after fixes |
| Auto-update | KEEP | Untouched |
| Stats display | KEEP | Numbers may change |
| Debug streaming | KEEP | Untouched |
| Fast path | KEEP | May need review after core fix |

---

## Completed

(See ANNA_HISTORY.md)
